mod bytecodes;
mod codegen;
pub mod diagnostics;
mod disassembler;
mod errors;
mod expressions;
mod lexer;
mod parser;
pub mod program;
mod tokens;
mod typer;
mod unification;
pub mod vm;

use std::{fs::File, io::Read};

use diagnostics::Diagnostics;
use program::Program;

pub fn read_file(path: &String) -> std::io::Result<String> {
    let mut file = File::open(path)?;
    let mut source = String::new();
    file.read_to_string(&mut source)?;
    Ok(source)
}

pub fn run_file(source_path: &String, debug: bool) -> Result<Option<vm::Value>, Diagnostics> {
    let source = read_file(source_path);
    match source {
        Ok(source) => run(&source, Some(source_path), debug),
        Err(err) => {
            let mut diagnostics = Diagnostics::new();
            diagnostics.add_error(errors::Error::FileErr(err.to_string()));
            Err(diagnostics)
        }
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

pub fn build(
    source: &str,
    file_name: Option<&String>,
    debug: bool,
) -> Result<Vec<u8>, Diagnostics> {
    let default_file_name = "n/a".to_string();
    println!(
        "\n# source (file: {}) =>",
        file_name.unwrap_or(&default_file_name)
    );

    let context = program::Context::new();
    let program = Program::new(context);
    program.compile(source, debug)
}

// TODO(anissen): Make a concept of diagnostics (containing just syntax error for now)
pub fn run(
    source: &str,
    file_name: Option<&String>,
    debug: bool,
) -> Result<Option<vm::Value>, Diagnostics> {
    let default_file_name = "n/a".to_string();
    println!(
        "\n# source (file: {}) =>",
        file_name.unwrap_or(&default_file_name)
    );

    let context = program::Context::new();
    let program = Program::new(context);
    match program.compile(source, debug) {
        Ok(bytecodes) => {
            if debug {
                println!("\n# disassembly =>");
                disassembler::disassemble(bytecodes.clone());
            }

            println!("\n# vm =>");
            let result = program.run(bytecodes, debug);
            Ok(result)
        }

        Err(diagnostics) => Err(diagnostics),
    }
}
