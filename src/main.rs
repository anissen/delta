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
        Ok(Some(value)) => println!("\nResult: {value:?}"),
        Ok(None) => println!("\nResult: N/A"),
        Err(diagnostics) => {
            println!();
            let source = delta::read_file(source_path);
            for ele in diagnostics.print(&source.unwrap()) {
                println!("\x1b[31merror:\x1b[0m");
                println!("{ele}");
                println!();
                //                 println!(
                //                     "\x1b[31merror:\x1b[0m
                //    ┌─ {filePath + fileName}:{line}:{column}
                //    │
                // {line}  │   {error_line}
                //    │   {arrows}
                //    │
                // {hint}"
                // );
            }
        }
    }
}
