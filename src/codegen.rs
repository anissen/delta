use std::collections::HashMap;

use crate::bytecodes::ByteCode;
use crate::expressions::{BinaryOperator, Expr, UnaryOperator};

pub struct Codegen {
    bytes: Vec<u8>,
    function_count: u8,
}

pub fn codegen(expressions: Vec<Expr>) -> Vec<u8> {
    Codegen::new().emit(expressions)
}

impl Codegen {
    fn new() -> Self {
        Self {
            bytes: vec![],
            function_count: 0,
        }
    }

    // TODO(anissen): We need a mapping from variable to an index

    fn do_emit(&mut self, expressions: Vec<Expr>) {
        let mut localVariableIndex: u8 = 0;
        // TODO(anissen): Make this a proper return type.
        for expr in expressions {
            match expr {
                Expr::Boolean(b) => {
                    self.emit_bytecode(ByteCode::PushBoolean);
                    self.emit_byte(b as u8);
                }

                Expr::Integer(i) => self.emit_bytes(ByteCode::PushInteger, i.to_be_bytes()),

                Expr::Float(f) => self.emit_bytes(ByteCode::PushFloat, f.to_be_bytes()),

                Expr::Variable(name) => {
                    println!("{}", name);
                    self.emit_bytecode(ByteCode::GetValue);
                    self.emit_byte(localVariableIndex);
                    localVariableIndex += 1;
                }

                Expr::Grouping(expr) => self.do_emit(vec![*expr]),

                Expr::Function { params, expr } => {
                    self.emit_bytecode(ByteCode::FunctionStart);

                    // bytecodes: function start, function index, param count, function body, function end

                    self.emit_byte(self.function_count);
                    self.function_count += 1;

                    self.emit_byte(params.len() as u8); // TODO(anissen): Guard against overflow
                    self.do_emit(vec![*expr]);

                    self.emit_bytecode(ByteCode::FunctionEnd);
                }

                Expr::Call { name, args } => {
                    let arg_count = args.len();
                    self.do_emit(args);
                    self.emit_bytecode(ByteCode::Call);
                    self.emit_byte(arg_count as u8);
                    // self.emit_byte(*self.value_index.get(&name).unwrap());
                }

                Expr::Assignment { variable, expr } => {
                    self.do_emit(vec![*expr]);
                    self.emit_bytecode(ByteCode::SetValue);

                    self.emit_byte(localVariableIndex);
                    localVariableIndex += 1;
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
                        self.do_emit(vec![*expr]);
                        self.emit_bytecode(ByteCode::Not);
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
