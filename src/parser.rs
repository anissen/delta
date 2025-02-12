use crate::expressions::BinaryOperator;
use crate::expressions::Expr;
use crate::expressions::IsArm;
use crate::expressions::UnaryOperator;
use crate::tokens::Token;
use crate::tokens::TokenKind;
use crate::tokens::TokenKind::{
    BackSlash, Bang, BangEqual, Comment, Equal, EqualEqual, False, Float, Identifier, Integer,
    KeywordIs, LeftChevron, LeftChevronEqual, LeftParen, Minus, NewLine, Percent, Pipe, Plus,
    RightChevron, RightChevronEqual, RightParen, Slash, Space, Star, StringConcat, Tab, True,
    Underscore,
};

/*
program        → declaration* EOF ;
declaration    → funDecl | varDecl | expression ;
funDecl        → "\" IDENTIFIER IDENTIFIER* block ;
varDecl        → IDENTIFIER "=" expression ;
block          → "\n" INDENTATION declaration ("\n" INDENTATION declaration)* ;
expression     → assignment ;
assignment     → IDENTIFIER "=" logic_or ;
logic_or       → logic_and ( "or" logic_and )* ;
logic_and      → equality ( "and" equality )* ;
equality       → comparison ( ( "!=" | "==" ) comparison )* ;
comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
term           → factor ( ( "-" | "+" ) factor )* ;
factor         → unary ( ( "/" | "*" ) unary )* ;
unary          → ( "!" | "-" ) unary | call ;
call           → expression "|" primary expression? ;
primary        → "true" | "false" | NUMBER | STRING | IDENTIFIER | "(" expression ")" ;
---
NUMBER         → DIGIT+ ( "." DIGIT+ )? ;
STRING         → "\"" <any char except "\"">* "\"" ;
IDENTIFIER     → ALPHA ( ALPHA | DIGIT )* ;
ALPHA          → "a" ... "z" | "A" ... "Z" | "_" ;
DIGIT          → "0" ... "9" ;
---
MISSING:
- string concatenation
- is
*/

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
            .filter(|token| !matches!(token.kind, Space))
            .collect();
        // let kinds: Vec<TokenKind> = non_whitespace_tokens
        //     .clone()
        //     .into_iter()
        //     .map(|t| t.kind)
        //     .collect();
        // println!("condensed tokens: {:?}", kinds);
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

    // assignment → IDENTIFIER "=" logic_or
    fn assignment(&mut self) -> Result<Option<Expr>, String> {
        let expr = self.is()?;
        if expr.is_some() && self.matches(&Equal) {
            match expr.unwrap() {
                Expr::Value(name) => {
                    let token = self.previous();
                    let value = self.assignment()?;
                    Ok(Some(Expr::Assignment {
                        value: name,
                        _token: token,
                        expr: Box::new(value.unwrap()),
                    }))
                }

                _ => Err("Invalid assignment target".to_string()),
            }
        } else {
            Ok(expr)
        }
    }

    // is → string_concat "is" NEWLINE is_arm* | string_concat
    fn is(&mut self) -> Result<Option<Expr>, String> {
        let expr = self.string_concat()?;
        if self.matches(&KeywordIs) {
            self.consume(&NewLine)?;
            self.increase_indentation();
            let mut arms = vec![];
            let mut has_default = false;
            while self.matches_indentation() {
                let arm = self.is_arm()?;
                if has_default {
                    if arm.pattern.is_none() {
                        return Err("An `is` block cannot have multiple default arms.".to_string());
                    } else {
                        return Err("Unreachable due to default arm above.".to_string());
                    }
                }
                has_default = arm.pattern.is_none();

                // TODO(anissen): Check for multiple capture arms or arms after a capture arm

                arms.push(arm);
            }
            if arms.is_empty() {
                return Err("`is` block must have at least one arm".to_string());
            }
            self.decrease_indentation();
            Ok(Some(Expr::Is {
                expr: Box::new(expr.unwrap()),
                arms,
            }))
        } else {
            Ok(expr)
        }
    }

    // is_arm → INDENT ( ( "_" | expression ) block )
    fn is_arm(&mut self) -> Result<IsArm, String> {
        for _ in 0..self.indentation {
            self.consume(&Tab)?;
        }

        if self.matches(&Underscore) {
            // Default arm
            if let Some(block) = self.block()? {
                Ok(IsArm {
                    pattern: None,
                    block,
                })
            } else {
                Err("Error parsing block of default `is` arm".to_string())
            }
        } else if let Some(pattern) = self.expression()? {
            // Non-default arm
            if let Some(block) = self.block()? {
                Ok(IsArm {
                    pattern: Some(pattern),
                    block,
                })
            } else {
                Err("Error parsing block of `is` arm".to_string())
            }
        } else {
            Err("Error parsing pattern of `is` arm".to_string())
        }
    }

    // string_concat → STRING "{" logic_or "}";
    fn string_concat(&mut self) -> Result<Option<Expr>, String> {
        let mut expr = self.logic_or()?;
        while expr.is_some() && self.matches(&StringConcat) {
            let token = self.previous();
            let right = self.logic_or()?;
            expr = Some(Expr::Binary {
                left: Box::new(expr.unwrap()),
                operator: BinaryOperator::StringConcat,
                _token: token,
                right: Box::new(right.unwrap()),
            });
        }
        Ok(expr)
    }

    // logic_or → logic_and ( "or" logic_and )* ;
    fn logic_or(&mut self) -> Result<Option<Expr>, String> {
        // TODO(anissen): Implement
        self.logic_and()
    }

    // logic_and → equality ( "and" equality )* ;
    fn logic_and(&mut self) -> Result<Option<Expr>, String> {
        // TODO(anissen): Implement
        self.equality()
    }

    // equality → comparison ( ( "!=" | "==" ) comparison )* ;
    fn equality(&mut self) -> Result<Option<Expr>, String> {
        let expr = self.comparison()?;
        if expr.is_some() && self.matches_any(&[EqualEqual, BangEqual]) {
            let token = self.previous();
            let right = self.comparison()?;
            Ok(Some(Expr::Comparison {
                left: Box::new(expr.unwrap()),
                token,
                right: Box::new(right.unwrap()),
            }))
        } else {
            Ok(expr)
        }
    }

    // comparison → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
    fn comparison(&mut self) -> Result<Option<Expr>, String> {
        let expr = self.term()?;
        if expr.is_some()
            && self.matches_any(&[
                LeftChevron,
                LeftChevronEqual,
                RightChevron,
                RightChevronEqual,
            ])
        {
            let token = self.previous();
            let right = self.term()?;
            Ok(Some(Expr::Comparison {
                left: Box::new(expr.unwrap()),
                token,
                right: Box::new(right.unwrap()),
            }))
        } else {
            Ok(expr)
        }
    }

    // term → factor ( ( "-" | "+" ) factor )* ;
    fn term(&mut self) -> Result<Option<Expr>, String> {
        let mut expr = self.factor()?;
        while expr.is_some() && self.matches_any(&[Plus, Minus]) {
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
                _token: token,
                right: Box::new(right.unwrap()),
            });
        }
        Ok(expr)
    }

    // factor → unary ( ( "/" | "*" ) unary )* ;
    fn factor(&mut self) -> Result<Option<Expr>, String> {
        let mut expr = self.unary()?;
        while expr.is_some() && self.matches_any(&[Slash, Star, Percent]) {
            let token = self.previous();
            let operator = match token.kind {
                Slash => BinaryOperator::Division,
                Star => BinaryOperator::Multiplication,
                Percent => BinaryOperator::Modulus,
                _ => panic!("unreachable"),
            };
            let right = self.unary()?;
            expr = Some(Expr::Binary {
                left: Box::new(expr.unwrap()),
                operator,
                _token: token,
                right: Box::new(right.unwrap()),
            });
        }
        Ok(expr)
    }

    // unary → ( "!" | "-" ) unary | call ;
    fn unary(&mut self) -> Result<Option<Expr>, String> {
        if self.matches_any(&[Bang, Minus]) {
            let token = self.previous();
            let operator = match token.kind {
                Bang => UnaryOperator::Not,
                Minus => UnaryOperator::Negation,
                _ => panic!("cannot happen"),
            };
            let right = self.unary()?;
            Ok(Some(Expr::Unary {
                operator,
                _token: token,
                expr: Box::new(right.unwrap()),
            }))
        } else {
            self.call()
        }
    }

    // call → primary "|" primary* | primary
    fn call(&mut self) -> Result<Option<Expr>, String> {
        let expr = self.primary()?;
        if self.matches(&Pipe) {
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
        // TODO(anissen): Checking for string concatenation here does not feel right
        while !self.is_at_end()
            && !self.check(&NewLine)
            && !self.check(&Pipe)
            && !self.check(&StringConcat)
        {
            let arg = self.primary()?; // precedence after string concatenation
            if let Some(arg) = arg {
                args.push(arg);
            }
        }
        let call_expr = Expr::Call { name: lexeme, args };
        if self.matches(&Pipe) {
            self.call_with_first_arg(call_expr)
        } else {
            Ok(Some(call_expr))
        }
    }

    // function → IDENTIFIER* block
    fn function(&mut self) -> Result<Option<Expr>, String> {
        let mut params = vec![];
        while self.matches(&Identifier) {
            let param = self.previous();
            params.push(param);
        }
        let expr = self.block()?;
        Ok(Some(Expr::Function {
            params,
            expr: Box::new(expr.unwrap()),
        }))
    }

    // block → NEWLINE (INDENT expression NEWLINE?)*
    fn block(&mut self) -> Result<Option<Expr>, String> {
        self.consume(&NewLine)?;
        self.increase_indentation();
        let mut exprs = vec![];
        loop {
            for _ in 0..self.indentation {
                self.consume(&Tab)?;
            }
            if let Some(expr) = self.expression()? {
                exprs.push(expr);
            }

            // Required when having nested blocks to avoid each consuming the newline
            if self.check(&NewLine) {
                self.consume(&NewLine)?;
            }

            if !self.matches_indentation() {
                // TODO(anissen): Should probably Err if indentation is wrong
                break;
            }
        }
        self.decrease_indentation();
        Ok(Some(Expr::Block { exprs }))
    }

    // whitespace → NEWLINE | COMMENT
    fn whitespace(&mut self) -> Result<Option<Expr>, String> {
        if self.matches_any(&[NewLine, Comment]) {
            Ok(None)
        } else if self.is_at_end() {
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

    // primary → "true" | "false" | INTEGER | FLOAT | STRING | IDENTIFIER | "(" expression ")" ;
    fn primary(&mut self) -> Result<Option<Expr>, String> {
        if self.matches(&Identifier) {
            let lexeme = self.previous().lexeme;
            Ok(Some(Expr::Value(lexeme)))
        } else if self.matches(&Integer) {
            let lexeme = self.previous().lexeme;
            let value = lexeme.parse::<i32>();
            match value {
                Ok(value) => Ok(Some(Expr::Integer(value))),
                Err(err) => Err(err.to_string()),
            }
        } else if self.matches(&Float) {
            let lexeme = self.previous().lexeme;
            let value = lexeme.parse::<f32>();
            match value {
                Ok(value) => Ok(Some(Expr::Float(value))),
                Err(err) => Err(err.to_string()),
            }
        } else if self.matches(&False) {
            Ok(Some(Expr::Boolean(false)))
        } else if self.matches(&True) {
            Ok(Some(Expr::Boolean(true)))
        } else if self.matches(&TokenKind::String) {
            let lexeme = self.previous().lexeme;
            Ok(Some(Expr::String(lexeme)))
        } else if self.matches(&LeftParen) {
            let expr = self.expression()?;
            self.consume(&RightParen)?;
            Ok(Some(Expr::Grouping(Box::new(expr.unwrap()))))
        } else if self.matches(&BackSlash) {
            self.function()
        } else {
            self.whitespace()
        }
    }

    fn matches_indentation(&self) -> bool {
        (0..self.indentation as usize).all(|i| {
            self.tokens.len() > self.current + 1 && self.tokens[self.current + i].kind == Tab
        })
    }

    fn matches_any(&mut self, kinds: &[TokenKind]) -> bool {
        kinds.iter().any(|k| self.matches(k))
    }

    fn matches(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
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

    fn increase_indentation(&mut self) {
        self.indentation += 1;
    }

    fn decrease_indentation(&mut self) {
        self.indentation -= 1;
    }
}
