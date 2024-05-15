mod bytecodes;
mod codegen;
mod expressions;
mod lexer;
mod parser;
mod tokens;
mod vm;

fn main() {
    let source = r"2-1.3 xyz 4
oh yeah
42";
    println!("# source =>");
    println!("{}", source);

    println!("# lexing =>");
    let tokens = lexer::lex(&source);
    tokens.iter().for_each(|f| match f.kind {
        tokens::TokenKind::SyntaxError => {
            println!(
                "! syntax error at '{}' (line {}, column {})",
                f.lexeme, f.position.line, f.position.column,
            )
        }
        _ => println!("token: {:?}", f.kind),
    });
    println!();

    println!("# parsing =>");
    let ast = parser::parse(tokens);
    println!("tokens: {:?}", ast);

    println!("# code gen =>");
    let bytecodes = codegen::codegen(ast);
    println!("byte codes: {:?}", bytecodes);

    println!("# vm =>");
    vm::run(bytecodes);
}
