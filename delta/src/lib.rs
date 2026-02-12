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

#[derive(Debug, Clone, Default)]
pub struct ProgramMetadata {
    pub compilation_metadata: CompilationMetadata,
    pub execution_metadata: ExecutionMetadata,
}

#[derive(Debug, Clone, Default)]
pub struct CompilationMetadata {
    pub bytecode: Vec<u8>,
    pub bytecode_length: usize,
    pub disassembled_instructions: String,
}

#[derive(Debug, Clone, Default)]
pub struct ExecutionMetadata {
    pub instructions_executed: usize,
    pub jumps_performed: usize,
    pub bytes_read: usize,
    pub stack_allocations: usize,
    pub max_stack_height: usize,
}

#[derive(Debug, Clone)]
pub struct ProgramResult {
    pub value: Option<vm::Value>,
    pub metadata: ProgramMetadata,
}

pub struct DeltaArguments {
    pub source_path: String,
    pub no_run: bool,
    pub debug: bool,
}

pub fn read_file(path: &String) -> std::io::Result<String> {
    let mut file = File::open(path)?;
    let mut source = String::new();
    file.read_to_string(&mut source)?;
    Ok(source)
}

pub fn run_file(args: &DeltaArguments) -> Result<ProgramResult, Diagnostics> {
    let source = read_file(&args.source_path);
    match source {
        Ok(source) => run(&source, args),
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

pub fn run(source: &str, args: &DeltaArguments) -> Result<ProgramResult, Diagnostics> {
    println!("\n# source (file: {}) =>", &args.source_path);

    let context = program::Context::new();
    let mut program = Program::new(context, args);
    let result = program.reload(source.to_string());
    match result {
        None => {
            println!("\n# vm =>");
            let value = if !args.no_run { program.run() } else { None };
            Ok(ProgramResult {
                value,
                metadata: program.metadata,
            })
        }
        Some(diagnostics) => Err(diagnostics),
    }
}
