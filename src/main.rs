use std::{io::{Write, Read}, fs::File};
use handlebars::{Handlebars, to_json};
use noirc_frontend::{lexer::Lexer, token::{Token, SpannedToken, Keyword}};

#[derive(Debug, Clone, Copy, serde::Serialize, PartialEq)]
enum Type {
    Function,
    Module,
    Struct,
    Trait,
    Implementation,
}

#[derive(Debug, Clone, serde::Serialize)]
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

fn generate_doc(tokens: Vec<Output>) -> Result<(), Box<dyn std::error::Error>> {
    let mut handlebars = Handlebars::new();

    handlebars.register_template_file("doc_template", "doc_template.html")
    .expect("Failed to register HTML template");

    let mut data = std::collections::BTreeMap::new();

    data.insert("functions", to_json(&tokens.iter().filter(|Output {r#type, ..}| *r#type == Type::Function ).collect::<Vec<_>>()));
    data.insert("modules", to_json(&tokens.iter().filter(|Output {r#type, ..}| *r#type == Type::Module ).collect::<Vec<_>>()));
    data.insert("structs", to_json(&tokens.iter().filter(|Output {r#type, ..}| *r#type == Type::Struct ).collect::<Vec<_>>()));
    data.insert("traits", to_json(&tokens.iter().filter(|Output {r#type, ..}| *r#type == Type::Trait ).collect::<Vec<_>>()));

    let rendered_html = handlebars.render("doc_template", &data)?;

    let mut file = File::create("comments.html")?;
    file.write_all(rendered_html.as_bytes())?;

    Ok(())
}

fn main() {
    let doc = get_doc("prog.nr").unwrap();

    let res = Output::to_output(doc);

    dbg!(&res);

    generate_doc(res).unwrap();
}
