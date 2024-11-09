use crate::tokens::{Span, Token, TokenKind};

struct Lexer {
    source: Vec<char>, // TODO(anissen): Should this be a `str`?
    start: usize,
    current: usize,
    line: usize,
    column: usize,
}

#[derive(Debug, Clone)]
pub struct Error {
    pub position: Span,
    pub lexeme: String, // TODO(anissen): Should probably be &'a str,
}

type Errors = Vec<Error>;

// TODO(anissen): Ideally, I would like to return `Result<Vec<Token>, Errors>`
// and have the caller handle it gracefully, but I can't figure out how.
pub fn lex(source: &str) -> Result<Vec<Token>, String> {
    match Lexer::new().scan_tokens(source) {
        Ok(tokens) => Ok(tokens),

        Err(errors) => Err(errors
            .into_iter()
            .map(|err| {
                format!(
                    "! syntax error at '{}' (line {}, column {})",
                    err.lexeme, err.position.line, err.position.column,
                )
            })
            .collect::<Vec<String>>()
            .join("\n")),
    }
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

    fn scan_tokens(&mut self, source: &'a str) -> Result<Vec<Token>, Errors> {
        self.source = source.chars().collect();

        let mut errors = vec![];
        let mut tokens = vec![];

        while !self.is_at_end() {
            self.start = self.current;
            let result = self.scan_next();
            let position = Span {
                line: self.line,
                column: self.column,
                // start: self.start,
                // length: self.current - self.start,
            };
            let lexeme = match result {
                Ok(TokenKind::String) => source[self.start + 1..self.current - 1].to_string(), // drop quotes
                _ => source[self.start..self.current].to_string(),
            };

            match result {
                Ok(kind) => {
                    let is_new_line = kind == TokenKind::NewLine;
                    let token = Token {
                        kind,
                        position,
                        lexeme,
                    };
                    tokens.push(token);
                    if is_new_line {
                        self.line += 1;
                        self.column = 0;
                    } else {
                        self.column += self.current - self.start;
                    }
                }

                Err(_) => errors.push(Error { position, lexeme }),
            }
        }

        if !errors.is_empty() {
            Err(errors)
        } else {
            Ok(tokens)
        }
    }

    fn scan_next(&mut self) -> Result<TokenKind, String> {
        let char = self.advance();
        match char {
            ' ' => self.spaces(),
            '+' => Ok(TokenKind::Plus),
            '-' if self.is_digit(self.peek()) => Ok(self.number()),
            '-' => Ok(TokenKind::Minus),
            '*' => Ok(TokenKind::Star),
            '/' => Ok(TokenKind::Slash),
            '%' => Ok(TokenKind::Percent),
            '\\' => Ok(TokenKind::BackSlash),
            '!' => Ok(TokenKind::Bang),
            '=' if self.matches('=') => Ok(TokenKind::EqualEqual),
            '=' => Ok(TokenKind::Equal),
            '#' => Ok(self.comment()),
            '|' => Ok(TokenKind::Pipe),
            '(' => Ok(TokenKind::LeftParen),
            ')' => Ok(TokenKind::RightParen),
            '\t' => Ok(TokenKind::Tab),
            '\n' => Ok(TokenKind::NewLine),
            '\"' => self.string(),
            c if self.is_letter(c) => Ok(self.identifier()),
            c if self.is_digit(c) => Ok(self.number()),
            _ => Err(format!("Unexpected token: {}", char)),
        }
    }

    fn spaces(&mut self) -> Result<TokenKind, String> {
        let mut spaces = 1;
        while !self.is_at_end() && self.peek() == ' ' {
            self.advance();
            spaces += 1;
        }
        match spaces {
            1 => Ok(TokenKind::Space),
            4 => Ok(TokenKind::Tab), // HACK because Zed cannot handle hard tabs correctly. Scanning for '\t' should be sufficient.
            _ => Err("Unexpected whitespace".to_string()),
        }
    }

    fn identifier(&mut self) -> TokenKind {
        while !self.is_at_end()
            && (self.is_letter(self.peek()) || self.is_digit(self.peek()) || self.peek() == '_')
        {
            self.advance();
        }

        let lexeme = self.source[self.start..self.current]
            .iter()
            .collect::<String>();
        match lexeme.as_str() {
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            _ => TokenKind::Identifier,
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

    fn string(&mut self) -> Result<TokenKind, String> {
        loop {
            if self.is_at_end() {
                return Err("Unclosed string literal".to_string());
            } else if self.peek() == '\n' {
                return Err("String literal must be single line".to_string());
            } else if self.peek() == '\"' {
                self.advance();
                break;
            }
            self.advance();
        }
        Ok(TokenKind::String)
    }

    fn matches(&mut self, c: char) -> bool {
        if self.peek() == c {
            self.advance();
            true
        } else {
            false
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

    fn is_letter(&self, value: char) -> bool {
        value.is_ascii_alphabetic()
    }

    fn is_digit(&self, value: char) -> bool {
        value.is_ascii_digit()
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.source[self.current - 1]
    }
}
