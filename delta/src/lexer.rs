use crate::tokens::{Position, Token, TokenKind};

struct Lexer {
    source: Vec<char>, // TODO(anissen): Should this be a `str`?
    start: usize,
    current: usize,
    line: usize,
    column: usize,
    string_interpolation: bool,
    tokens: Vec<Token>,
}

pub fn lex(source: &str) -> Vec<Token> {
    Lexer::new().scan_tokens(source)
}

impl Lexer {
    fn new() -> Self {
        Self {
            source: Vec::default(),
            start: 0,
            current: 0,
            line: 1,
            column: 1,
            string_interpolation: false,
            tokens: vec![],
        }
    }

    fn scan_tokens(&mut self, source: &str) -> Vec<Token> {
        self.source = source.chars().collect();

        while !self.is_at_end() {
            self.start = self.current;
            let token_kind = self.scan_next();
            let lexeme = self.source[self.start..self.current].iter().collect();
            let token = match token_kind {
                TokenKind::Text => self.get_string_token(lexeme),
                _ => self.get_token_from_lexeme(token_kind, lexeme),
            };
            self.add_token(token);
        }

        self.tokens.clone()
    }

    fn add_token_kind(&mut self, kind: TokenKind) {
        let lexeme = self.source[self.start..self.current].iter().collect();
        let token = self.get_token_from_lexeme(kind, lexeme);
        self.add_token(token);
    }

    fn get_token_from_lexeme(&mut self, kind: TokenKind, lexeme: String) -> Token {
        let position = Position {
            line: self.line,
            column: self.column,
        };

        Token {
            kind,
            position,
            lexeme,
        }
    }

    fn add_token(&mut self, token: Token) {
        let is_new_line = token.kind == TokenKind::NewLine;
        self.tokens.push(token);
        if is_new_line {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += self.current - self.start;
        }
    }

    fn scan_next(&mut self) -> TokenKind {
        let char = self.advance();
        match char {
            ' ' => self.spaces(),
            '+' if self.matches('.') => TokenKind::PlusDot,
            '+' => TokenKind::Plus,
            '-' if self.is_digit(self.peek()) => self.number(),
            '-' if self.matches('.') => TokenKind::MinusDot,
            '-' => TokenKind::Minus,
            '*' if self.matches('.') => TokenKind::StarDot,
            '*' => TokenKind::Star,
            '/' if self.matches('.') => TokenKind::SlashDot,
            '/' => TokenKind::Slash,
            '%' if self.matches('.') => TokenKind::PercentDot,
            '%' => TokenKind::Percent,
            '\\' => TokenKind::BackSlash,
            '!' if self.matches('=') => TokenKind::BangEqual,
            '!' => TokenKind::Bang,
            '=' if self.matches('=') => TokenKind::EqualEqual,
            '=' => TokenKind::Equal,
            ',' => TokenKind::Comma,
            '#' => self.comment(),
            '|' => TokenKind::Pipe,
            '∆' => self.context(),
            '.' => TokenKind::Dot,
            '(' => TokenKind::LeftParen,
            ')' => TokenKind::RightParen,
            '[' => TokenKind::LeftBracket,
            ']' => TokenKind::RightBracket,
            '{' => TokenKind::LeftBrace,
            '}' if self.string_interpolation => {
                self.string_interpolation = false;
                if self.matches('"') {
                    // String ends after concatenation, skip this token
                    self.scan_next()
                } else {
                    self.add_token_kind(TokenKind::StringConcat);
                    self.string()
                }
            }
            '}' => TokenKind::RightBrace,
            '<' if self.matches('=') => {
                if self.matches('.') {
                    TokenKind::LeftChevronEqualDot
                } else {
                    TokenKind::LeftChevronEqual
                }
            }
            '<' if self.matches('.') => TokenKind::LeftChevronDot,
            '<' => TokenKind::LeftChevron,
            '>' if self.matches('=') => {
                if self.matches('.') {
                    TokenKind::RightChevronEqualDot
                } else {
                    TokenKind::RightChevronEqual
                }
            }
            '>' if self.matches('.') => TokenKind::RightChevronDot,
            '>' => TokenKind::RightChevron,
            '_' => TokenKind::Underscore,
            '\t' => TokenKind::Tab,
            '\n' => TokenKind::NewLine,
            ':' => self.tag(),
            '\"' => self.string(),
            c if self.is_digit(c) => self.number(),
            _ if self.match_keyword("is") => TokenKind::KeywordIs,
            _ if self.match_keyword("if") => TokenKind::KeywordIf,
            _ if self.match_keyword("and") => TokenKind::KeywordAnd,
            _ if self.match_keyword("or") => TokenKind::KeywordOr,
            _ if self.match_keyword("component") => TokenKind::KeywordComponent,
            _ if self.match_keyword("query") => TokenKind::KeywordQuery,
            _ if self.match_keyword("create") => TokenKind::KeywordCreate,
            _ if self.match_keyword("f32") => TokenKind::KeywordF32,
            // '∆' if self.matches('.') && self.is_letter(self.peek()) => self.context_value(), // TODO(anissen): Key-value pair, e.g. ∆.x = y
            // '∆' => self.identifier(), // TODO(anissen): Named context, e.g. ∆x or ∆x.y
            c if self.is_letter(c) => self.identifier(),
            _ => TokenKind::SyntaxError("Unexpected token"),
        }
    }

    fn spaces(&mut self) -> TokenKind {
        let mut spaces = 1;
        while !self.is_at_end() && self.peek() == ' ' {
            self.advance();
            spaces += 1;
            if spaces == 4 {
                // Need to split multiple indentations into tabs
                break;
            }
        }
        match spaces {
            1 => TokenKind::Space,
            4 => TokenKind::Tab, // HACK because Zed cannot handle hard tabs correctly. Scanning for '\t' should be sufficient.
            _ => TokenKind::SyntaxError(
                "Unexpected whitespace (number of spaces more than 1 and less than 4)",
            ),
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

    // fn context_value(&mut self) -> TokenKind {
    //     while !self.is_at_end()
    //         && (self.is_letter(self.peek()) || self.is_digit(self.peek()) || self.peek() == '_')
    //     {
    //         self.advance();
    //     }

    //     TokenKind::Identifier
    // }

    fn match_keyword(&mut self, keyword: &str) -> bool {
        if self.source.len() <= self.current + keyword.len() {
            false
        } else {
            // TODO(anissen): This char-by-char check does not work for multi-char characters (e.g. UTF8)
            let matches_keyword = keyword
                .chars()
                .enumerate()
                .all(|(i, c)| self.peek_at(i) == c);
            if !matches_keyword {
                false
            } else {
                let token_after_keyword = self.peek_at(keyword.len());
                match token_after_keyword {
                    '\0' | ' ' | ',' | '\n' => {
                        // TODO(anissen): This should be an allow-list (e.g. [a-z][0-1]) instead of a black-list
                        self.advance_many(keyword.len() - 1);
                        true
                    }
                    _ => false,
                }
            }
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

    fn context(&mut self) -> TokenKind {
        self.identifier();
        TokenKind::Context
    }

    fn comment(&mut self) -> TokenKind {
        while !self.is_at_end() && self.peek() != '\n' {
            self.advance();
        }
        TokenKind::Comment
    }

    fn tag(&mut self) -> TokenKind {
        loop {
            if self.is_at_end() {
                return TokenKind::SyntaxError("Unterminated tag");
            }
            if self.peek().is_whitespace() {
                break;
            }
            self.advance();
        }

        // self.start += 1;
        let length = self.current - self.start;
        if length == 0 {
            TokenKind::SyntaxError("Empty tag")
        } else {
            TokenKind::Tag
        }
    }

    fn string(&mut self) -> TokenKind {
        loop {
            if self.is_at_end() {
                return TokenKind::SyntaxError("Unterminated string");
            } else {
                match self.peek() {
                    '\n' => return TokenKind::SyntaxError("String literal must be single line"),
                    '\"' => {
                        self.advance();
                        break;
                    }
                    '{' => {
                        let lexeme = self.source[self.start + 1..self.current]
                            .iter()
                            .collect::<String>();
                        let string_token = self.get_string_token(lexeme);
                        self.add_token(string_token);

                        self.start = self.current;
                        self.advance();
                        self.string_interpolation = true;
                        return TokenKind::StringConcat;
                    }
                    _ => (), // no-op
                }
                self.advance();
            }
        }

        self.start += 1;
        TokenKind::Text
    }

    fn get_string_token(&mut self, value: String) -> Token {
        let escaped_value = self.escape_string(value);
        self.get_token_from_lexeme(TokenKind::Text, escaped_value)
    }

    fn escape_string(&mut self, value: String) -> String {
        value
            .trim_start_matches("\"")
            .trim_end_matches("\"")
            .replace("\\n", "\n")
            .replace("\\t", "\t")
            .replace("\\\'", "\'")
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

    fn peek_at(&self, index: usize) -> char {
        if self.is_at_end() {
            Default::default()
        } else {
            self.source[self.current - 1 + index]
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

    fn advance_many(&mut self, count: usize) {
        for _ in 0..count {
            self.advance();
        }
    }
}
