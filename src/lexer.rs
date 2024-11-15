use crate::tokens::{Span, Token, TokenKind};

struct Lexer {
    source: Vec<char>, // TODO(anissen): Should this be a `str`?
    start: usize,
    current: usize,
    line: usize,
    column: usize,
    string_interpolation: bool,
    tokens: Vec<Token>,
    errors: Vec<Error>,
}

#[derive(Debug, Clone)]
pub struct Error {
    pub position: Span,
    pub lexeme: String,      // TODO(anissen): Should probably be &'a str,
    pub description: String, // TODO(anissen): Should probably be &'a str,
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
                    "! syntax error at '{}' (line {}, column {}): {}",
                    err.lexeme, err.position.line, err.position.column, err.description
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
            string_interpolation: false,
            tokens: vec![],
            errors: vec![],
        }
    }

    fn scan_tokens(&mut self, source: &'a str) -> Result<Vec<Token>, Errors> {
        self.source = source.chars().collect();

        while !self.is_at_end() {
            self.start = self.current;
            self.scan_next();
        }

        if !self.errors.is_empty() {
            Err(self.errors.clone()) // HACK
        } else {
            Ok(self.tokens.clone()) // HACK
        }
    }

    fn add_token_kind(&mut self, kind: TokenKind) {
        let lexeme = self.source[self.start..self.current].iter().collect();
        self.add_token_kind_with_lexeme(kind, lexeme);
    }

    fn add_token_kind_with_lexeme(&mut self, kind: TokenKind, lexeme: String) {
        let position = Span {
            line: self.line,
            column: self.column,
        };

        let token = Token {
            kind,
            position,
            lexeme,
        };
        self.add_token(token);
    }

    fn add_token(&mut self, token: Token) {
        let is_new_line = token.kind == TokenKind::NewLine;
        self.tokens.push(token);
        if is_new_line {
            self.line += 1;
            self.column = 0;
        } else {
            self.column += self.current - self.start;
        }
    }

    fn add_error(&mut self, description: String) {
        let position = Span {
            line: self.line,
            column: self.column,
        };
        let lexeme = self.source[self.start..self.current].iter().collect();
        self.errors.push(Error {
            position,
            lexeme,
            description,
        })
    }

    fn scan_next(&mut self) {
        let char = self.advance();
        // TODO(anissen): This can hopefully be cleaned up somewhat -- e.g. have a set of simple cases that just return a tokenkind and a more advanced that handles tokens itself.
        match char {
            ' ' => self.spaces(),
            '+' => self.add_token_kind(TokenKind::Plus),
            '-' if self.is_digit(self.peek()) => {
                let number = self.number();
                self.add_token_kind(number)
            }
            '-' => self.add_token_kind(TokenKind::Minus),
            '*' => self.add_token_kind(TokenKind::Star),
            '/' => self.add_token_kind(TokenKind::Slash),
            '%' => self.add_token_kind(TokenKind::Percent),
            '\\' => self.add_token_kind(TokenKind::BackSlash),
            '!' => self.add_token_kind(TokenKind::Bang),
            '=' if self.matches('=') => self.add_token_kind(TokenKind::EqualEqual),
            '=' => self.add_token_kind(TokenKind::Equal),
            '#' => {
                let comment = self.comment();
                self.add_token_kind(comment)
            }
            '|' => self.add_token_kind(TokenKind::Pipe),
            '(' => self.add_token_kind(TokenKind::LeftParen),
            ')' => self.add_token_kind(TokenKind::RightParen),
            '{' => self.add_token_kind(TokenKind::LeftBrace),
            '}' if self.string_interpolation => {
                self.add_token_kind(TokenKind::StringConcat);
                self.string_interpolation = false;
                self.string();
            }
            '}' => self.add_token_kind(TokenKind::RightBrace),
            '\t' => self.add_token_kind(TokenKind::Tab),
            '\n' => self.add_token_kind(TokenKind::NewLine),
            '\"' => self.string(),
            c if self.is_letter(c) => {
                let identifier = self.identifier();
                self.add_token_kind(identifier)
            }
            c if self.is_digit(c) => {
                let number = self.number();
                self.add_token_kind(number)
            }
            _ => self.add_error(format!("Unexpected token: {}", char)),
        }
    }

    fn spaces(&mut self) {
        let mut spaces = 1;
        while !self.is_at_end() && self.peek() == ' ' {
            self.advance();
            spaces += 1;
        }
        match spaces {
            1 => self.add_token_kind(TokenKind::Space),
            4 => self.add_token_kind(TokenKind::Tab), // HACK because Zed cannot handle hard tabs correctly. Scanning for '\t' should be sufficient.
            _ => self.add_error("Unexpected whitespace".to_string()),
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

    fn string(&mut self) {
        loop {
            if self.is_at_end() {
                return self.add_error("Unterminated string".to_string());
            } else {
                match self.peek() {
                    '\n' => {
                        return self.add_error("String literal must be single line".to_string())
                    }
                    '\"' => {
                        self.advance();
                        break;
                    }
                    '{' => {
                        let lexeme = self.source[self.start + 1..self.current]
                            .iter()
                            .collect::<String>();
                        self.add_string_token(lexeme);

                        self.advance(); // Skip '{'
                        self.add_token_kind(TokenKind::StringConcat);
                        self.start = self.current;
                        self.string_interpolation = true;
                        return;
                    }
                    _ => (), // no-op
                }
                self.advance();
            }
        }

        let lexeme = self.source[self.start + 1..self.current - 1]
            .iter()
            .collect::<String>();
        self.add_string_token(lexeme);
    }

    fn add_string_token(&mut self, value: String) {
        let escaped = value
            .replace("\\n", "\n")
            .replace("\\t", "\t")
            .replace("\\\'", "\'");

        self.add_token_kind_with_lexeme(TokenKind::String, escaped);
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
