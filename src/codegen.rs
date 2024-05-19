use std::collections::HashMap;

use crate::bytecodes::ByteCode;
use crate::expressions::{BinaryOperator, Expr, UnaryOperator};

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

                Expr::Unary {
                    operator,
                    token: _,
                    expr,
                } => match operator {
                    UnaryOperator::Negation => {
                        self.do_emit(vec![*expr]);
                        self.emit_bytecode(ByteCode::Negation);
                    }

                    UnaryOperator::Not => {
                        panic!("not implemented"); // TODO(anissen): Implement
                    }
                },

                Expr::Binary {
                    left,
                    operator,
                    token: _,
                    right,
                } => {
                    self.do_emit(vec![*left, *right]);
                    match operator {
                        BinaryOperator::Addition => self.emit_bytecode(ByteCode::Addition),

                        BinaryOperator::Subtraction => self.emit_bytecode(ByteCode::Subtraction),

                        BinaryOperator::Multiplication => {
                            self.emit_bytecode(ByteCode::Multiplication)
                        }

                        BinaryOperator::Division => self.emit_bytecode(ByteCode::Division),
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
