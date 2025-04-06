pub fn assert_ok(source: &str, expected: delta::vm::Value) {
    match delta::run(&source.to_string(), None, true) {
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

pub fn assert_err(source: &str, expected: String) {
    match delta::run(&source.to_string(), None, true) {
        Ok(Some(result)) => assert!(false, "Expected result to be Err but was Ok: {:?}", result),
        Err(diagnostics) => {
            assert!(diagnostics.count() == 1);
            let errs = diagnostics.get_errors();
            let err = errs.first().unwrap();
            assert!(
                err.content == expected,
                "Expected error to be '{:?}' but it was '{:?}'",
                expected,
                diagnostics
            )
        }
        _ => panic!(),
    };
}
