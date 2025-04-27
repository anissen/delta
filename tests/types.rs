pub mod common;

use common::{assert_type_fail, assert_type_ok};

#[test]
fn plus_integer() {
    assert_type_ok("1 + 2");

    assert_type_fail("1 + 2.4", "Expected int but found float".to_string());
    assert_type_fail("1.2 + 2", "Expected int but found float".to_string());
}

#[test]
fn plus_float() {
    assert_type_ok("1.2 +. 3.4");

    assert_type_fail("1 +. 2.4", "Expected float but found int".to_string());
    assert_type_fail("1.2 +. 2", "Expected float but found int".to_string());
}

// call = \f x y
// 	x | f y

// add = \x y
// 	x + y

// mult = \x y
// 	x * y

// add | call 4 5
// mult | call 4 5

// concat = \x y
// 	"{x}{y}"

// concat | call "hello world"

// mult_float = \x y
// 	x *. y

// mult_float | call 4.1 5.2
