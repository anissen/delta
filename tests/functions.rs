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
fn repeated_function_calling() {
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

#[test]
#[ignore = "not implemented yet"]
fn calling_function_declared_later() {
    assert_ok(
        r"
call_square = \v
    v | square

square = \v
    v * v

5 | call_square",
        Value::Integer(25),
    )
}

#[test]
#[ignore = "not implemented yet"]
fn nested_function() {
    assert_ok(
        r#"
match = \v
	inner_add = \v2 v3
		v2 + v3
	v | inner_add 1

"result is {3 | match}"
"#,
        Value::String("result is 4".to_string()),
    )
}

#[test]
#[ignore = "not implemented yet"]
fn repeated_nested_function_call() {
    assert_ok(
        r#"
match = \v
	inner_add = \v2 v3
		v2 + v3
	res1 = v | inner_add 1
	unused = 1
	res1 + (v | inner_add 1)

"result is {3 | match}"
"#,
        Value::String("result is 4".to_string()),
    )
}

#[test]
#[ignore = "not implemented yet"]
fn recursive_function() {
    assert_ok(
        r"
rec = \v
	v is
		n if n <= 0
			n
		_
			v - 1 | rec

2 | rec",
        Value::Integer(0),
    )
}

#[test]
#[ignore = "not implemented yet"]
fn recursive_repeat_function() {
    assert_ok(
        r#"
repeat = \str times
	str | repeat_part "" times

repeat_part = \str acc tt
	tt is
		t if t <= 1
			acc
		_
			"{acc}{str}" | repeat_part str (tt - 1)

"hey" | repeat 3
"#,
        Value::String("heyheyhey".to_string()),
    )
}

#[test]
#[ignore = "not implemented yet"]
fn nested_recursive_repeat_function() {
    assert_ok(
        r#"
repeat = \str times
	repeat_part = \acc tt
		tt is
			t if t <= 1
				acc
			_
				"{acc}{str}" | repeat_part (tt - 1)
	str | repeat_part times

"yo" | repeat 3
"#,
        Value::String("yoyoyo".to_string()),
    )
}
