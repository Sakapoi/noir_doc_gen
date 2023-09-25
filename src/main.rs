use std::{io::{Write, Read}, collections::BTreeMap, fs::File};
use handlebars::Handlebars;

fn get_doc(input_file: &str) -> Result<Vec<SpannedToken>, Box<dyn std::error::Error>> {
    let mut file = File::open(input_file)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let parsed_str = Lexer::lex(&contents);

    Ok(parsed_str.0.0)
}


fn generate_doc(input_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open(input_file)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let parsed_str = Lexer::lex(&contents);

    let mut data = BTreeMap::new();

    dbg!(parsed_str.0.0[0].to_string());

    data.insert("comments".to_string(), "qwe"); 

    let mut file = File::create("comments.html")?;

    let mut handlebars = Handlebars::new();

    handlebars.register_template_string("comments_template", include_str!("../comments_template.html"))
    .expect("Failed to register HTML template");

    let rendered_html = handlebars.render("comments_template", &data)?;
    dbg!(&rendered_html);
    file.write_all(rendered_html.as_bytes())?;

    Ok(())
}

use noirc_frontend::{parser, lexer::Lexer};

fn main() {
    let code = r#"
        //qwe
        ///doc comments
    "#;

    dbg!(parser::parse_program(code));

    let qwe = Lexer::lex(code);

    dbg!(qwe.0.0);
    dbg!(qwe.1);

    generate_doc("prog.nr").unwrap();
}





