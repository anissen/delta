pub mod common;

use common::{assert_err, assert_ok};
use delta::vm::Value;

#[test]
#[ignore = "not yet implemented"]
fn empty() {
    assert_err(r"# nothing here", "missing return value".to_string());
}

#[test]
fn plus_integer() {
    assert_ok(r"1 + 2 + 3 + 4 + 5", Value::Integer(15));
}

#[test]
fn plus_float() {
    assert_ok(r"1.1 +. 2.2 +. 3.3 +. 4.4 +. 5.5", Value::Float(16.5));
}

#[test]
fn boolean_true() {
    assert_ok(r"true", Value::True);
}

#[test]
fn boolean_false() {
    assert_ok(r"false", Value::False);
}

#[test]
fn boolean_and() {
    assert_ok(r"true and true", Value::True);
    assert_ok(r"true and false", Value::False);
    assert_ok(r"false and true", Value::False);
    assert_ok(r"false and false", Value::False);
    assert_ok(r"true and true and true", Value::True);

    assert_ok(r"2 <= 3 and 2 > 1", Value::True);
}

#[test]
fn boolean_or() {
    assert_ok(r"true or true", Value::True);
    assert_ok(r"true or false", Value::True);
    assert_ok(r"false or true", Value::True);
    assert_ok(r"false or false", Value::False);
    assert_ok(r"false or true or false", Value::True);

    assert_ok(r"2 > 3 or 2 > 1", Value::True);
}

#[test]
fn boolean_mixed_logic() {
    assert_ok(r"false or (true and true)", Value::True);
    assert_ok(r"true and (false or true)", Value::True);
    assert_ok(r"true and (false and true)", Value::False);
    assert_ok(r"true and (false or false)", Value::False);
    assert_ok(r"(true or false) and (false or true)", Value::True);
    assert_ok(r"false or (true and (false or true))", Value::True);
}

#[test]
fn strings() {
    assert_ok(r#""""#, Value::String("".to_string()));
    assert_ok(r#""hello world""#, Value::String("hello world".to_string()));
}

#[test]
fn modulo_integer() {
    assert_ok(r"4 % 2", Value::Integer(0));
    assert_ok(r"5 % 2", Value::Integer(1));
}

#[test]
fn modulo_float() {
    assert_ok(r"4.0 %. 2.0", Value::Float(0.0));
    assert_ok(r"5.4 %. 2.1", Value::Float(1.2000003)); // Float errors, curses!
}

#[test]
fn division_by_zero() {
    assert_ok(r"10 / 0", Value::Integer(0));
    assert_ok(r"54.32 /. 0.0", Value::Float(0.0));
}

#[test]
fn integer_division_by_zero() {
    assert_ok(
        r"
x = 42
y = 0
x / y",
        Value::Integer(0),
    );
}

#[test]
fn float_division_by_zero() {
    assert_ok(
        r"
x = 42.3
y = 0.0
x /. y",
        Value::Float(0.0),
    );
}

#[test]
fn equality_positive_integers() {
    assert_ok(r"0 == 0", Value::True);
    assert_ok(r"1 == 1", Value::True);
    assert_ok(r"5 == 5", Value::True);

    assert_ok(r"3 == 5", Value::False);
}

#[test]
fn comparison_positive_integers() {
    assert_ok(r"0 < 1", Value::True);
    assert_ok(r"0 <= 0", Value::True);
    assert_ok(r"0 >= 0", Value::True);
    assert_ok(r"3 > 2", Value::True);
    assert_ok(r"5 >= 5", Value::True);
}

#[test]
fn inequality_positive_integers() {
    assert_ok(r"2 != 4", Value::True);
}

#[test]
fn equality_negative_integers() {
    assert_ok(r"-0 == -0", Value::True);
    assert_ok(r"-1 == -1", Value::True);
    assert_ok(r"-5 == -5", Value::True);

    assert_ok(r"-3 == -5", Value::False);
}

#[test]
fn comparison_negative_integers() {
    assert_ok(r"-0 > -1", Value::True);
    assert_ok(r"-0 <= -0", Value::True);
    assert_ok(r"-3 < -2", Value::True);
    assert_ok(r"-5 >= -5", Value::True);
}

#[test]
fn inequality_negative_integers() {
    assert_ok(r"-2 != -4", Value::True);
}

#[test]
fn equality_integer_expressions() {
    assert_ok(r"1 + 2 == 3", Value::True);
    assert_ok(r"3 == 1 + 2", Value::True);
    assert_ok(r"2 + 1 == 1 + 2", Value::True);

    assert_ok(r"1 + 1 == 3", Value::False);
}

#[test]
fn inequality_integer_expressions() {
    assert_ok(r"2 != 1 + 2", Value::True);
}

#[test]
fn equality_positive_floats() {
    assert_ok(r"0.0 ==. 0.0", Value::True);
    assert_ok(r"5.4 ==. 5.4", Value::True);
    assert_ok(r"123.456789 ==. 123.456789", Value::True);

    assert_ok(r"12.3 ==. 12.0", Value::False);
}

#[test]
fn comparison_negative_floats() {
    assert_ok(r"-0.9 >. -1.0", Value::True);
    assert_ok(r"-0.0 <=. -0.0", Value::True);
    assert_ok(r"-0.0 >=. -0.0", Value::True);
    assert_ok(r"-3.45 <. -2.34", Value::True);
    assert_ok(r"-5.67 >=. -5.67", Value::True);
}

#[test]
fn inequality_positive_floats() {
    assert_ok(r"2.3 !=. 5.4", Value::True);
}

#[test]
fn equality_negative_floats() {
    assert_ok(r"-0.0 ==. -0.0", Value::True);
    assert_ok(r"-0.3 ==. -0.3", Value::True);
    assert_ok(r"-5.4 ==. -5.4", Value::True);
    assert_ok(r"-123.456789 ==. -123.456789", Value::True);

    assert_ok(r"-12.3 ==. -12.0", Value::False);
    assert_ok(r"-12.3 ==. 12.3", Value::False);
}

#[test]
fn inequality_negative_floats() {
    assert_ok(r"-2.3 !=. -5.4", Value::True);
}

#[test]
fn equality_booleans() {
    assert_ok(r"true == true", Value::True);
    assert_ok(r"false == false", Value::True);

    assert_ok(r"true == false", Value::False);
    assert_ok(r"false == true", Value::False);
}

#[test]
fn inequality_booleans() {
    assert_ok(r"true != false", Value::True);
}

#[test]
fn equality_strings() {
    assert_ok(r#""" == """#, Value::True);
    assert_ok(r#""Hello!" == "Hello!""#, Value::True);

    assert_ok(r#""Hello" == "World""#, Value::False);
    assert_ok(r#""42" == 42"#, Value::False);
}

#[test]
fn inequality_strings() {
    assert_ok(r#""Hello" != "World""#, Value::True);
}

#[test]
fn string_interpolation() {
    assert_ok(r#""Hello {40 + 2}""#, Value::String("Hello 42".to_string()));
    assert_ok(
        r#""Hello {(40 + 2) / 2}""#,
        Value::String("Hello 21".to_string()),
    );
    assert_ok(
        r#""{2} * {2 + 1} == {2 * (2 + 1)}""#,
        Value::String("2 * 3 == 6".to_string()),
    );
    assert_ok(
        r#""float value is {0.1 +. 0.2}""#,
        Value::String("float value is 0.3".to_string()),
    );

    assert_ok(
        r#""result is {2 * 2 == 4}!""#,
        Value::String("result is true!".to_string()),
    );

    assert_ok(
        r#""result is {3 < 4} and {3.4 <. 4.5}!""#,
        Value::String("result is true and true!".to_string()),
    );
}

#[test]
#[ignore = "not yet implemented"]
fn mixed_division() {
    assert_err(r"42.3 /. 2", "incompatible types for division".to_string());
}

#[test]
fn undefined_variable() {
    assert_err("x", "Name not found in scope: x".to_string());
}
