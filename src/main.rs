mod lexer;
mod parser;

fn main() {
    let source = "2+1.3 xyz 4";
    println!("{}", source);
    println!();

    let tokens = lexer::lex(source);
    println!("lexing =>");
    // tokens.iter().for_each(|f| println!("tokens: {:?}", f));
    tokens.iter().for_each(|f| match f.kind {
        lexer::TokenKind::SyntaxError => {
            println!(
                "syntax error at '{}' (line {}, column {})",
                f.lexeme, f.position.line, f.position.column,
            )
        }
        _ => println!("tokens: {:?}", f.kind),
    });
    println!();

    println!("parsing =>");
    let ast = parser::parse(tokens);
    println!("tokens: {:?}", ast);
}
