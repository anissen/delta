use std::char;
use vec;

#[derive(Debug)]
pub enum Token {
    Integer(i32),
    Float(f32),
    Plus,
    SyntaxError,
    NewLine,
    EOF,
}

struct Lexer {
    source: Vec<char>,
    start: usize,
    current: usize,
    line: usize,
    column: usize,
}

pub fn lex(source: &str) -> Vec<Token> {
    Lexer::new().scan_tokens(source)
}

impl Lexer {
    pub fn new() -> Self {
        Self {
            source: Vec::default(),
            start: 0,
            current: 0,
            line: 0,
            column: 0,
        }
    }

    pub fn scan_tokens(&mut self, source: &str) -> Vec<Token> {
        self.source = source.chars().collect();

        let mut tokens: Vec<Token> = vec![];

        while !self.is_at_end() {
            self.start = self.current;
            if let Some(token) = self.scan_token() {
                tokens.push(token); // TODO: Enrich tokens with line and column
            }
            self.column += self.current - self.start;
        }
        tokens.push(Token::EOF);
        tokens
    }

    fn scan_token(&mut self) -> Option<Token> {
        let char = self.advance();
        let token = match char {
            ' ' => None,
            '+' => Some(Token::Plus),
            '\n' => {
                self.line += 1;
                self.column = 0;
                Some(Token::NewLine)
            }
            c if self.is_digit(c) => Some(self.number()),
            _ => Some(Token::SyntaxError),
        };
        token
    }

    fn number(&mut self) -> Token {
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
            Token::Float(value)
        } else {
            let value = str.parse::<i32>().unwrap();
            Token::Integer(value)
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
