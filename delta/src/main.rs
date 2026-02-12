use std::process::exit;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        println!("No source file argument provided.");
        exit(1);
    }

    let delta_args = delta::DeltaArguments {
        source_path: args[1].clone(),
        debug: args.contains(&"--debug".to_string()),
        no_run: args.contains(&"--no-run".to_string()),
    };
    let result = delta::run_file(&delta_args);
    match result {
        Ok(program_result) => match program_result.value {
            Some(value) => {
                println!("\nResult: {value:?}");
            }
            None => {
                println!("\nResult: N/A");
            }
        },
        Err(diagnostics) => {
            println!();
            let source = delta::read_file(&delta_args.source_path);
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
