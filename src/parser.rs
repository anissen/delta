use crate::expressions::BinaryOperator;
use crate::expressions::Expr;
use crate::expressions::UnaryOperator;
use crate::tokens::Token;
use crate::tokens::TokenKind;
use crate::tokens::TokenKind::{
    Bang, Comment, Equal, False, Float, Identifier, Integer, LeftParen, Minus, NewLine, Plus,
    RightParen, Slash, Space, Star, True,
};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

pub fn parse(tokens: Vec<Token>) -> Result<Vec<Expr>, String> {
    Parser::new(tokens).parse()
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        let non_whitespace_tokens: Vec<Token> = tokens
            .into_iter()
            .filter(|token| match token.kind {
                Comment | NewLine | Space => false, // TODO(anissen): We probably need to keep newline for semantic later
                _ => true,
            })
            .collect();
        let kinds: Vec<TokenKind> = non_whitespace_tokens
            .clone()
            .into_iter()
            .map(|t| t.kind)
            .collect();
        println!("condensed tokens: {:?}", kinds);
        Self {
            tokens: non_whitespace_tokens,
            current: 0,
        }
    }

    fn parse(&mut self) -> Result<Vec<Expr>, String> {
        let mut expressions = Vec::new();
        loop {
            let res = self.expression()?;
            expressions.push(res);
            // if let Ok(expression) = res {
            //     expressions.push(expression);
            // } else {
            //     // synchronize
            //     // TODO(anissen): Handle error synchronization
            //     // println!("Error detected: {:?}", res);
            //     // break;
            // }
            if self.is_at_end() {
                break;
            }
        }
        Ok(expressions)
    }

    fn expression(&mut self) -> Result<Expr, String> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, String> {
        // TODO(anissen): Fix precedence
        // let expr = self.or()?;
        let expr = self.term()?;
        if self.matches(&[Equal]) {
            match expr {
                Expr::Variable(name) => {
                    let value = self.assignment()?;
                    Ok(Expr::Assignment {
                        variable: name,
                        expr: Box::new(value),
                    })
                }

                _ => Err("Invalid assignment target".to_string()),
            }
        } else {
            Ok(expr)
        }
    }

    fn term(&mut self) -> Result<Expr, String> {
        let mut expr = self.factor()?;
        while self.matches(&[Plus, Minus]) {
            let token = self.previous();
            let operator = match token.kind {
                Plus => BinaryOperator::Addition,
                Minus => BinaryOperator::Subtraction,
                _ => panic!("unreachable"),
            };
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                token,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, String> {
        let mut expr = self.unary()?;
        while self.matches(&[Slash, Star]) {
            let token = self.previous();
            let operator = match token.kind {
                Slash => BinaryOperator::Division,
                Star => BinaryOperator::Multiplication,
                _ => panic!("unreachable"),
            };
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                token,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, String> {
        if self.matches(&[Bang, Minus]) {
            let token = self.previous();
            let operator = match token.kind {
                Bang => UnaryOperator::Not,
                Minus => UnaryOperator::Negation,
                _ => panic!("cannot happen"),
            };
            let right = self.unary()?;
            Ok(Expr::Unary {
                operator,
                token,
                expr: Box::new(right),
            })
        } else {
            self.primary()
        }
    }

    // fn whitespace(&mut self) -> Result<Option<Expr>, String> {
    //     if self.matches(&[Comment]) {
    //         Ok(Some(Expr::Comment(self.previous().lexeme)))
    //     } else if self.matches(&[NewLine, Space]) {
    //         Ok(None)
    //     } else {
    //         Err("Unhandled whitespace".to_string())
    //     }
    // }

    fn primary(&mut self) -> Result<Expr, String> {
        if self.matches(&[Identifier]) {
            Ok(Expr::Variable(self.previous().lexeme))
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
        } else if self.matches(&[False]) {
            return Ok(Expr::Boolean(false));
        } else if self.matches(&[True]) {
            return Ok(Expr::Boolean(true));
        } else if self.matches(&[LeftParen]) {
            let expr = self.expression()?;
            self.consume(&RightParen);
            return Ok(Expr::Grouping(Box::new(expr)));
        } else {
            let error = format!(
                "Parse error of kind {:?} at {:?} ({:?})",
                self.peek().kind,
                self.previous().lexeme,
                self.previous().position
            );
            Err(error)
        }
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

    fn consume(&mut self, kind: &TokenKind) -> Result<Token, String> {
        if self.check(kind) {
            Ok(self.advance())
        } else {
            Err("Unexpected token".to_string())
        }
    }

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
        self.current >= self.tokens.len()
    }

    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }
}
