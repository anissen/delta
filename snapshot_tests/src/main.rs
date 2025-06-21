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
    let tests_dir = current_dir.join("tests");

    let mut files_processed = 0;
    let mut ignored_files = Vec::new();

    // Walk through all files in the current directory and subdirectories
    for entry in WalkDir::new(&tests_dir) {
        let entry = entry?;
        let path = entry.path();

        // Skip if not a .toml file
        if !path.extension().map_or(false, |ext| ext == "toml") {
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
    let script = doc.get("script").and_then(|v| v.as_str()).unwrap();

    if let Some(ignored) = doc.get("ignored") {
        return Ok(ProcessStatus::Ignored(ignored.to_string()));
    }

    let file_name = path.file_name().unwrap().display().to_string();

    let result = run_script(file_name, script);

    let output_section = doc
        .entry("output".to_string())
        .or_insert_with(|| Value::Table(Table::new()));

    if let Value::Table(table) = output_section {
        match result {
            Ok(output) => {
                let (result, result_type) = match output {
                    Some(value) => {
                        let result_type = match value {
                            delta::vm::Value::True => "boolean".to_string(),
                            delta::vm::Value::False => "boolean".to_string(),
                            delta::vm::Value::Integer(_) => "integer".to_string(),
                            delta::vm::Value::Float(_) => "float".to_string(),
                            delta::vm::Value::String(_) => "string".to_string(),
                            delta::vm::Value::Function(_) => "function".to_string(),
                        };
                        (value.to_string(), result_type)
                    }
                    None => ("N/A".to_string(), "None".to_string()),
                };
                table.insert("result".to_string(), Value::String(result));
                table.insert("type".to_string(), Value::String(result_type));
            }
            Err(err) => {
                table.insert("error".to_string(), Value::String(err.to_string()));
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
) -> Result<Option<delta::vm::Value>, delta::diagnostics::Diagnostics> {
    // Set a timeout?
    delta::run(source, Some(&file_name), false)
    // TODO(anissen): Also output bytecode length, bytecode instructions, instructions executed, bytes read, (allocations?)
}
