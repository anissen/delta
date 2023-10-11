mod lexer;

fn main() {
    let source = "2+1.3 + 4";
    println!("{}", source);

    let tokens = lexer::lex(source);
    println!("=>");
    tokens.iter().for_each(|f| println!("tokens: {:?}", f));
}
