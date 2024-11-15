#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    BackSlash,
    Bang,
    Comment,
    Equal,
    EqualEqual,
    False,
    Float,
    Identifier,
    Integer,
    LeftBrace,
    LeftParen,
    Minus,
    NewLine,
    Percent,
    Pipe,
    Plus,
    RightBrace,
    RightParen,
    Slash,
    Space,
    Star,
    String,
    StringConcat,
    // SyntaxError,
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
