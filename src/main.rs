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

    println!("# source =>");
    println!("{}", source);

    println!("# lexing =>");
    let tokens = lexer::lex(&source);
    tokens.iter().for_each(|f| match f.kind {
        tokens::TokenKind::SyntaxError => {
            println!(
                "! syntax error at '{}' (line {}, column {})",
                f.lexeme, f.position.line, f.position.column,
            )
        }
        _ => println!("token: {:?}", f.kind),
    });
    println!();

    println!("# parsing =>");
    let ast = parser::parse(tokens);
    println!("tokens: {:?}", ast);

    println!("# code gen =>");
    let bytecodes = codegen::codegen(ast);
    println!("byte codes: {:?}", bytecodes);

    println!("# vm =>");
    vm::run(bytecodes);
}
