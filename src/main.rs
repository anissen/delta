use std::process::exit;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        println!("No source file argument provided.");
        exit(1);
    }

    let source_path = &args[1];
    let debug = args.contains(&"--debug".to_string());
    let result = delta::run_file(source_path, debug);
    match result {
        Ok(Some(value)) => println!("\nResult: {:?}", value),
        Ok(None) => println!("\nResult: N/A"),
        Err(diagnostics) => {
            let errors = diagnostics.get_errors();
            match &errors[..] {
                [err] => println!("\nError occured:\n{}", err),
                _ => {
                    println!("\nErrors occured:");
                    for ele in errors {
                        println!("{}", ele);
                    }
                }
            }
        }
    }
}
