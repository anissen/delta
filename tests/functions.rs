mod common;

use common::assert_ok;
use delta::vm;

#[test]
fn function_calling() {
    assert_ok(
        r"
add = \v1 v2
    v1 + v2

5 | add 3",
        vm::Value::Integer(8),
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
        vm::Value::Integer(6),
    )
}

#[test]
fn single_chained_function_calling() {
    assert_ok(
        r"
add_one = \v
    v + 1

5 | add_one | add_one",
        vm::Value::Integer(7),
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
        vm::Value::Integer(29),
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
        vm::Value::Integer(6),
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
        vm::Value::Integer(6),
    )
}

#[test]
fn comparison_function() {
    assert_ok(
        r"
is_5 = \v
    v == 5

5 | is_5",
        vm::Value::True,
    )
}

// #[test]
// fn indexes_are_off() {
//     assert_ok(
//         r"
// add = \v1 v2
//     v1 + v2

// is_even = \v3
//     v3 % 2 == 0

// is_odd = \v4
//     res = v4 | is_even
//     !res

// 5 | is_odd",
//         vm::Value::True,
//     )
// }
