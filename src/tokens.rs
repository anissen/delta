#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Bang,
    Comment,
    Equal,
    False,
    Float,
    Identifier,
    Integer,
    LeftParen,
    Minus,
    NewLine,
    Plus,
    RightParen,
    Slash,
    Space,
    Star,
    Tab,
    True,
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
