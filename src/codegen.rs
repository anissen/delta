use crate::bytecodes::ByteCode;
use crate::expressions::Expr;
use crate::tokens::TokenKind;

pub struct Codegen {
    bytes: Vec<u8>,
}

pub fn codegen(expressions: Vec<Expr>) -> Vec<u8> {
    Codegen::new().emit(expressions)
}

impl Codegen {
    fn new() -> Self {
        Self { bytes: vec![] }
    }

    fn do_emit(&mut self, expressions: Vec<Expr>) {
        for expr in expressions {
            match expr {
                Expr::Integer(i) => self.emit_bytes(ByteCode::PushInteger, i.to_be_bytes()),

                Expr::Float(f) => self.emit_bytes(ByteCode::PushFloat, f.to_be_bytes()),

                Expr::Binary {
                    left,
                    operator,
                    right,
                } => {
                    self.do_emit(vec![*left, *right]);
                    match operator.kind {
                        TokenKind::Plus => self.emit_bytecode(ByteCode::Addition),
                        TokenKind::Minus => self.emit_bytecode(ByteCode::Subtraction),
                        _ => {
                            println!("Unhandled binary expr: {:?}", operator);
                            ()
                        }
                    }
                }

                _ => {
                    println!("Unhandled expr: {:?}", expr);
                    ()
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
