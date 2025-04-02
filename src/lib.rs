mod bytecodes;
mod codegen;
mod diagnostics;
mod disassembler;
mod expressions;
mod lexer;
mod parser;
pub mod program;
mod tokens;
pub mod vm;

use std::{fs::File, io::Read};

use diagnostics::Diagnostics;

pub fn read_file(path: &String) -> std::io::Result<String> {
    let mut file = File::open(path)?;
    let mut source = String::new();
    file.read_to_string(&mut source)?;
    Ok(source)
}

pub fn run_file(source_path: &String, debug: bool) -> Result<Option<vm::Value>, String> {
    let source = read_file(source_path);
    match source {
        Ok(source) => run(&source, Some(source_path), debug),
        Err(err) => Err(err.to_string()),
    }
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
pub fn run(
    source: &str,
    file_name: Option<&String>,
    debug: bool,
) -> Result<Option<vm::Value>, String> {
    let mut diagnostics = Diagnostics::new();

    let default_file_name = "n/a".to_string();
    println!(
        "\n# source (file: {}) =>",
        file_name.unwrap_or(&default_file_name)
    );

    println!("\n# lexing =>");
    let start = std::time::Instant::now();
    let tokens = lexer::lex(source);
    let duration = start.elapsed();
    println!("Elapsed: {:?}", duration);

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

    if debug {
        tokens.iter().for_each(|token| {
            println!(
                "token: {:?} at '{}' (line {}, column: {})",
                token.kind, token.lexeme, token.position.line, token.position.column
            )
        });
    }

    println!("\n# parsing =>");
    let start = std::time::Instant::now();
    let ast = parser::parse(tokens)?;
    let duration = start.elapsed();
    println!("Elapsed: {:?}", duration);
    if debug {
        println!("ast: {:?}", ast);
    }

    let context = program::Context::new();

    // TODO(anissen): Should use `program` w. diagnostics

    println!("\n# code gen =>");
    let start = std::time::Instant::now();
    let bytecodes = match codegen::codegen(&ast, &context) {
        Ok(bytecodes) => bytecodes,
        Err(diagnostics) => {
            eprintln!("Errors: {:?}", diagnostics);
            return Err("errors".to_string()); // TODO(anissen): Should return diagnostics
        }
    };

    let duration = start.elapsed();
    println!("Elapsed: {:?}", duration);

    if debug {
        println!("byte code length: {}", bytecodes.len());
        println!("byte codes: {:?}", bytecodes);
    }

    if debug {
        println!("\n# disassembly =>");
        disassembler::disassemble(bytecodes.clone());
    }

    println!("\n# vm =>");
    let result = vm::run(bytecodes, &context, debug);

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
