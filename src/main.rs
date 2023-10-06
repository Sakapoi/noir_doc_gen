use std::{io::{Write, Read, BufReader, BufRead}, fs::File, fmt, vec};
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

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Function => write!(f, "Function"),
            Type::Module => write!(f, "Module"),
            Type::Struct => write!(f, "Struct"),
            Type::Trait => write!(f, "Trait"),
            Type::OuterComment => write!(f, "OuterComment"),
        }
    }
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
        additional_doc: String,
        implementations: Vec<Implementation>,
    },
    Trait {
        signature: String,
        additional_doc: String,
        required_methods: Vec<Function>,
        provided_methods: Vec<Function>,
        implementations: Vec<Implementation>,
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

    fn get_implementations(&self) -> Option<Vec<Implementation>> {
        match self {
            Info::Struct { implementations, .. } => {
                Some(implementations.clone())
            }
            Info::Trait { implementations, .. } => {
                Some(implementations.clone())
            }
            _ => {
                None
            }
        }
    }

    fn get_additional_doc(&self) -> Option<String> {
        match self {
            Info::Struct { additional_doc, .. } => {
                Some(additional_doc.to_string())
            }
            Info::Trait { additional_doc, .. } => {
                Some(additional_doc.to_string())
            }
            _ => {
                None
            }
        }
    }

    fn get_required_methods(&self) -> Option<Vec<Function>> {
        match self {
            Info::Trait { required_methods, .. } => {
                Some(required_methods.clone())
            }
            _ => {
                None
            }
        }
    }

    fn get_provided_methods(&self) -> Option<Vec<Function>> {
        match self {
            Info::Trait { provided_methods, .. } => {
                Some(provided_methods.clone())
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
        let mut skip_count = 0;

        for i in 0..tokens.len() {
            if skip_count > 0 {
                skip_count -= 1;
                continue;
            }
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
                    let sign = fn_signature(&tokens, i - 1);
                    
                    Output{r#type, name, doc, information: Info::Function { signature: sign }}
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
                    let sign = struct_signature(&tokens, i - 1);
                    let ad_doc = additional_doc(&tokens, i);

                    Output{r#type, name: name.clone(), doc, information: Info::Struct { signature: sign, additional_doc: ad_doc, implementations: Implementation::get_implementations(&tokens, i, name) }}
                }
                Token::Keyword(Keyword::Trait) => {
                    skip_count = skip_impl_block(&tokens, i);

                    let r#type = Type::Trait;
                    let name = match &tokens[i + 1] {
                        Token::Ident(idn) => {
                            idn.clone()
                        }
                        _ => {continue;}
                    };
                    let doc = doc(&tokens, i);

                    let ad_doc = additional_doc(&tokens, i);
                    let impls = Implementation::get_implementations(&tokens, i, name.clone());
                    let info = trait_info(&tokens, i);

                    Output{r#type, name, doc, information: Info::Trait { signature: info.0, additional_doc: ad_doc, required_methods: info.1, provided_methods: info.2, implementations: impls }}
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
                Token::Keyword(Keyword::Impl) => {
                    skip_count = skip_impl_block(&tokens, i);
                    continue;
                }
                _ => {continue;}
            };

            res.push(out);
        }

        res
    }
}

impl fmt::Display for Output {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Type: {:?}\n", self.r#type)?;
        write!(f, "Name: {}\n", self.name)?;
        write!(f, "Doc: {}\n", self.doc)?;
        Ok(())
    }
}

fn skip_impl_block(tokens: &[Token], index: usize) -> usize {
    let mut brace_counter = 0;
    let mut i = index;

    while brace_counter != 1 {
        match &tokens[i] {
            Token::LeftBrace => {
                i += 1;
                brace_counter += 1;
            }
            _ => {
                i += 1;
            }
        }
    }

    while brace_counter != 0 {
        match &tokens[i] {
            Token::LeftBrace => {
                i += 1;
                brace_counter += 1;
            }
            Token::RightBrace => {
                i += 1;
                brace_counter -= 1;
            }
            _ => {
                i += 1;
            }
        }
    }

    i - index - 1
}

fn fn_signature(tokens: &[Token], index: usize) -> String {
    let mut res = String::new();
    let mut i = index;
    loop {
        match &tokens[i + 1] {
            Token::LeftBrace | Token::Semicolon => {
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

fn struct_signature(tokens: &[Token], index: usize) -> String {
    let mut res = String::new();
    let mut i = index;
    let mut is_private = true;

    loop {
        match &tokens[i + 1] {
            Token::LeftBrace => {
                res.push_str("{");
                res.push_str("\n");
                loop {
                    match tokens[i + 1] {
                        Token::RightBrace => {
                            if is_private {
                                res.push_str("/* private fields */");
                            }
                            res.push_str("\n");
                            res.push_str("}");
                            break;
                        }
                        Token::Keyword(Keyword::Pub) => {
                            is_private = false;
                            loop {
                                match tokens[i + 1] {
                                    Token::Comma => {
                                        if tokens[i + 2] == Token::RightBrace {
                                            res.push_str(",");
                                        }
                                        else {
                                            res.push_str(",\n");
                                        }
                                        i += 1;
                                        break;
                                    }
                                    Token::RightBrace => {
                                        break;
                                    }
                                    _ => {
                                        res.push_str(&tokens[i + 1].to_string());
                                        res.push_str(" ");
                                        i += 1;
                                    }
                                }
                            }
                        }
                        _ => { i += 1; }
                    }
                }
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

fn trait_info(tokens: &[Token], index: usize) -> (String, Vec<Function>, Vec<Function>) {
    let mut sign = String::new();
    let mut required_methods = Vec::new();
    let mut provided_methods = Vec::new();
    let mut i = index;
    let mut brace_counter;

    loop {
        match &tokens[i + 1] {
            Token::LeftBrace => {
                sign.push_str("{");
                sign.push_str("\n");
                loop {
                    match tokens[i + 1] {
                        Token::RightBrace => {
                            sign.push_str("}");
                            break;
                        }
                        Token::Keyword(Keyword::Fn) => {
                            let name = match &tokens[i + 2] {
                                Token::Ident(idn) => {
                                    idn.clone()
                                }
                                _ => {break;}
                            };
                            let doc = doc(&tokens, i + 1);
                            let fn_sign = fn_signature(&tokens, i);

                            loop {
                                match tokens[i + 1] {
                                    Token::Semicolon => {
                                        required_methods.push(Function { name, doc, signature: fn_sign, is_method: true });
                                        sign.push_str(";");
                                        sign.push_str("\n");
                                        break;
                                    }
                                    Token::LeftBrace => {
                                        provided_methods.push(Function { name, doc, signature: fn_sign, is_method: true });
                                        brace_counter = 1;
                                        sign.push_str("{ ... }");
                                        sign.push_str("\n");
                                        while brace_counter != 0 {
                                            i += 1;
                                            match tokens[i + 1] {
                                                Token::LeftBrace => {
                                                    brace_counter +=1;
                                                }
                                                Token::RightBrace => {
                                                    brace_counter -=1;
                                                }
                                                _ => {}
                                            }
                                        }
                                        i +=1;
                                        break;
                                    }
                                    _ => {
                                        sign.push_str(&tokens[i + 1].to_string());
                                        sign.push_str(" ");
                                        i += 1;
                                    }
                                }
                            }
                        }
                        _ => { i += 1; }
                    }
                }
                break;
            }
            _ => {
                sign.push_str(&tokens[i + 1].to_string());
                sign.push_str(" ");
                i += 1;
            }
        };
    }

    (sign, required_methods, provided_methods)
}

fn additional_doc(tokens: &[Token], index: usize) -> String {
    let res = match &tokens[index - 1] {
        Token::DocComment(DocComments::Outer(dc)) => {
            let mut res = dc.to_string();
            let mut doc_end = true;
            let mut iter = 2;
            while doc_end && ((index as i32) - (iter as i32)) >= 0 {
                match &tokens[index - iter] {
                    Token::DocComment(DocComments::Outer(doc)) => {
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
                    Token::DocComment(DocComments::Outer(doc)) => {
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

#[derive(Debug, Clone, Template)]
#[template(path = "func_template.html")]
struct Function {
    name: String, 
    doc: String, 
    signature: String,
    is_method: bool,
}

fn generate_function_pages(func: Function) -> Result<(), Box<dyn std::error::Error>> {
    if func.is_method {
        return Ok(());
    }
    let rendered_html = func.render().unwrap();

    let output_file_name = format!("generated_doc/{}.html", func.name);

    let mut file = File::create(output_file_name)?;
    file.write_all(rendered_html.as_bytes())?;

    Ok(())
}

#[derive(Debug, Template)]
#[template(path = "struct_template.html")]
struct Structure {
    name: String, 
    doc: String, 
    additional_doc: String,
    signature: String,
    implementations: Vec<Implementation>,
}

#[derive(Debug, Clone)]
struct Implementation {
    signature: String,
    functions: Vec<Function>,
}

impl Implementation {
    fn get_implementations(tokens: &[Token], index: usize, orig_name: String) -> Vec<Implementation> {
        let mut res = Vec::new();
        let mut functions = Vec::new();
        let mut signature = String::new();
        let mut right_impl = false;
        let mut i = index;
        let mut brace_counter = 0;

        while i < tokens.len() {
            match tokens[i] {
                Token::Keyword(Keyword::Impl) => {
                    loop {
                        match &tokens[i] {
                            Token::Ident(name) => {
                                if name == &orig_name {
                                    right_impl = true;
                                }
                                signature.push_str(&tokens[i].to_string());
                                signature.push_str(" ");
                                i +=1;
                            }
                            Token::LeftBrace => {
                                if !right_impl {
                                    signature = "".to_string();
                                    break;
                                }
                                else {
                                    brace_counter += 1;
                                    i += 1;
                                    while brace_counter != 0 {
                                        match &tokens[i] {
                                            Token::Keyword(Keyword::Fn) => {
                                                let name = match &tokens[i + 1] {
                                                    Token::Ident(idn) => {
                                                        idn.clone()
                                                    }
                                                    _ => {continue;}
                                                };
                                                let doc = doc(&tokens, i);
                                                let sign = fn_signature(&tokens, i - 1);
                                                
                                                functions.push(Function{ name, doc, signature: sign, is_method: true });

                                                i += 1;
                                            }
                                            Token::LeftBrace => {
                                                i += 1;
                                                brace_counter += 1;
                                            }
                                            Token::RightBrace => {
                                                i += 1;
                                                brace_counter -= 1;
                                            }
                                            _ => {
                                                i += 1;
                                            }
                                        }
                                    }

                                    res.push(Implementation { signature: signature.clone(), functions: functions.clone() });
                                    signature = "".to_string();
                                    functions = vec![];
                                    break;
                                }
                            }
                            _ => {
                                signature.push_str(&tokens[i].to_string());
                                signature.push_str(" ");
                                i +=1;
                            }
                        }
                    }
                }
                _ => {i += 1;}
            }
        }

        res
    }
}

fn generate_structure_pages(structure: Structure) -> Result<(), Box<dyn std::error::Error>> {
    let rendered_html = structure.render().unwrap();

    let output_file_name = format!("generated_doc/{}.html", structure.name);

    let mut file = File::create(output_file_name)?;
    file.write_all(rendered_html.as_bytes())?;

    Ok(())
}

#[derive(Debug, Template)]
#[template(path = "trait_template.html")]
struct Trait {
    name: String, 
    doc: String, 
    signature: String,
    additional_doc: String,
    required_methods: Vec<Function>,
    provided_methods: Vec<Function>,
    implementations: Vec<Implementation>,
}

fn generate_trait_pages(r#trait: Trait) -> Result<(), Box<dyn std::error::Error>> {
    let rendered_html = r#trait.render().unwrap();

    let output_file_name = format!("generated_doc/{}.html", r#trait.name);

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
                generate_function_pages(
                    Function { 
                        name: i.name.clone(), 
                        doc: i.doc.clone(), 
                        signature: i.information.get_signature().unwrap(),
                        is_method: false, 
                    }
                )?;
            } 
            Type::Struct => {
                generate_structure_pages(
                    Structure { 
                        name: i.name.clone(), 
                        doc: i.doc.clone(), 
                        additional_doc: i.information.get_additional_doc().unwrap(),
                        signature: i.information.get_signature().unwrap(), 
                        implementations: i.information.get_implementations().unwrap()
                    } 
                )?;
            } 
            Type::Trait => {
                generate_trait_pages(
                    Trait { 
                        name: i.name.clone(),
                        doc: i.doc.clone(), 
                        signature: i.information.get_signature().unwrap(), 
                        additional_doc: i.information.get_additional_doc().unwrap(),
                        required_methods: i.information.get_required_methods().unwrap(), 
                        provided_methods: i.information.get_provided_methods().unwrap(), 
                        implementations: i.information.get_implementations().unwrap()
                    }
                )?;
            }
            _ => {}
        }
    }

    Ok(())
}

fn main() {
    generate_doc("input_files/prog.nr").unwrap();

    // let mut file = File::open("input_files/outer_com.nr").unwrap();
    // let mut contents = String::new();
    // file.read_to_string(&mut contents).unwrap();

    // let res = parse_program(&contents);
    // let impls = dbg!(res.0.impls);
}
