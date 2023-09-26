use std::{io::{Write, Read}, collections::BTreeMap, fs::File};
use handlebars::Handlebars;
use noirc_frontend::{parser, lexer::Lexer, token::{Token, SpannedToken, Keyword}};

#[derive(Debug)]
enum Type {
    Function,
    Module,
    Struct,
    Trait,
    Implementation,
}

#[derive(Debug)]
struct Output {
    r#type: Type,
    name: String,
    doc: String
}

impl Output {
    fn to_output(input: Vec<SpannedToken>) -> Vec<Self> {
        let mut res = Vec::new();
        let tokens = input.into_iter().map(|x| x.into_token()).collect::<Vec<_>>();

        for i in 0..tokens.len() {
            let r#type = match tokens[i] {
                Token::Keyword(Keyword::Fn) => {
                    Type::Function
                }
                Token::Keyword(Keyword::Mod) => {
                    Type::Module
                }
                Token::Keyword(Keyword::Struct) => {
                    Type::Struct
                }
                Token::Keyword(Keyword::Trait) => {
                    Type::Trait
                }
                Token::Keyword(Keyword::Impl) => {
                    Type::Implementation
                }
                _ => {continue;}
            };

            let name = match &tokens[i + 1] {
                Token::Ident(idn) => {
                    idn.clone()
                }
                _ => {continue;}
            };

            let doc = match &tokens[i - 1] {
                Token::DocComment(dc) => {
                    let mut res = dc.to_string();
                    let mut doc_end = true;
                    let mut iter = 2;
                    while doc_end && ((i as i32) - (iter as i32)) >= 0 {
                        match &tokens[i - iter] {
                            Token::DocComment(doc) => {
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
                    while doc_find && ((i as i32) - (iter as i32)) >= 0 {
                        match &tokens[i - iter] {
                            Token::DocComment(doc) => {
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

            res.push(Output{r#type, name, doc})
        }

        res
    }
}

fn get_doc(input_file: &str) -> Result<Vec<SpannedToken>, Box<dyn std::error::Error>> {
    let mut file = File::open(input_file)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let parsed_str = Lexer::lex(&contents);

    Ok(parsed_str.0.0)
}


fn generate_doc(tokens: Vec<(String ,String)>) -> Result<(), Box<dyn std::error::Error>> {
    let mut handlebars = Handlebars::new();

    handlebars.register_template_string("comments_template", include_str!("../comments_template.html"))
    .expect("Failed to register HTML template");

    let mut doc = Vec::new();

    for i in tokens.iter() {
        if i.0 == "doc-coments" {
            doc.push(&i.1);
        }
    }

    let mut data = std::collections::BTreeMap::new();
    data.insert("strings", &doc);

    let rendered_html = handlebars.render("comments_template", &data)?;

    let mut file = File::create("comments.html")?;
    file.write_all(rendered_html.as_bytes())?;

    Ok(())
}

fn main() {
    let doc = get_doc("prog.nr").unwrap();

    let res = Output::to_output(doc);

    dbg!(res);

    // generate_doc(doc).unwrap();
}


// {
//     "type": "module",
//     "name": "peter",
//     "doc": "mega harosh 1 2 3",
//     "path": "src/peter.nr"
//   }


