# ∆ delta

delta is a programming language and runtime for games and interactive applications. Currently in early concept stage.

## Overview

delta is designed with a focus on functional programming patterns, pattern matching, and expressive syntax for game development. The language features a pipeline-based function calling syntax, robust pattern matching with guards, and a type system optimized for interactive applications.

## Language Syntax and Semantics

### Basic Data Types

- **Integers**: `1`, `42`, `-5`
- **Floats**: `3.14`, `-0.5`
- **Booleans**: `true`, `false`
- **Strings**: `"Hello"`, `"World"`
- **Tags**: `:ok`, `:err "Something went wrong"`

### Arithmetic Operations

```delta
1 + 2 + 3          # Addition: 6
10 - 5             # Subtraction: 5
3 * 4              # Multiplication: 12
8 / 2              # Division: 4
7 % 3              # Modulo: 1
```

### Boolean Operations

```delta
true and false     # Logical AND: false
true or false      # Logical OR: true
not true           # Logical NOT: false
```

### Comparison Operations

Equality
```delta
5 == 5             # Equality: true
3 != 4             # Inequality: true
```

Integer comparisons
```delta
10 > 5             # Greater than: true
3 < 7              # Less than: true
5 >= 5             # Greater than or equal: true
2 <= 8             # Less than or equal: true
```

Float comparisons
```delta
10.3 >. 5.2        # Greater than: true
3.2 <. 7.4         # Less than: true
5.0 >=. 5.0        # Greater than or equal: true
2.1 <=. 8.7        # Less than or equal: true
```

### String Interpolation

delta supports string interpolation with expressions inside `{}` braces:

```delta
"Result: {40 + 2}" # "Result: 42"
```

### Function Definition

Functions are defined using the `\` (lambda) syntax:

```delta
add = \v1 v2
    v1 + v2

is_even = \v
    v % 2 == 0

greet = \name
    "Hello {name}!"
```

### Function Calling and Pipelines

Functions are called and composed using pipes.

```delta
3 | add 1 | is_even # Returns true (4 is even)
```

The result on the left-hand side of the pipe is passed as the first argument to the function on the right-hand side of the pipe. For instance, in `arg | func1 arg2 | func2`, `arg` is passed as the first argument to `func1` and the result of `func1` is passed as the only argument to `func2`.

### Tagged Values

delta supports tagged unions:

```delta
# Creating tagged values
:ok                 # Success without payload
:error "oh noes!"   # Error with payload
:the_answer 42      # Tag with an integer payload
```

### Pattern Matching

Pattern matching is performed using the `is` keyword:

```delta
# Basic pattern matching
2 is
		1
        "one"
    2
        "two"
    3
        "three"
    _
        "something else entirely"

# Pattern matching on tags
result = :ok "it worked!"
result is
    :ok "great success"
        "Hurray, great success!"
    :ok value
        "Success: {value}"
    :error
        "An error occurred"
    other
        "Unknown value: {other}"
```

### Guards in Pattern Matching

Patterns can include conditional guards using `if`:

```delta
number is
    1
        "exactly one"
    other if other >= 2
        "two or greater: {other}"
    other if other % 2 == 0
        "dunno, but it's even"
    _
    		"none of the above"
```

### Example: Complete Function

Here's a complete example showing multiple language features:

```delta
match = \tag
    tag is
        :container x
            "container with value {x}"
        other
            "other value: {other}"

(:container 4) | match
# Returns: "container with value 4"
```

## Running the Project

### Prerequisites

- Rust toolchain (latest stable)
- Cargo package manager

### Basic Execution

To run a delta program:

```bash
cargo run -- examples/workbench.∆
```

### Debug Mode

For detailed execution information including bytecode and VM statistics:

```bash
cargo run -- examples/workbench.∆ --debug
```

This will show:
- Compiled bytecode
- Disassembled instructions
- VM execution statistics (stack allocations, instructions executed, etc.)

### Running Other Examples

You can run any delta file by providing its path:

```bash
cargo run -- path/to/your/file.∆
cargo run -- path/to/your/file.∆ --debug
```

## Development and Testing

### Snapshot Testing

The project uses comprehensive snapshot testing to ensure language behavior consistency. The test suite is located in the `snapshots/tests/` directory and covers.

### Running Snapshot Tests

To run and update all snapshot tests:

```bash
cd snapshots
cargo run
```

### Test Structure

Each test is defined in a TOML file with this structure:

```toml
script = """
# delta code goes here
1 + 2 + 3
"""

[output]
result = "6"
type = "integer"

[output.compiler]
bytecode = "[...]"
bytecode_length = 42
disassembled = """
# Disassembled bytecode instructions
"""

[output.vm]
bytes_read = 43
instructions_executed = 5
jumps_performed = 0
max_stack_height = 2
stack_allocations = 6
```

### Test Categories

The test suite is organized into categories:

- `base/` - Basic language features (arithmetic, booleans, strings, comparison)
- `functions/` - Function definition, calling, and chaining
- `pattern_matching/` - Pattern matching with various complexity levels
- `tags/` - Tagged value creation and matching
- `types/` - Type system behavior
- `errors/` - Error handling and edge cases

## Project Structure

- `src/` - Main language implementation (compiler, VM, runtime)
- `examples/` - Example delta programs
- `snapshots/` - Test runner and snapshot test files
- `snapshots/tests/` - Comprehensive test suite organized by feature

## Language Philosophy

delta emphasizes:
- **Pipeline-first design**: Function composition through pipelines
- **Pattern matching**: Robust pattern matching with guards for control flow
- **Type safety**: Strong typing with inference
- **Expressiveness**: Concise syntax for common game development patterns
- **Performance**: Bytecode compilation for efficient execution

## Contributing

This project is in early development. The snapshot test suite provides a comprehensive specification of current language behavior and serves as both documentation and regression testing.

To contribute:
1. Add tests for new features in `snapshots/tests/`
2. Run `cd snapshots && cargo run` to validate changes
3. Ensure all existing tests continue to pass
