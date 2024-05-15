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
    let source_path = "examples/workbench.∆";
    let mut file = File::open(source_path).expect("Unable to open file");
    let mut source = String::new();
    file.read_to_string(&mut source)
        .expect("Error reading file.");

    println!("\n# source =>");
    println!("{}", source);

    println!("\n# lexing =>");
    let tokens = lexer::lex(&source);
    tokens.iter().for_each(|token| match token.kind {
        tokens::TokenKind::SyntaxError => {
            println!(
                "! syntax error at '{}' (line {}, column {})",
                token.lexeme, token.position.line, token.position.column,
            )
        }
        _ => println!("token: {:?} ({:?})", token.kind, token.lexeme),
    });
    println!();

    println!("\n# parsing =>");
    let ast = parser::parse(tokens);
    println!("tokens: {:?}", ast);

    println!("\n# code gen =>");
    let bytecodes = codegen::codegen(ast);
    println!("byte codes: {:?}", bytecodes);

    println!("\n# vm =>");
    vm::run(bytecodes);
}
