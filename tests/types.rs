pub mod common;

use common::{assert_type_fail, assert_type_ok};

#[test]
fn plus_integer() {
    assert_type_ok("1 + 2");

    assert_type_fail("1 + 2.4", "Expected integer but found float".to_string());
    assert_type_fail("1.2 + 2", "Expected integer but found float".to_string());
}

#[test]
fn plus_float() {
    assert_type_ok("1.2 +. 3.4");

    assert_type_fail("1 +. 2.4", "Expected float but found integer".to_string());
    assert_type_fail("1.2 +. 2", "Expected float but found integer".to_string());
}
