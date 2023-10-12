mod lexer;

fn main() {
    let source = "2+1.3 xyz 4";
    println!("{}", source);

    let tokens = lexer::lex(source);
    println!("=>");
    // tokens.iter().for_each(|f| println!("tokens: {:?}", f));
    tokens.iter().for_each(|f| {
        match f.kind {
            lexer::TokenKind::SyntaxError => {
                println!(
                    "syntax error at '{}' (line {}, column {})",
                    f.lexeme, f.position.line, f.position.column,
                )
            }
            _ => println!("tokens: {:?}", f.kind),
        }
        // println!("tokens: {:?}", f)
    });
}
