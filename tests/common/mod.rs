pub fn assert_ok(source: &str, expected: delta::vm::Value) {
    let result = match delta::run(&source.to_string(), None) {
        Ok(Some(r)) => r,
        _ => panic!(),
    };
    assert!(
        result == expected,
        "Expected to succeed with {:?} but was {:?}",
        expected,
        result
    );
}
