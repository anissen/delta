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
    let tokens = lexer::lex(source);
    let (tokens, syntax_errors): (Vec<tokens::Token>, Vec<tokens::Token>) =
        tokens.into_iter().partition(|token| match token.kind {
            tokens::TokenKind::SyntaxError(_) => false,
            _ => true,
        });
    syntax_errors.iter().for_each(|token| match token.kind {
        tokens::TokenKind::SyntaxError(description) => {
            println!(
                "\n⚠️ syntax error: {} at {:?} ({:?})\n",
                description, token.lexeme, token.position
            )
        }
        _ => panic!(),
    });

    tokens.iter().for_each(|token| {
        println!(
            "token: {:?} at '{}' (line {}, column: {})",
            token.kind, token.lexeme, token.position.line, token.position.column
        )
    });

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

    syntax_errors.iter().for_each(|token| match token.kind {
        tokens::TokenKind::SyntaxError(description) => {
            println!(
                "\n⚠️ syntax error: {} at {:?} ({:?})\n",
                description, token.lexeme, token.position
            )
        }
        _ => panic!(),
    });

    Ok(result)
}
