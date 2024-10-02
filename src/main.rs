use std::{fs::File, io::Read, process::exit};

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

    let result = run_file(source_path);
    match result {
        Ok(Some(value)) => println!("Result: {:?}", value),

        Ok(None) => println!("Result: N/A"),

        Err(err) => println!("Error(s) occured:\n{}", err),
    }
}

fn run_file(source_path: &String) -> Result<Option<vm::Value>, String> {
    let mut file = File::open(source_path).expect("Unable to open file");
    let mut source = String::new();
    file.read_to_string(&mut source)
        .expect("Error reading file.");

    run(&source, Some(source_path))
}

fn run(source: &String, file_name: Option<&String>) -> Result<Option<vm::Value>, String> {
    let default_file_name = "n/a".to_string();
    println!(
        "\n# source (file: {}) =>",
        file_name.unwrap_or(&default_file_name)
    );
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

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_ok(source: &str, expected: vm::Value) {
        let result = match run(&source.to_string(), None) {
            Ok(Some(r)) => r,
            _ => panic!(),
        };
        assert!(
            result == expected,
            "Expected to succeed with {:?} but was {:?}",
            expected,
            result
        );
    }

    #[test]
    fn empty() {
        let source = r"# nothing here";
        let result = match run(&source.to_string(), None) {
            Ok(None) => true,
            _ => false,
        };
        assert!(result);
    }

    #[test]
    fn integer_plus() {
        assert_ok(r"1 + 2 + 3 + 4 + 5", vm::Value::Integer(15));
    }

    #[test]
    fn float_plus() {
        assert_ok(r"1.1 + 2.2 + 3.3 + 4.4 + 5.5", vm::Value::Float(16.5));
    }

    #[test]
    fn function_calling() {
        assert_ok(
            r"
add = \v1 v2
    v1 + v2

5 | add 3",
            vm::Value::Integer(8),
        );
    }

    #[test]
    fn nested_function_calling() {
        assert_ok(
            r"
add = \v1 v2
    v1 + v2

add_one = \v
    v | add 1

5 | add_one",
            vm::Value::Integer(6),
        )
    }
}
