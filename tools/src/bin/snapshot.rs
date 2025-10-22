use std::fs;
use std::path::Path;
use toml::{Table, Value};
use walkdir::WalkDir;

enum ProcessStatus {
    Processed,
    Ignored(String),
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = std::env::current_dir()?;
    let tests_dir = current_dir.parent().unwrap().join("snapshots");

    let mut files_processed = 0;
    let mut ignored_files = Vec::new();

    // Walk through all files in the current directory and subdirectories
    for entry in WalkDir::new(&tests_dir) {
        let entry = entry?;
        let path = entry.path();

        // Skip if not a .toml file
        if path.extension().is_none_or(|ext| ext != "toml") {
            continue;
        }

        let process_reuslt = process_toml_file(path)?;
        match process_reuslt {
            ProcessStatus::Processed => files_processed += 1,
            ProcessStatus::Ignored(reason) => {
                ignored_files.push((path.display().to_string(), reason))
            }
        };
    }

    if !ignored_files.is_empty() {
        println!("Ignored {} file(s)", ignored_files.len());
        for (ref path, reason) in ignored_files {
            println!(
                "  => Ignored {}: {}",
                Path::strip_prefix(Path::new(path), &tests_dir)
                    .unwrap()
                    .display(),
                reason
            );
        }
        println!();
    }

    println!(
        "Successfully processed {} TOML files in {}",
        files_processed,
        tests_dir.display()
    );

    Ok(())
}

fn process_toml_file(path: &Path) -> Result<ProcessStatus, Box<dyn std::error::Error>> {
    // Read the file
    let content = fs::read_to_string(path)?;

    // Parse as TOML
    let mut doc: Table = content.parse()?;

    // Get "script" value from doc
    let script = doc
        .get("script")
        .and_then(|v| v.as_str())
        .unwrap()
        .to_string();

    if let Some(ignored) = doc.get("ignored") {
        return Ok(ProcessStatus::Ignored(ignored.to_string()));
    }

    let file_name = path.file_name().unwrap().display().to_string();

    let result = run_script(file_name.clone(), &script);

    // Always create/replace the output section with a new empty table
    doc.insert("output".to_string(), Value::Table(Table::new()));
    let output_section = doc.get_mut("output").unwrap();

    if let Value::Table(table) = output_section {
        match result {
            Ok(program_result) => {
                let (result, result_type) = match program_result.value {
                    Some(value) => {
                        let result_type = match value {
                            delta::vm::Value::True => "boolean".to_string(),
                            delta::vm::Value::False => "boolean".to_string(),
                            delta::vm::Value::Integer(_) => "integer".to_string(),
                            delta::vm::Value::Float(_) => "float".to_string(),
                            delta::vm::Value::String(_) => "string".to_string(),
                            delta::vm::Value::SimpleTag { .. } => "tag".to_string(),
                            delta::vm::Value::Tag { .. } => "tag".to_string(),
                            delta::vm::Value::Function(_) => "function".to_string(),
                            delta::vm::Value::List(_) => "list".to_string(),
                        };
                        (value.to_string(), result_type)
                    }
                    None => ("N/A".to_string(), "None".to_string()),
                };
                table.insert("result".to_string(), Value::String(result));
                table.insert("type".to_string(), Value::String(result_type));

                let compilation_metadata = program_result.metadata.compilation_metadata;
                let execution_metadata = program_result.metadata.execution_metadata;

                // Add compiler metadata
                let mut compiler_table = Table::new();
                compiler_table.insert(
                    "bytecode_length".to_string(),
                    Value::Integer(compilation_metadata.bytecode_length as i64),
                );
                compiler_table.insert(
                    "bytecode".to_string(),
                    Value::String(format!("{:?}", compilation_metadata.bytecode)),
                );
                compiler_table.insert(
                    "disassembled".to_string(),
                    Value::String(compilation_metadata.disassembled_instructions),
                );
                table.insert("compiler".to_string(), Value::Table(compiler_table));

                // Add VM metadata
                let mut vm_table = Table::new();
                vm_table.insert(
                    "instructions_executed".to_string(),
                    Value::Integer(execution_metadata.instructions_executed as i64),
                );
                vm_table.insert(
                    "jumps_performed".to_string(),
                    Value::Integer(execution_metadata.jumps_performed as i64),
                );
                vm_table.insert(
                    "bytes_read".to_string(),
                    Value::Integer(execution_metadata.bytes_read as i64),
                );
                vm_table.insert(
                    "stack_allocations".to_string(),
                    Value::Integer(execution_metadata.stack_allocations as i64),
                );
                vm_table.insert(
                    "max_stack_height".to_string(),
                    Value::Integer(execution_metadata.max_stack_height as i64),
                );
                table.insert("vm".to_string(), Value::Table(vm_table));
            }
            Err(diagnostics) => {
                let errors = diagnostics.print(&script).join("\n\n");
                table.insert("error".to_string(), Value::String(errors));
            }
        }
    }

    // Convert back to TOML string
    let new_content = toml::to_string_pretty(&doc)?;

    // Write back to file
    fs::write(path, new_content)?;

    Ok(ProcessStatus::Processed)
}

fn run_script(
    file_name: String,
    source: &str,
) -> Result<delta::ProgramResult, delta::diagnostics::Diagnostics> {
    // Set a timeout?
    delta::run(source, Some(&file_name), true)
}
