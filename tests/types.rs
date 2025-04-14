pub mod common;

use common::{assert_type_fail, assert_type_ok};
// use delta::program::Context;
// use delta::lexer;
// use delta::parser;
// use delta::typer;

// fn type_ok(script: &str) {
//     let context = program::Context::new();
//     let tokens = lexer::lex(source);
// match    parser::parse(tokens) {
//     Ok(ast) => match typer::type_check(&ast, &context) {
//         Ok(typed_ast) => assert(true);
//         Err(diagnostics) => assert(false, diagnostics.to_string());
//     }
//     Err(diagnostics) => assert(false, diagnostics.to_string());
// }
// }

#[test]
fn plus_integer() {
    assert_type_fail("1 + 2.4", "asdf".to_string());
    assert_type_fail("1.2 + 2", "asdf2".to_string());
}

#[test]
fn plus_float() {
    assert_type_ok("1.2 +. 3.4");
    assert_type_fail("1 +. 2.4", "asdf".to_string());
    assert_type_fail("1.2 +. 2", "asdf2".to_string());
}

// TODO(anissen): Make helper methods for typing w. success/failure
