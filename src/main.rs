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
        Ok(program_result) => {
            match program_result.value {
                Some(value) => {
                    println!("\nResult: {value:?}");
                }
                None => {
                    println!("\nResult: N/A");
                }
            }
            println!("\nExecution Metadata:");
            println!(
                "  Bytecode length: {}",
                program_result.metadata.bytecode_length
            );
            println!(
                "  Instructions executed: {}",
                program_result.metadata.instructions_executed
            );
            println!(
                "  Jumps performed: {}",
                program_result.metadata.jumps_performed
            );
            println!("  Bytes read: {}", program_result.metadata.bytes_read);
            println!(
                "  Stack allocations: {}",
                program_result.metadata.stack_allocations
            );
            println!(
                "  Max stack height: {}",
                program_result.metadata.max_stack_height
            );
            if debug {
                println!(
                    "  Disassembled instructions:\n{}",
                    program_result.metadata.disassembled_instructions
                );
            }
        }
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
