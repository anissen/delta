#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    BackSlash,
    Bang,
    BangEqual,
    Comment,
    Equal,
    EqualEqual,
    False,
    Float,
    Identifier,
    Integer,
    KeywordAnd,
    KeywordOr,
    KeywordIs,
    KeywordIf,
    LeftBrace,
    LeftParen,
    LeftChevron,
    LeftChevronDot,
    LeftChevronEqual,
    LeftChevronEqualDot,
    Minus,
    MinusDot,
    NewLine,
    Percent,
    PercentDot,
    Pipe,
    Plus,
    PlusDot,
    RightBrace,
    RightParen,
    RightChevron,
    RightChevronDot,
    RightChevronEqual,
    RightChevronEqualDot,
    Slash,
    SlashDot,
    Space,
    Star,
    StarDot,
    StringConcat,
    SyntaxError(&'static str),
    Tab,
    Text,
    True,
    Underscore,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    // pub start: u32,
    // pub end: u32,

    // start and end *can* be extracted from the lexeme
    // start: usize,
    // length: usize,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub position: Position,
    pub lexeme: String, // TODO(anissen): Should probably be &'a str,
}
