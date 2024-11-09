mod common;

use common::assert_ok;
use delta::vm;

#[test]
fn empty() {
    let source = r"# nothing here";
    let result = match delta::run(&source.to_string(), None) {
        Ok(None) => true,
        _ => false,
    };
    assert!(result);
}

#[test]
fn plus_integer() {
    assert_ok(r"1 + 2 + 3 + 4 + 5", vm::Value::Integer(15));
}

#[test]
fn plus_float() {
    assert_ok(r"1.1 + 2.2 + 3.3 + 4.4 + 5.5", vm::Value::Float(16.5));
}

#[test]
fn boolean_true() {
    assert_ok(r"true", vm::Value::True);
}

#[test]
fn boolean_false() {
    assert_ok(r"false", vm::Value::False);
}

#[test]
fn strings() {
    assert_ok(r#""""#, vm::Value::String("".to_string()));
    assert_ok(
        r#""hello world""#,
        vm::Value::String("hello world".to_string()),
    );
}

#[test]
fn modulo_integer() {
    assert_ok(r"4 % 2", vm::Value::Integer(0));
    assert_ok(r"5 % 2", vm::Value::Integer(1));
}

#[test]
fn modulo_float() {
    assert_ok(r"4.0 % 2.0", vm::Value::Float(0.0));
    assert_ok(r"5.4 % 2.1", vm::Value::Float(1.2000003)); // Float errors, curses!
}

#[test]
fn integer_division_by_zero() {
    assert_ok(
        r"
x = 42
y = 0
x / y",
        vm::Value::Integer(0),
    );
}

#[test]
fn float_division_by_zero() {
    assert_ok(
        r"
x = 42.3
y = 0.0
x / y",
        vm::Value::Float(0.0),
    );
}

#[test]
fn comparison_positive_integers() {
    assert_ok(r"0 == 0", vm::Value::True);
    assert_ok(r"1 == 1", vm::Value::True);
    assert_ok(r"5 == 5", vm::Value::True);

    assert_ok(r"3 == 5", vm::Value::False);
}

#[test]
fn comparison_negative_integers() {
    assert_ok(r"-0 == -0", vm::Value::True);
    assert_ok(r"-1 == -1", vm::Value::True);
    assert_ok(r"-5 == -5", vm::Value::True);

    assert_ok(r"-3 == -5", vm::Value::False);
}

#[test]
fn comparison_integer_expressions() {
    assert_ok(r"1 + 2 == 3", vm::Value::True);
    assert_ok(r"3 == 1 + 2", vm::Value::True);
    assert_ok(r"2 + 1 == 1 + 2", vm::Value::True);

    assert_ok(r"1 + 1 == 3", vm::Value::False);
}

#[test]
fn comparison_positive_floats() {
    assert_ok(r"0.0 == 0.0", vm::Value::True);
    assert_ok(r"5.4 == 5.4", vm::Value::True);
    assert_ok(r"123.456789 == 123.456789", vm::Value::True);

    assert_ok(r"12.3 == 12.0", vm::Value::False);
}

#[test]
fn comparison_negative_floats() {
    assert_ok(r"-0.0 == -0.0", vm::Value::True);
    assert_ok(r"-0.3 == -0.3", vm::Value::True);
    assert_ok(r"-5.4 == -5.4", vm::Value::True);
    assert_ok(r"-123.456789 == -123.456789", vm::Value::True);

    assert_ok(r"-12.3 == -12.0", vm::Value::False);
    assert_ok(r"-12.3 == 12.3", vm::Value::False);
}

#[test]
fn comparison_booleans() {
    assert_ok(r"true == true", vm::Value::True);
    assert_ok(r"false == false", vm::Value::True);

    assert_ok(r"true == false", vm::Value::False);
    assert_ok(r"false == true", vm::Value::False);
}

// #[test]
// fn mixed_division() {
//     assert_err(r"42.3 / 2", ...);
// }
