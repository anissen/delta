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
fn integer_plus() {
    assert_ok(r"1 + 2 + 3 + 4 + 5", vm::Value::Integer(15));
}

#[test]
fn float_plus() {
    assert_ok(r"1.1 + 2.2 + 3.3 + 4.4 + 5.5", vm::Value::Float(16.5));
}
