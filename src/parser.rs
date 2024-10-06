use crate::expressions::BinaryOperator;
use crate::expressions::Expr;
use crate::expressions::UnaryOperator;
use crate::tokens::Token;
use crate::tokens::TokenKind;
use crate::tokens::TokenKind::{
    BackSlash, Bang, Comment, Equal, False, Float, Identifier, Integer, LeftParen, Minus, NewLine,
    Pipe, Plus, RightParen, Slash, Space, Star, True,
};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    indentation: u8,
}

pub fn parse(tokens: Vec<Token>) -> Result<Vec<Expr>, String> {
    Parser::new(tokens).parse()
}

// TODO(anissen): Clean up `.unwrap` in this file

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        let non_whitespace_tokens: Vec<Token> = tokens
            .into_iter()
            .filter(|token| match token.kind {
                Space => false,
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
            indentation: 0,
        }
    }

    fn parse(&mut self) -> Result<Vec<Expr>, String> {
        let mut expressions = Vec::new();
        loop {
            let res = self.expression()?;
            if let Some(expression) = res {
                expressions.push(expression);
            }
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

    fn expression(&mut self) -> Result<Option<Expr>, String> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Option<Expr>, String> {
        let expr = self.term()?;
        if expr.is_some() && self.matches(&[Equal]) {
            match expr.unwrap() {
                Expr::Variable(name) => {
                    let token = self.previous();
                    let value = self.assignment()?;
                    Ok(Some(Expr::Assignment {
                        variable: name,
                        token,
                        expr: Box::new(value.unwrap()),
                    }))
                }

                _ => Err("Invalid assignment target".to_string()),
            }
        } else {
            Ok(expr)
        }
    }

    fn term(&mut self) -> Result<Option<Expr>, String> {
        let mut expr = self.factor()?;
        while expr.is_some() && self.matches(&[Plus, Minus]) {
            let token = self.previous();
            let operator = match token.kind {
                Plus => BinaryOperator::Addition,
                Minus => BinaryOperator::Subtraction,
                _ => panic!("unreachable"),
            };
            let right = self.factor()?;
            expr = Some(Expr::Binary {
                left: Box::new(expr.unwrap()),
                operator,
                token,
                right: Box::new(right.unwrap()),
            });
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Option<Expr>, String> {
        let mut expr = self.unary()?;
        while expr.is_some() && self.matches(&[Slash, Star]) {
            let token = self.previous();
            let operator = match token.kind {
                Slash => BinaryOperator::Division,
                Star => BinaryOperator::Multiplication,
                _ => panic!("unreachable"),
            };
            let right = self.unary()?;
            expr = Some(Expr::Binary {
                left: Box::new(expr.unwrap()),
                operator,
                token,
                right: Box::new(right.unwrap()),
            });
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Option<Expr>, String> {
        if self.matches(&[Bang, Minus]) {
            let token = self.previous();
            let operator = match token.kind {
                Bang => UnaryOperator::Not,
                Minus => UnaryOperator::Negation,
                _ => panic!("cannot happen"),
            };
            let right = self.unary()?;
            Ok(Some(Expr::Unary {
                operator,
                token,
                expr: Box::new(right.unwrap()),
            }))
        } else {
            self.call()
        }
    }

    fn call(&mut self) -> Result<Option<Expr>, String> {
        let expr = self.primary()?;
        if self.matches(&[Pipe]) {
            self.call_with_first_arg(expr.unwrap())
        } else {
            Ok(expr)
        }
    }

    fn call_with_first_arg(&mut self, expr: Expr) -> Result<Option<Expr>, String> {
        self.consume(&Identifier)?;
        let lexeme = self.previous().lexeme;
        // TODO(anissen): Check that function name exists and is a function
        let first_arg = expr;
        let mut args = vec![first_arg];
        while !self.is_at_end() && !self.check(&NewLine) && !self.check(&Pipe) {
            let arg = self.expression()?;
            if let Some(arg) = arg {
                args.push(arg);
            }
        }
        let call_expr = Expr::Call { name: lexeme, args };
        if self.matches(&[Pipe]) {
            self.call_with_first_arg(call_expr)
        } else {
            Ok(Some(call_expr))
        }
    }

    fn function(&mut self) -> Result<Option<Expr>, String> {
        let mut params = vec![];
        while self.matches(&[TokenKind::Identifier]) {
            let param = self.previous();
            params.push(param);
        }
        let expr = self.block()?;
        Ok(Some(Expr::Function {
            params,
            expr: Box::new(expr.unwrap()),
        }))
    }

    fn block(&mut self) -> Result<Option<Expr>, String> {
        self.consume(&TokenKind::NewLine)?;
        self.indentation += 1;
        for _ in 0..self.indentation {
            self.consume(&TokenKind::Tab)?;
        }
        let mut exprs = vec![];
        loop {
            if let Some(expr) = self.expression()? {
                exprs.push(expr);
            }
            self.consume(&TokenKind::NewLine)?;
            let matches_indentation =
                (0..self.indentation).all(|_| self.matches(&[TokenKind::Tab])); // TODO(anissen): Should probably Err if indentation is wrong
            if !matches_indentation {
                break;
            }
        }
        self.indentation -= 1;
        Ok(Some(Expr::Block { exprs }))
    }

    fn whitespace(&mut self) -> Result<Option<Expr>, String> {
        if self.matches(&[NewLine, Comment]) {
            Ok(None)
        } else {
            let error = format!(
                "Parse error of kind {:?} at {:?} ({:?})",
                self.peek().kind,
                self.previous().lexeme,
                self.previous().position
            );
            Err(error)
        }
    }

    fn primary(&mut self) -> Result<Option<Expr>, String> {
        if self.matches(&[Identifier]) {
            let lexeme = self.previous().lexeme;
            Ok(Some(Expr::Variable(lexeme)))
        } else if self.matches(&[Integer]) {
            let lexeme = self.previous().lexeme;
            let value = lexeme.parse::<i32>();
            match value {
                Ok(value) => Ok(Some(Expr::Integer(value))),
                Err(err) => Err(err.to_string()),
            }
        } else if self.matches(&[Float]) {
            let lexeme = self.previous().lexeme;
            let value = lexeme.parse::<f32>();
            match value {
                Ok(value) => Ok(Some(Expr::Float(value))),
                Err(err) => Err(err.to_string()),
            }
        } else if self.matches(&[False]) {
            Ok(Some(Expr::Boolean(false)))
        } else if self.matches(&[True]) {
            Ok(Some(Expr::Boolean(true)))
        } else if self.matches(&[LeftParen]) {
            let expr = self.expression()?;
            self.consume(&RightParen)?;
            Ok(Some(Expr::Grouping(Box::new(expr.unwrap()))))
        } else if self.matches(&[BackSlash]) {
            self.function()
        } else {
            self.whitespace()
        }
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
            let message = format!("Expected {:?} but found {:?}", kind, &self.peek());
            Err(message.to_string())
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
