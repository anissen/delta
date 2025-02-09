mod common;

use common::assert_ok;
use delta::vm;

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
        vm::Value::String("yes".to_string()),
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
        vm::Value::String("result is 'oh yes'".to_string()),
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
        vm::Value::String("result is 25".to_string()),
    );
}

