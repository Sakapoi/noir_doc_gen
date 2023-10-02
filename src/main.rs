use std::{io::{Write, Read, BufReader, BufRead}, fs::File};
use noirc_frontend::{lexer::Lexer, token::{Token, SpannedToken, Keyword, DocComments}};
use askama::Template;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Type {
    Function,
    Module,
    Struct,
    Trait,
    OuterComment,
}

#[derive(Debug, Clone)]
enum Info {
    Function{
        signature: String,
    },
    Module{
        // ??
    },
    Struct {
        signature: String,
        implementations: String,
    },
    Trait {
        signature: String,
        additional_doc: String,
        implementations: String,
    },
    Blanc,
}

impl Info {
    fn get_signature(&self) -> Option<String> {
        match self {
            Info::Function { signature } => {
                Some(signature.to_string())
            },
            Info::Struct { signature, .. } => {
                Some(signature.to_string())
            }
            Info::Trait { signature, .. } => {
                Some(signature.to_string())
            }
            _ => {
                None
            }
        }
    }
}

#[derive(Debug, Clone)]
struct Output {
    r#type: Type,
    name: String,
    doc: String,
    information: Info,
}

impl Output {
    fn to_output(input: Vec<SpannedToken>) -> Vec<Self> {
        let mut res = Vec::new();
        let tokens = input.into_iter().map(|x| x.into_token()).collect::<Vec<_>>();
        let mut is_first = true;

        for i in 0..tokens.len() {
            let out = match &tokens[i] {
                Token::Keyword(Keyword::Fn) => {
                    let r#type = Type::Function;
                    let name = match &tokens[i + 1] {
                        Token::Ident(idn) => {
                            idn.clone()
                        }
                        _ => {continue;}
                    };
                    let doc = doc(&tokens, i);
                    let sign = signature(&tokens, i - 1);
                    
                    Output{r#type, name, doc, information: Info::Function { signature: sign }}
                }
                Token::Keyword(Keyword::Mod) => {
                    let r#type = Type::Module;
                    let name = match &tokens[i + 1] {
                        Token::Ident(idn) => {
                            idn.clone()
                        }
                        _ => {continue;}
                    };
                    let doc = doc(&tokens, i);
                    Output{r#type, name, doc, information: Info::Blanc}
                }
                Token::Keyword(Keyword::Struct) => {
                    let r#type = Type::Struct;
                    let name = match &tokens[i + 1] {
                        Token::Ident(idn) => {
                            idn.clone()
                        }
                        _ => {continue;}
                    };
                    let doc = doc(&tokens, i);
                    Output{r#type, name, doc, information: Info::Blanc}
                }
                Token::Keyword(Keyword::Trait) => {
                    let r#type = Type::Trait;
                    let name = match &tokens[i + 1] {
                        Token::Ident(idn) => {
                            idn.clone()
                        }
                        _ => {continue;}
                    };
                    let doc = doc(&tokens, i);
                    Output{r#type, name, doc, information: Info::Blanc}
                }
                Token::DocComment(DocComments::Outer(_)) => {
                    let r#type = Type::OuterComment;
                    let name = "".to_string();

                    let res = outer_doc(&tokens, i);

                    let doc = if is_first {
                        is_first = false;
                        res.0
                    }
                    else {
                        if res.1 == i {
                            is_first = true;
                        }
                        "".to_string()
                    };
                    

                    Output{r#type, name, doc, information: Info::Blanc}
                }
                _ => {continue;}
            };

            res.push(out);
        }

        res
    }
}

fn signature(tokens: &[Token], index: usize) -> String {
    let mut res = String::new();
    let mut i = index;
    loop {
        match &tokens[i + 1] {
            Token::LeftBrace => {
                break;
            }
            _ => {
                res.push_str(&tokens[i + 1].to_string());
                res.push_str(" ");
                i += 1;
            }
        };
    }
    res
}

fn doc(tokens: &[Token], index: usize) -> String {
    let res = match &tokens[index - 1] {
        Token::DocComment(DocComments::Single(dc)) | 
        Token::DocComment(DocComments::Block(dc)) => {
            let mut res = dc.to_string();
            let mut doc_end = true;
            let mut iter = 2;
            while doc_end && ((index as i32) - (iter as i32)) >= 0 {
                match &tokens[index - iter] {
                    Token::DocComment(DocComments::Single(doc)) | 
                    Token::DocComment(DocComments::Block(doc)) => {
                        res.insert_str(0, &doc.to_string());
                        iter += 1;
                    }
                    _ => {
                        doc_end = false;
                    }
                }
                
            }
            res
        }
        _ => {
            let mut res = String::new();
        
            let mut doc_find = true;
            let mut iter = 2;
            while doc_find && ((index as i32) - (iter as i32)) >= 0 {
                match &tokens[index - iter] {
                    Token::DocComment(DocComments::Single(doc)) | 
                    Token::DocComment(DocComments::Block(doc)) => {
                        res.insert_str(0, &doc.to_string());
                        iter += 1;
                    }
                    Token::Keyword(Keyword::Fn) | Token::Keyword(Keyword::Mod) |
                    Token::Keyword(Keyword::Struct) | Token::Keyword(Keyword::Trait) |
                    Token::Keyword(Keyword::Impl) => {
                        doc_find = false;
                    }
                    _ => { iter += 1; }
                }
                
            }
            res
        }
    };
    res
}

fn outer_doc(tokens: &[Token], index: usize) -> (String, usize) {
    let mut i = index;
    let mut res = tokens[i].to_string();
    let mut doc_find = true;
    while doc_find {
        match &tokens[i + 1] {
            Token::DocComment(DocComments::Outer(doc)) => {
                res.push_str(doc);
                i += 1;
            }
            _ => { doc_find = false; }
        }
    }

    (res, i)
}

fn get_doc(input_file: &str) -> Result<Vec<SpannedToken>, Box<dyn std::error::Error>> {
    let mut file = File::open(input_file)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let parsed_str = Lexer::lex(&contents);

    Ok(parsed_str.0.0)
}

#[derive(Template)]
#[template(path = "code_template.html")]
struct Code {
    codelines: Vec<CodeLine>,
}

#[derive(Debug)]
struct CodeLine {
    number: u32,
    text: String,
}

fn get_text(input_file: &str) -> Result<Vec<CodeLine>, Box<dyn std::error::Error>> {
    let file = File::open(input_file)?;
    let reader = BufReader::new(file);
    let mut code = Vec::new();
    let mut i = 0;

    for line in reader.lines() {
        i += 1;
        code.push(CodeLine{ number: i, text: line? });
    }

    Ok(code)
}

fn generate_code_page(input_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let codelines = get_text(input_file)?;

    let code = Code{ codelines };

    let rendered_html = code.render().unwrap();

    let mut file = File::create("generated_doc/codepage.html")?;
    file.write_all(rendered_html.as_bytes())?;

    Ok(())
}

#[derive(Debug, Template)]
#[template(path = "func_template.html")]
struct Function {
    name: String, 
    doc: String, 
    signature: String
}

fn generate_function_pages(func: Function) -> Result<(), Box<dyn std::error::Error>> {
    let rendered_html = func.render().unwrap();

    let output_file_name = format!("generated_doc/{}.html", func.name);

    let mut file = File::create(output_file_name)?;
    file.write_all(rendered_html.as_bytes())?;

    Ok(())
}

#[derive(Debug, Template)]
#[template(path = "doc_template.html")]
struct AllOutput {
    all_output: Vec<Output>,
    filename: String,
}

fn generate_doc(input_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let doc = get_doc(input_file).unwrap();

    let tokens = Output::to_output(doc);

    let out = AllOutput{ all_output: tokens.clone(), filename: input_file.to_string() };

    let rendered_html = out.render().unwrap();

    let mut file = File::create("generated_doc/mainpage.html")?;
    file.write_all(rendered_html.as_bytes())?;

    generate_code_page(input_file)?;

    for i in tokens.iter() {
        match i.r#type {
            Type::Function => {
                generate_function_pages(Function { name: i.name.clone(), doc: i.doc.clone(), signature: i.information.get_signature().unwrap() })?;
            } 
            _ => {}
        }
    }

    Ok(())
}

fn main() {
    generate_doc("input_files/prog.nr").unwrap();
}
