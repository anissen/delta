use crate::tokens::{Span, Token, TokenKind};
use std::char;
use vec;

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

    fn scan_tokens(&mut self, source: &'a str) -> Vec<Token> {
        self.source = source.chars().collect();

        let mut tokens = vec![];

        while !self.is_at_end() {
            self.start = self.current;
            let kind = self.scan_next();
            let position = Span {
                line: self.line,
                column: self.column,
                // start: self.start,
                // length: self.current - self.start,
            };
            let is_new_line = kind == TokenKind::NewLine;
            let token = Token {
                kind,
                position,
                lexeme: source[self.start..self.current].to_string(),
            };
            tokens.push(token);
            if is_new_line {
                self.line += 1;
                self.column = 0;
            } else {
                self.column += self.current - self.start;
            }
        }
        // TODO(anissen): EOF could probably be handled more gracefully
        tokens.push(Token {
            kind: TokenKind::EOF,
            position: Span {
                line: self.line,
                column: self.column,
            },
            lexeme: "".to_string(),
        });
        tokens
    }

    fn scan_next(&mut self) -> TokenKind {
        let char = self.advance();
        match char {
            ' ' => TokenKind::Space,
            '+' => TokenKind::Plus,
            '-' => TokenKind::Minus,
            '*' => TokenKind::Star,
            '/' => TokenKind::Slash,
            '!' => TokenKind::Bang,
            '#' => self.comment(),
            '\t' => TokenKind::Tab,
            '\n' => TokenKind::NewLine,
            c if self.is_digit(c) => self.number(),
            _ => TokenKind::SyntaxError,
        }
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
            TokenKind::Float
        } else {
            TokenKind::Integer
        }
    }

    fn comment(&mut self) -> TokenKind {
        while !self.is_at_end() && self.peek() != '\n' {
            self.advance();
        }
        TokenKind::Comment
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
