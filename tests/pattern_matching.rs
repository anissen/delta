mod common;

use common::{assert_err, assert_ok};
use delta::vm::Value;

#[test]
fn pattern_matching_integers() {
    assert_ok(
        r#"
2 is
    1
       	"no"
    2
       	"yes"
    3
       	"also no"
"#,
        Value::String("yes".to_string()),
    );
}

#[test]
fn nested_pattern_matching_integers() {
    assert_ok(
        r#"
res = 2 is
	4
		"nope"
	3
		"no"
	2
		true is
			true
				"oh yes"
			false
				"oh no"
	1
		"also no"

"result is '{res}'"
"#,
        Value::String("result is 'oh yes'".to_string()),
    );
}

#[test]
fn pattern_matching_with_functions() {
    assert_ok(
        r#"
square = \v
    v * v

match = \v
    v is
        2
            5 | square

"result is {2 | match}"
"#,
        Value::String("result is 25".to_string()),
    );
}

#[test]
fn pattern_matching_default() {
    assert_ok(
        r#"
3 is
    2
        "nope"
    _
        "yes"
"#,
        Value::String("yes".to_string()),
    );
}

#[test]
fn pattern_matching_expression() {
    assert_ok(
        r#"
3 is
    1 + 1 + 1
        "yes"
    _
        "no"
"#,
        Value::String("yes".to_string()),
    );
}

#[test]
fn expression_pattern_matching() {
    assert_ok(
        r#"
2 + 2 is
    4
        "yes"
    _
        "no"
"#,
        Value::String("yes".to_string()),
    );
}

#[test]
fn pattern_matching_capture() {
    assert_ok(
        r#"
2 is
    value
        "value captured is {value}"
"#,
        Value::String("value captured is 2".to_string()),
    );
}

#[test]
fn pattern_matching_capture_guard() {
    assert_ok(
        r#"
2 is
    1
        "no"
    other if other >= 2
        "value captured is {other}"
"#,
        Value::String("value captured is 2".to_string()),
    );
}

#[test]
fn pattern_matching_multiple_capture_guards() {
    assert_ok(
        r#"
2.3 is
    other if other <=. 2.0
        "nope"
    other if other >=. 2.0
        "value captured is {other}"
"#,
        Value::String("value captured is 2.3".to_string()),
    );
}

#[test]
fn pattern_matching_complex_capture_guard() {
    assert_ok(
        r#"
2.5 is
    other if other >=. 2.0 and other <. 3.0
        "captured {other}"
"#,
        Value::String("captured 2.5".to_string()),
    );
}

#[test]
#[ignore = "not yet implemented"]
fn pattern_matching_capture_non_boolean_guard() {
    assert_err(
        r#"
2 is
    1
        "no
    other if 2 + 2 # not a boolean expression
        "value captured is {other}"
"#,
        "Expected expression to be boolean".to_string(),
    );
}

#[test]
fn multiple_default_patterns() {
    assert_err(
        r#"
3 is
    _
        "ok"
    _
        "not okay"
"#,
        "An `is` block cannot have multiple default arms.".to_string(),
    );
}

#[test]
fn arm_after_default_pattern() {
    assert_err(
        r#"
3 is
    _
        "ok"
    3
        "not okay"
"#,
        "Unreachable due to default arm above.".to_string(),
    );
}

#[test]
fn pattern_matching_function_call() {
    assert_ok(
        r#"
add = \v1 v2
    v1 + v2

(2 | add 3) is
    5
       	"yes"
    _
        "no"
"#,
        Value::String("yes".to_string()),
    );
}

#[test]
fn pattern_matching_with_locals() {
    assert_ok(
        r#"
match = \v
    x = 3
    v + x is
        4
            y = 5
            x + y + v

"result is {1 | match}"
"#,
        Value::String("result is 9".to_string()),
    );
}

#[test]
fn pattern_matching_with_more_locals() {
    assert_ok(
        r#"
square = \v
    v * v

match = \v
    x = 2
    v + x is
       	3
            y = 4
            v + y
        4
           	z = v + x + 5
           	z | square
        5
           	w = 6
           	w2 = 7
           	x + w + w2

"result is {1 | match}, {2 | match} and {3 | match}"
"#,
        Value::String("result is 5, 81 and 15".to_string()),
    );
}

#[test]
fn pattern_matching_with_even_more_locals() {
    assert_ok(
        r#"
x = 1
y = 2

square = \v
    v * v

z = 3

match = \v
    x = 2
    res = v + x is
       	3
            y = 4
            v + y
        4
           	z = v + x + 5
           	z | square
        captured
           	w = 6
           	w2 = 7
           	captured + x + w + w2
    val = 8
    res + val

w = 4

"result is {1 | match}, {2 | match} and {3 | match}"
"#,
        Value::String("result is 13, 89 and 28".to_string()),
    );
}
