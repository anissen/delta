use std::{fs::File, io::Read, path::Path, process::exit};

mod bytecodes;
mod codegen;
mod disassembler;
mod expressions;
mod lexer;
mod parser;
mod tokens;
mod vm;

// https://github.com/brightly-salty/rox/

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        println!("No source file argument provided.");
        exit(1);
    }

    let source_path = &args[1];

    // let path = Path::new("..").join("examples").join("workbench.∆");
    // let source_path = path.to_str().unwrap().to_string();

    let result = run(source_path);
    match result {
        Ok(Some(value)) => println!("Result: {:?}", value),

        Ok(None) => println!("Result: N/A"),

        Err(err) => println!("Error(s) occured:\n{}", err),
    }
}

fn run(source_path: &String) -> Result<Option<vm::Value>, String> {
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

    println!("\n# disassembly =>");
    let disassembled = disassembler::disassemble(bytecodes.clone());
    println!("disassembled:");
    for ele in disassembled {
        println!("{:?}", ele);
    }
    // println!("disassembled: {:?}", disassembled);

    println!("\n# vm =>");
    let result = vm::run(bytecodes);

    Ok(result)
}
