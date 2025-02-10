use std::process::exit;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        println!("No source file argument provided.");
        exit(1);
    }

    let source_path = &args[1];
    let result = delta::run_file(source_path);
    match result {
        Ok(Some(value)) => println!("Result: {:?}", value),
        Ok(None) => println!("Result: N/A"),
        Err(err) => println!("Error(s) occured:\n{}", err),
    }
}
