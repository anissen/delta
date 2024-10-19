pub fn assert_ok(source: &str, expected: delta::vm::Value) {
    match delta::run(&source.to_string(), None) {
        Ok(Some(result)) => {
            assert!(
                result == expected,
                "Expected to succeed with {:?} but was {:?}",
                expected,
                result
            );
        }
        err => assert!(false, "Expected result to be Ok but was Err: {:?}", err),
    };
}
