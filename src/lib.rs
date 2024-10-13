mod bytecodes;
mod codegen;
mod disassembler;
mod expressions;
mod lexer;
mod parser;
mod tokens;
pub mod vm;

use std::{fs::File, io::Read};

pub fn run_file(source_path: &String) -> Result<Option<vm::Value>, String> {
    let mut file = File::open(source_path).expect("Unable to open file");
    let mut source = String::new();
    file.read_to_string(&mut source)
        .expect("Error reading file.");

    run(&source, Some(source_path))
}

pub fn run(source: &String, file_name: Option<&String>) -> Result<Option<vm::Value>, String> {
    let default_file_name = "n/a".to_string();
    println!(
        "\n# source (file: {}) =>",
        file_name.unwrap_or(&default_file_name)
    );
    println!("{}", source);

    println!("\n# lexing =>");
    let tokens = lexer::lex(source)?;
    tokens
        .iter()
        .for_each(|token| println!("token: {:?} ({:?})", token.kind, token.lexeme));

    println!("\n# parsing =>");
    let ast = parser::parse(tokens)?;
    println!("ast: {:?}", ast);

    println!("\n# code gen =>");
    let bytecodes = codegen::codegen(ast);
    println!("byte codes: {:?}", bytecodes);

    println!("\n# disassembly =>");
    let disassembled = disassembler::disassemble(bytecodes.clone());
    println!("disassembled:");
    for ele in disassembled {
        println!("{:?}", ele);
    }

    println!("\n# vm =>");
    let result = vm::run(bytecodes);

    Ok(result)
}
