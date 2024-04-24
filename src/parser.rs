use crate::lexer::TokenKind;
use crate::lexer::{Span, Token};
use std::vec::IntoIter;
// use Expr::*;

type BoxedExpr<'a> = Box<Expr<'a>>;

#[derive(Debug)]
pub enum Expr<'a> {
    Error(),
    Integer(i32),
    Float(f32),
    Binary {
        left: BoxedExpr<'a>,
        operator: Token<'a>,
        right: BoxedExpr<'a>,
    },
}

struct Parser {
    // <'a> {
    // tokens: Vec<Token<'a>>,
    // token_iter: Box<Iterator<Item = Token<'a>>>,
    // token_iter: IntoIterator<Item = Token<'a>>,
}

pub fn parse<'a>(tokens: Vec<Token<'a>>) -> Vec<Expr<'a>> {
    Parser::new().parse_tokens::<IntoIter<Token>>(tokens.into_iter())
}

impl Parser {
    fn new() -> Self {
        Self {}
    }

    fn parse_tokens<'a, Container: IntoIterator<Item = Token<'a>>>(
        &self,
        iter: Container,
    ) -> Vec<Expr<'a>> {
        // let mut iterator = iter.into_iter();
        // Expr::Float(32.1)

        let peekable_iter = iter.into_iter().peekable(); // TODO: The peakable trait is unused
        peekable_iter.map(|_| Expr::Float(12.3)).collect()

        peekable_iter.

        // Expr::Binary {
        //     left: Box::new(Expr::Float(32.1)),
        //     op: Token {
        //         kind: TokenKind::Plus,
        //         lexeme: "+",
        //         position: Span { line: 1, column: 2 },
        //     },
        //     right: Box::new(Expr::Float(45.6)),
        // }
    }

    fn addition(&self) -> Expr {
        let expr = self.multiplication();

        while self.is_kind(TokenKind::Plus) {
            let operator = self.previous();
            let right = self.multiplication();
            expr = Expr::Binary {
                left: expr,
                operator,
                right,
            };
        }
        expr
    }

    fn multiplication(&self) -> Expr {
        if self.is_kind(TokenKind::Star) {
            let left = multiplication();
            let operator = previous();
            let right = multiplication();
            Expr::Binary {
                left,
                operator,
                right,
            }
        } else {
            Expr::Error()
        }
    }

    fn is_kind(&self, kind: TokenKind) -> bool {
        let is_match = self.check(kind);
        if is_match {
            self.advance();
        }
        is_match
    }

    // fn check(&self) -> bool {
    //     if self.is_at_end() {
    //         false
    //     } else {
    //     }
    // }

    // fn advance(&self) -> () {}
}
