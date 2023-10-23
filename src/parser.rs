use crate::lexer::TokenKind;
use crate::lexer::{Span, Token};
use std::vec::IntoIter;
// use Expr::*;

type BoxedExpr<'a> = Box<Expr<'a>>;

#[derive(Debug)]
pub enum Expr<'a> {
    Integer(i32),
    Float(f32),
    Binary {
        left: BoxedExpr<'a>,
        op: Token<'a>,
        right: BoxedExpr<'a>,
    },
}

struct Parser {
    // <'a> {
    // tokens: Vec<Token<'a>>,
    // token_iter: Box<Iterator<Item = Token<'a>>>,
    // token_iter: IntoIterator<Item = Token<'a>>,
}

pub fn parse<'a>(tokens: Vec<Token<'a>>) -> Expr<'a> {
    Parser::new().parse_tokens::<IntoIter<Token<'_>>>(tokens.into_iter())
}

impl Parser {
    fn new() -> Self {
        Self {}
    }

    fn parse_tokens<'a, Container: IntoIterator<Item = Token<'a>>>(
        &self,
        iter: Container,
    ) -> Expr<'a> {
        // let mut iterator = iter.into_iter();
        // Expr::Float(32.1)
        Expr::Binary {
            left: Box::new(Expr::Float(32.1)),
            op: Token {
                kind: TokenKind::Plus,
                lexeme: "+",
                position: Span { line: 1, column: 2 },
            },
            right: Box::new(Expr::Float(45.6)),
        }
    }

    // fn matches(&self, kind: TokenKind) -> bool {
    //     let is_match = self.check(kind);
    //     if is_match {
    //         self.advance();
    //     }
    //     is_match
    // }

    // fn check(&self) -> bool {
    //     if self.is_at_end() {
    //         false
    //     } else {
    //     }
    // }

    // fn advance(&self) -> () {}
}
