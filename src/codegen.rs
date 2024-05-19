use std::collections::HashMap;

use crate::bytecodes::ByteCode;
use crate::expressions::Expr;
use crate::tokens::TokenKind;

pub struct Codegen {
    bytes: Vec<u8>,
    value_index: HashMap<String, u8>,
}

pub fn codegen(expressions: Vec<Expr>) -> Vec<u8> {
    Codegen::new().emit(expressions)
}

impl Codegen {
    fn new() -> Self {
        Self {
            bytes: vec![],
            value_index: HashMap::new(),
        }
    }

    // TODO(anissen): We need a mapping from variable to an index

    fn do_emit(&mut self, expressions: Vec<Expr>) {
        // TODO(anissen): Make this a proper return type.
        for expr in expressions {
            match expr {
                Expr::Comment(_) => (), // do nothing

                Expr::Integer(i) => self.emit_bytes(ByteCode::PushInteger, i.to_be_bytes()),

                Expr::Float(f) => self.emit_bytes(ByteCode::PushFloat, f.to_be_bytes()),

                Expr::Variable(name) => {
                    self.emit_bytecode(ByteCode::GetValue);
                    let index = self.value_index.get(&name).unwrap();
                    self.emit_byte(*index);
                }

                Expr::Assignment { variable, expr } => {
                    self.do_emit(vec![*expr]);
                    self.emit_bytecode(ByteCode::SetValue);
                    let index = self.value_index.len() as u8; // TODO(anissen): This cast is bad, m'kay!?
                    self.value_index.insert(variable, index);
                    self.emit_byte(index);
                }

                Expr::Unary { operator, expr } => match operator.kind {
                    TokenKind::Bang => {
                        self.do_emit(vec![*expr]);
                        self.emit_bytecode(ByteCode::Negation);
                    }

                    _ => {
                        println!("Unhandled unary expr: {:?}", operator);
                        panic!("Unhandled unary expr");
                    }
                },

                Expr::Binary {
                    left,
                    operator,
                    right,
                } => {
                    self.do_emit(vec![*left, *right]);
                    match operator.kind {
                        TokenKind::Plus => self.emit_bytecode(ByteCode::Addition),
                        TokenKind::Minus => self.emit_bytecode(ByteCode::Subtraction),
                        TokenKind::Star => self.emit_bytecode(ByteCode::Multiplication),
                        TokenKind::Slash => self.emit_bytecode(ByteCode::Division),
                        _ => {
                            println!("Unhandled binary expr: {:?}", operator);
                            panic!("Unhandled binary expr");
                        }
                    }
                }
            };
        }
    }

    pub fn emit(&mut self, expressions: Vec<Expr>) -> Vec<u8> {
        self.do_emit(expressions);
        self.bytes.clone()
    }

    fn emit_byte(&mut self, byte: u8) {
        self.bytes.push(byte);
    }

    fn emit_bytecode(&mut self, code: ByteCode) {
        self.emit_byte(code.into());
    }

    fn emit_bytes(&mut self, code: ByteCode, value: [u8; 4]) {
        self.emit_bytecode(code);
        for byte in value {
            self.emit_byte(byte);
        }
    }
}
