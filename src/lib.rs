mod bytecodes;
mod codegen;
mod disassembler;
mod expressions;
mod lexer;
mod parser;
pub mod program;
mod tokens;
pub mod vm;

use std::{fs::File, io::Read};

pub fn read_file(path: &String) -> String {
    let mut file = File::open(path).expect("Unable to open file");
    let mut source = String::new();
    file.read_to_string(&mut source)
        .expect("Error reading file.");
    source
}

pub fn run_file(source_path: &String) -> Result<Option<vm::Value>, String> {
    let source = read_file(source_path);
    run(&source, Some(source_path))
}

/*
    TODO(anissen): Create a program object that can be run repreatedly

    E.g.
    let context = delta::context::new();
    context.add_function("draw_circle", |call| {
        let x = call.get_float("x");
        let y = call.get_float("y");
        let radius = call.get_float("radius");
        draw_circle(x, y, radius, YELLOW);
    });
    (Alternatively use something like https://github.com/clarkmcc/cel-rust to be able to create typed arguments)

    // at some point, program also needs source code for foreign functions (for type checking)
    //
    let program = delta::program::new(context);
    program.compile();
    program.run("main", "setup");
    loop {
        program.run("main", "update");
        program.run("main", "draw");
    }
    program.dump("∆");
*/

// TODO(anissen): Make a concept of diagnostics (containing just syntax error for now)
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

    let context = program::Context::new();

    // TODO(anissen): Should use `program` w. diagnostics

    println!("\n# code gen =>");
    let bytecodes = codegen::codegen(ast, &context);
    println!("byte codes: {:?}", bytecodes);

    println!("\n# disassembly =>");
    let disassembled = disassembler::disassemble(bytecodes.clone());
    println!("disassembled:");
    for ele in disassembled {
        println!("{:?}", ele);
    }

    println!("\n# vm =>");
    let result = vm::run(bytecodes, &context);

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
    // Ok(Some(vm::Value::True))
}
