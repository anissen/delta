use std::fs;
use std::path::{Path, PathBuf};
use toml::{Table, Value};
use walkdir::WalkDir;

enum ProcessStatus {
    Processed,
    Ignored(String),
}

struct TestFile {
    path: PathBuf,
    script: String,
    previous_instructions: Option<usize>,
}

struct TestResult {
    path: PathBuf,
    status: ProcessStatus,
    instructions_diff: Option<InstructionsDiff>,
}

struct InstructionsDiff {
    previous: usize,
    current: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = std::env::current_dir()?;
    let tests_dir = current_dir.join("snapshots");

    // Phase 1: Collect all test files and their data
    println!("Collecting test files...");
    let mut test_files = Vec::new();

    for entry in WalkDir::new(&tests_dir) {
        let entry = entry?;
        let path = entry.path();

        // Skip if not a .toml file
        if path.extension().is_none_or(|ext| ext != "toml") {
            continue;
        }

        if let Some(test_file) = collect_test_data(path)? {
            test_files.push(test_file);
        }
    }

    println!("Found {} test file(s)\n", test_files.len());

    // Phase 2: Execute all tests and collect results
    println!("Executing tests...");
    let mut results = Vec::new();

    for test_file in test_files {
        let result = process_toml_file(&test_file)?;
        results.push(result);
    }

    println!("\n{}", "=".repeat(80));
    println!("Test Execution Report");
    println!("{}", "=".repeat(80));

    // Phase 3: Print comprehensive report
    let mut files_processed = 0;
    let mut ignored_files = Vec::new();
    let mut instruction_changes = Vec::new();

    for result in results {
        match result.status {
            ProcessStatus::Processed => {
                files_processed += 1;
                if let Some(diff) = result.instructions_diff {
                    if diff.previous != diff.current {
                        instruction_changes.push((result.path, diff));
                    }
                }
            }
            ProcessStatus::Ignored(reason) => {
                ignored_files.push((result.path, reason));
            }
        }
    }

    // Print instruction changes
    if !instruction_changes.is_empty() {
        println!(
            "\nInstruction Count Changes ({}):",
            instruction_changes.len()
        );
        println!("{}", "-".repeat(80));
        for (path, diff) in &instruction_changes {
            let rel_path = Path::strip_prefix(path, &tests_dir)
                .unwrap_or(path)
                .display();
            let change = diff.current as i64 - diff.previous as i64;
            let sign = if change > 0 { "+" } else { "" };
            println!(
                "  {} -> {} ({}{})",
                diff.previous, diff.current, sign, change
            );
            println!("    {}", rel_path);
        }
    }

    let ignored_file_count = ignored_files.len();
    // Print ignored files
    if !ignored_files.is_empty() {
        println!("\nIgnored Files ({}):", ignored_file_count);
        println!("{}", "-".repeat(80));
        for (path, reason) in ignored_files {
            let rel_path = Path::strip_prefix(&path, &tests_dir)
                .unwrap_or(&path)
                .display();
            println!("  {}", rel_path);
            println!("    Reason: {}", reason);
        }
    }

    // Print summary
    println!("\n{}", "=".repeat(80));
    println!("Summary:");
    println!("  Processed: {}", files_processed);
    println!("  Ignored: {}", ignored_file_count);
    println!("  Instruction changes: {}", instruction_changes.len());
    println!("{}", "=".repeat(80));

    Ok(())
}

fn collect_test_data(path: &Path) -> Result<Option<TestFile>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let doc: Table = content.parse()?;

    // Check if file should be ignored
    if doc.get("ignored").is_some() {
        return Ok(None);
    }

    let script = doc
        .get("script")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'script' field")?;

    let previous_instructions = doc
        .get("output")
        .and_then(|v| v.as_table())
        .and_then(|t| t.get("vm"))
        .and_then(|v| v.as_table())
        .and_then(|t| t.get("instructions_executed"))
        .and_then(|v| v.as_integer())
        .map(|v| v as usize);

    Ok(Some(TestFile {
        path: path.to_path_buf(),
        script: script.to_string(),
        previous_instructions,
    }))
}

fn process_toml_file(test_file: &TestFile) -> Result<TestResult, Box<dyn std::error::Error>> {
    // Read the file again
    let content = fs::read_to_string(&test_file.path)?;
    let mut doc: Table = content.parse()?;

    if let Some(ignored) = doc.get("ignored") {
        return Ok(TestResult {
            path: test_file.path.clone(),
            status: ProcessStatus::Ignored(ignored.to_string()),
            instructions_diff: None,
        });
    }

    let file_name = test_file.path.file_name().unwrap().display().to_string();

    let result = run_script(
        file_name.clone(),
        &test_file.script,
        true, /* TODO: This should be false but that removes disassembly */
    );

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
                            delta::vm::Value::Component(_) => "component".to_string(),
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
                let current_instructions = execution_metadata.instructions_executed;
                vm_table.insert(
                    "instructions_executed".to_string(),
                    Value::Integer(current_instructions as i64),
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

                // Calculate instruction diff
                let instructions_diff =
                    test_file
                        .previous_instructions
                        .map(|prev| InstructionsDiff {
                            previous: prev,
                            current: current_instructions,
                        });

                // Convert back to TOML string
                let new_content = toml::to_string_pretty(&doc)?;

                // Write back to file
                fs::write(&test_file.path, new_content)?;

                return Ok(TestResult {
                    path: test_file.path.clone(),
                    status: ProcessStatus::Processed,
                    instructions_diff,
                });
            }
            Err(diagnostics) => {
                let errors = diagnostics.print(&test_file.script).join("\n\n");
                table.insert("error".to_string(), Value::String(errors));
            }
        }
    }

    // Convert back to TOML string
    let new_content = toml::to_string_pretty(&doc)?;

    // Write back to file
    fs::write(&test_file.path, new_content)?;

    Ok(TestResult {
        path: test_file.path.clone(),
        status: ProcessStatus::Processed,
        instructions_diff: None,
    })
}

fn run_script(
    file_name: String,
    source: &str,
    debug: bool,
) -> Result<delta::ProgramResult, delta::diagnostics::Diagnostics> {
    // Set a timeout?
    delta::run(source, Some(&file_name), debug)
}
