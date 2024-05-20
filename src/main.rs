use std::{fs::File, io::Read};

mod bytecodes;
mod codegen;
mod expressions;
mod lexer;
mod parser;
mod tokens;
mod vm;

// https://github.com/brightly-salty/rox/

fn main() {
    let source_path = "examples/workbench.∆".to_string();
    let result = run(source_path);
    match result {
        Ok(Some(value)) => println!("Result: {:?}", value),

        Ok(None) => println!("Result: N/A"),

        Err(err) => println!("Error(s) occured:\n{}", err),
    }
}

fn run(source_path: String) -> Result<Option<vm::Value>, String> {
    let mut file = File::open(source_path).expect("Unable to open file");
    let mut source = String::new();
    file.read_to_string(&mut source)
        .expect("Error reading file.");

    println!("\n# source =>");
    println!("{}", source);

    println!("\n# lexing =>");
    let tokens = lexer::lex(&source)?;
    tokens
        .iter()
        .for_each(|token| println!("token: {:?} ({:?})", token.kind, token.lexeme));

    println!("\n# parsing =>");
    let ast = parser::parse(tokens)?;
    println!("ast: {:?}", ast);

    println!("\n# code gen =>");
    let bytecodes = codegen::codegen(ast);
    println!("byte codes: {:?}", bytecodes);

    println!("\n# vm =>");
    let result = vm::run(bytecodes);

    Ok(result)
}
