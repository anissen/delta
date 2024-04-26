use crate::lexer::Token;
use crate::lexer::TokenKind;
use crate::lexer::TokenKind::{Bang, Float, Integer, Minus, Plus, Slash, Space, Star, EOF};

#[derive(Debug)]
pub enum Expr {
    // Error(),
    Indentation(u32),
    Integer(i32),
    Float(f32),
    Unary {
        operator: Token,
        expr: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

pub fn parse(tokens: Vec<Token>) -> Vec<Expr> {
    Parser::new(tokens).parse()
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    fn parse(&mut self) -> Vec<Expr> {
        let mut expressions = Vec::new();
        loop {
            if let Ok(expression) = self.term() {
                expressions.push(expression);
            } else {
                // synchronize
                // TODO(anissen): Handle error synchronization
                break;
            }
            if self.is_at_end() {
                break;
            }
        }
        expressions
    }

    fn term(&mut self) -> Result<Expr, String> {
        let mut expr = self.factor()?;
        while self.matches(&[Plus, Minus]) {
            let operator = self.previous();
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, String> {
        let mut expr = self.unary()?;
        while self.matches(&[Slash, Star]) {
            let operator = self.previous();
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, String> {
        if self.matches(&[Bang, Minus]) {
            let operator = self.previous();
            let right = self.unary()?;
            Ok(Expr::Unary {
                operator,
                expr: Box::new(right),
            })
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<Expr, String> {
        if self.matches(&[Space]) {
            Ok(Expr::Indentation(1))
        } else if self.matches(&[Integer]) {
            let lexeme = self.previous().lexeme;
            let value = lexeme.parse::<i32>();
            match value {
                Ok(value) => Ok(Expr::Integer(value)),
                Err(err) => Err(err.to_string()),
            }
        } else if self.matches(&[Float]) {
            let lexeme = self.previous().lexeme;
            let value = lexeme.parse::<f32>();
            match value {
                Ok(value) => Ok(Expr::Float(value)),
                Err(err) => Err(err.to_string()),
            }
        } else {
            Err("parse error".to_string())
        }

        // if self.matches(&[False]) {
        //     return Ok(Expr::Literal(Literal::Bool(false)));
        // }
        // if self.matches(&[True]) {
        //     return Ok(Expr::Literal(Literal::Bool(true)));
        // }
        // if self.matches(&[Nil]) {
        //     return Ok(Expr::Literal(Literal::Nil));
        // }
        // if self.matches(&[Number, String_]) {
        //     return Ok(Expr::Literal(match self.previous().literal {
        //         Some(l) => l,
        //         None => Literal::Nil,
        //     }));
        // }
        // if self.matches(&[Identifier]) {
        //     return Ok(Expr::Variable(self.previous()));
        // }
        // if self.matches(&[LeftParen]) {
        //     let expr = self.expression()?;
        //     self.consume(&RightParen, "Expect `)` after expression")?;
        //     return Ok(Expr::Grouping(Box::new(expr)));
        // }
        // crate::error_at_token(&self.peek(), "Expect expression");
        // Err("Parse error")
    }

    fn matches(&mut self, kinds: &[TokenKind]) -> bool {
        for kind in kinds {
            if self.check(kind) {
                self.advance();
                return true;
            }
        }
        false
    }

    // fn consume(&mut self, type_: &TokenType, message: &str) -> Result<Token> {
    //     if self.check(type_) {
    //         Ok(self.advance())
    //     } else {
    //         crate::error_at_token(&self.peek(), message);
    //         Err(anyhow!("Parse error"))
    //     }
    // }

    fn check(&self, kind: &TokenKind) -> bool {
        if self.is_at_end() {
            false
        } else {
            &self.peek().kind == kind
        }
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().kind == EOF
    }

    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }
}
