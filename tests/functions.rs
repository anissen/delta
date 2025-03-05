pub mod common;

use common::assert_ok;
use delta::vm::Value;

#[test]
fn function_calling() {
    assert_ok(
        r"
add = \v1 v2
    v1 + v2

5 | add 3",
        Value::Integer(8),
    );
}

#[test]
fn function_calling_parentheses() {
    assert_ok(
        r"
add = \v1 v2
    v1 + v2

(5 | add 3)",
        Value::Integer(8),
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
        Value::Integer(6),
    )
}

#[test]
fn single_chained_function_calling() {
    assert_ok(
        r"
add_one = \v
    v + 1

5 | add_one | add_one",
        Value::Integer(7),
    )
}

#[test]
fn multiple_chained_functions_calling() {
    assert_ok(
        r"
square = \v
    v * v

add = \v1 v2
    v1 + v2

add_one = \v
    v | add 1

5 | square | add 3 | add_one",
        Value::Integer(29),
    )
}

#[test]
fn temp_values_in_function() {
    assert_ok(
        r"
add_one = \v
    x = 1
    y = x
    v + y

5 | add_one",
        Value::Integer(6),
    )
}

#[test]
fn temp_value_in_function_call() {
    assert_ok(
        r"
add = \v1 v2
    v1 + v2

add_one = \v
    x = 1
    v | add x

5 | add_one",
        Value::Integer(6),
    )
}

#[test]
fn comparison_function() {
    assert_ok(
        r"
is_5 = \v
    v == 5

5 | is_5",
        Value::True,
    )
}

#[test]
fn string_function() {
    assert_ok(
        r"
is_5 = \v
    v == 5

5 | is_5",
        Value::True,
    )
}

#[test]
fn string_interpolation_function() {
    assert_ok(
        r#"
greeting = \name
    "Hello {name}!"

"John" | greeting
"#,
        Value::String("Hello John!".to_string()),
    )
}

#[test]
fn string_interpolation_function_call() {
    assert_ok(
        r#"
add = \v1 v2
    v1 + v2

"result is {40 | add 2}!"
"#,
        Value::String("result is 42!".to_string()),
    )
}

#[test]
fn local_variables_work() {
    assert_ok(
        r"
v = 2

is_even = \v
    v % 2 == 0

is_odd = \v
    res = v | is_even
    !res

5 | is_odd",
        Value::True,
    )
}

#[test]
fn chained_function_calling() {
    assert_ok(
        r"
add = \v1 v2
    v1 + v2

is_even = \v
    v % 2 == 0

3 | add 1 | is_even",
        Value::True,
    )
}
