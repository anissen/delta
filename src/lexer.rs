use std::char;
use vec;

#[derive(Debug)]
pub enum TokenKind {
    Integer(i32),
    Float(f32),
    Plus,
    Star,
    SyntaxError,
    NewLine,
}

#[derive(Debug)]
pub struct Span {
    pub line: usize,
    pub column: usize,
    // start and end *can* be extracted from the lexeme
    // start: usize,
    // length: usize,
}

#[derive(Debug)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub position: Span,
    pub lexeme: &'a str,
}

struct Lexer {
    source: Vec<char>,
    start: usize,
    current: usize,
    line: usize,
    column: usize,
}

pub fn lex<'a>(source: &'a str) -> Vec<Token> {
    Lexer::new().scan_tokens(source)
}

impl<'a> Lexer {
    fn new() -> Self {
        Self {
            source: Vec::default(),
            start: 0,
            current: 0,
            line: 0,
            column: 0,
        }
    }

    fn scan_tokens(&mut self, source: &'a str) -> Vec<Token<'a>> {
        self.source = source.chars().collect();

        let mut tokens = vec![];

        while !self.is_at_end() {
            self.start = self.current;
            if let Some(kind) = self.scan_next() {
                let position = Span {
                    line: self.line,
                    column: self.column,
                    // start: self.start,
                    // length: self.current - self.start,
                };
                let token = Token {
                    kind,
                    position,
                    lexeme: &source[self.start..self.current],
                };
                tokens.push(token);
            }
            self.column += self.current - self.start;
        }
        tokens
    }

    fn scan_next(&mut self) -> Option<TokenKind> {
        let char = self.advance();
        let token = match char {
            ' ' => None,
            '+' => Some(TokenKind::Plus),
            '\n' => {
                self.line += 1;
                self.column = 0;
                Some(TokenKind::NewLine)
            }
            c if self.is_digit(c) => Some(self.number()),
            _ => Some(TokenKind::SyntaxError),
        };
        token
    }

    fn number(&mut self) -> TokenKind {
        while self.is_digit(self.peek()) {
            self.advance();
        }

        let is_float = self.peek() == '.' && self.is_digit(self.peek_next());
        if is_float {
            self.advance();
            while self.is_digit(self.peek()) {
                self.advance();
            }
        }

        let chars = self.source[self.start..self.current].to_vec();
        let str: String = chars.into_iter().collect(); // HACK: This could probably be done better
        if is_float {
            let value = str.parse::<f32>().unwrap();
            TokenKind::Float(value)
        } else {
            let value = str.parse::<i32>().unwrap();
            TokenKind::Integer(value)
        }
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            Default::default()
        } else {
            self.source[self.current]
        }
    }

    fn peek_next(&self) -> char {
        if self.current >= self.source.len() {
            Default::default()
        } else {
            self.source[self.current + 1]
        }
    }

    fn is_digit(&self, value: char) -> bool {
        value.is_digit(10)
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.source[self.current - 1]
    }
}
