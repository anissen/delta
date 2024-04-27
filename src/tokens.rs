
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Integer,
    Float,
    Bang,
    Plus,
    Minus,
    Star,
    Slash,
    Space,
    SyntaxError,
    NewLine,
    EOF,
}

#[derive(Debug, Clone)]
pub struct Span {
    pub line: usize,
    pub column: usize,
    // start and end *can* be extracted from the lexeme
    // start: usize,
    // length: usize,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub position: Span,
    pub lexeme: String, // TODO(anissen): Should probably be &'a str,
}