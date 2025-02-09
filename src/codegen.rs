use std::collections::{HashMap, HashSet};

use crate::bytecodes::ByteCode;
use crate::expressions::{BinaryOperator, Expr, UnaryOperator};
use crate::program::Context;
use crate::tokens::TokenKind;

pub struct Codegen<'a> {
    bytes: Vec<u8>,
    function_count: u8,
    context: &'a Context<'a>,
}

pub fn codegen<'a>(expressions: Vec<Expr>, context: &'a Context<'a>) -> Vec<u8> {
    Codegen::new(&context).emit(expressions)
}

// TODO(anissen): Add a function overview mapping for each scope containing { name, arity, starting IP, source line number  }.
// This will be used directly in the VM as well as for debug logging.

impl<'a> Codegen<'a> {
    fn new(context: &'a Context<'a>) -> Self {
        Self {
            bytes: vec![],
            function_count: 0,
            context,
        }
    }

    fn emit_exprs(
        &mut self,
        expressions: &Vec<Expr>,
        environment: &mut HashMap<String, u8>,
        locals: &mut HashSet<String>,
    ) {
        for expr in expressions {
            self.emit_expr(expr, environment, locals);
        }
    }

    fn emit_expr(
        &mut self,
        expr: &Expr,
        environment: &mut HashMap<String, u8>,
        locals: &mut HashSet<String>,
    ) {
        // TODO(anissen): Make this a proper return type.
        println!("do_emit with expr: {:?}", expr);
        match expr {
            Expr::Boolean(true) => self.emit_bytecode(ByteCode::PushTrue),

            Expr::Boolean(false) => self.emit_bytecode(ByteCode::PushFalse),

            Expr::Integer(i) => self.emit_bytes(ByteCode::PushInteger, i.to_be_bytes()),

            Expr::Float(f) => self.emit_bytes(ByteCode::PushFloat, f.to_be_bytes()),

            Expr::Value(name) => {
                if self.context.has_value(&name) {
                    self.emit_bytecode(ByteCode::GetForeignValue);
                    // TODO(anissen): Should (also) output index
                    if name.len() > 255 {
                        panic!("function name too long!");
                    }
                    self.emit_byte(name.len() as u8);
                    self.emit_raw_bytes(&mut name.as_bytes().to_vec());
                } else {
                    println!("read value: {}", name);
                    let index = environment.get(name).unwrap();
                    self.emit_bytecode(ByteCode::GetLocalValue);
                    self.emit_byte(*index);
                }
            }

            Expr::String(str) => {
                self.emit_bytecode(ByteCode::PushString);
                if str.len() > 255 {
                    panic!("string too long!");
                }
                self.emit_byte(str.len() as u8);
                self.emit_raw_bytes(&mut str.as_bytes().to_vec());
            }

            Expr::Grouping(expr) => self.emit_expr(expr, environment, locals),

            Expr::Block { exprs } => self.emit_exprs(exprs, environment, locals),

            Expr::Function { params, expr } => {
                let mut function_environment = environment.clone();
                let mut function_locals = HashSet::new();

                self.emit_bytecode(ByteCode::FunctionStart);

                for (index, param) in params.iter().enumerate() {
                    println!("set param: {}", param.lexeme);
                    function_environment.insert(param.lexeme.clone(), index as u8);

                    function_locals.insert(param.lexeme.clone());
                }
                // bytecodes: function start, function index, param count, function body, function end

                self.emit_byte(self.function_count);
                self.function_count += 1;

                self.emit_byte(params.len() as u8); // TODO(anissen): Guard against overflow

                // emit function signatures here?
                // e.g.
                // function_signatures = []
                // bytes = emit(expr, ...) // also populates function_signatures
                // write_bytes(function_signatures)
                // write_bytes(bytes)

                println!("function_environment: {:?}", function_environment);
                self.emit_expr(expr, &mut function_environment, &mut function_locals);

                self.emit_bytecode(ByteCode::FunctionEnd);
            }

            Expr::Call { name, args } => {
                let arg_count = args.len();
                self.emit_exprs(args, environment, locals);

                if self.context.has_function(&name) {
                    // TODO(anissen): Maybe this should be its own Expr instead?
                    self.emit_bytecode(ByteCode::CallForeign);
                    self.emit_byte(self.context.get_index(&name));
                    self.emit_byte(arg_count as u8);

                    if name.len() > 255 {
                        panic!("function name too long!");
                    }
                    self.emit_byte(name.len() as u8);
                    self.emit_raw_bytes(&mut name.as_bytes().to_vec());
                } else {
                    self.emit_bytecode(ByteCode::Call);
                    self.emit_byte(arg_count as u8);
                    println!(
                        "call function '{}' with environment: {:?}, locals: {:?}",
                        name, environment, locals
                    );
                    let index = environment.get(name).unwrap();
                    if locals.contains(name) {
                        self.emit_byte(0);
                    } else {
                        self.emit_byte(1);
                    }
                    self.emit_byte(*index);

                    if name.len() > 255 {
                        panic!("function name too long!");
                    }
                    self.emit_byte(name.len() as u8);
                    self.emit_raw_bytes(&mut name.as_bytes().to_vec());
                };
            }

            Expr::Assignment {
                value,
                _token: _,
                expr,
            } => {
                println!("assignment with environment: {:?}", environment);
                self.emit_expr(expr, environment, locals);
                self.emit_bytecode(ByteCode::SetLocalValue);

                let index = locals.len() as u8;
                println!("insert value {} at index {}", value, index);
                environment.insert(value.clone(), index);
                locals.insert(value.clone());
                self.emit_byte(index);
            }

            Expr::Comparison { left, token, right } => {
                // println!("comparison, with environment: {:?}", environment);
                self.emit_expr(left, environment, locals);
                self.emit_expr(right, environment, locals);

                match token.kind {
                    TokenKind::EqualEqual => self.emit_bytecode(ByteCode::Equals),
                    TokenKind::BangEqual => {
                        self.emit_bytecode(ByteCode::Equals);
                        self.emit_bytecode(ByteCode::Not);
                    }
                    TokenKind::LeftChevron => self.emit_bytecode(ByteCode::LessThan),
                    TokenKind::LeftChevronEqual => self.emit_bytecode(ByteCode::LessThanEquals),
                    TokenKind::RightChevron => {
                        self.emit_bytecode(ByteCode::LessThanEquals);
                        self.emit_bytecode(ByteCode::Not);
                    }
                    TokenKind::RightChevronEqual => {
                        self.emit_bytecode(ByteCode::LessThan);
                        self.emit_bytecode(ByteCode::Not);
                    }
                    _ => panic!("unexpected comparison operator"),
                }
            }

            Expr::Unary {
                operator,
                _token: _,
                expr,
            } => match operator {
                UnaryOperator::Negation => {
                    self.emit_expr(expr, environment, locals);
                    self.emit_bytecode(ByteCode::Negation);
                }

                UnaryOperator::Not => {
                    self.emit_expr(expr, environment, locals);
                    self.emit_bytecode(ByteCode::Not);
                }
            },

            Expr::Binary {
                left,
                operator,
                _token: _,
                right,
            } => {
                self.emit_expr(left, environment, locals);
                self.emit_expr(right, environment, locals);
                match operator {
                    BinaryOperator::Addition => self.emit_bytecode(ByteCode::Addition),

                    BinaryOperator::Subtraction => self.emit_bytecode(ByteCode::Subtraction),

                    BinaryOperator::Multiplication => self.emit_bytecode(ByteCode::Multiplication),

                    BinaryOperator::Division => self.emit_bytecode(ByteCode::Division),

                    BinaryOperator::Modulus => self.emit_bytecode(ByteCode::Modulo),

                    BinaryOperator::StringConcat => self.emit_bytecode(ByteCode::StringConcat),
                }
            }

            Expr::Is { expr, arms } => {
                let mut jump_to_end_offsets = vec![];
                for arm in arms {
                    if let Some(pattern) = &arm.pattern {
                        self.emit_expr(expr, environment, locals);

                        self.emit_expr(&pattern, environment, locals);

                        self.emit_bytecode(ByteCode::Equals);
                        self.emit_bytecode(ByteCode::Not);

                        let next_arm_offset = self.emit_jump_if_true();

                        self.emit_expr(&arm.block, environment, locals);

                        self.emit_bytecode(ByteCode::PushTrue); // Hack to avoid having a jump instruction without arguments
                        let end_offset = self.emit_jump_if_true();
                        jump_to_end_offsets.push(end_offset);

                        self.patch_jump_to_current_byte(next_arm_offset);
                    } else {
                        self.emit_expr(&arm.block, environment, locals);
                    }
                }

                // TODO(anissen): default block goes here

                for offset in jump_to_end_offsets {
                    self.patch_jump_to_current_byte(offset);
                }

                /*
                x is
                    true
                        y
                    false
                        z
                ==>
                x
                arm1.pattern
                jne arm2
                arm1.block
                goto end
                arm2: jne arm3
                arm2.block
                goto end
                arm3: jne arm4
                ...
                goto end
                default-block
                end:
                */
            }
        };
    }

    // pub fn emit_foreign_functions(&mut self, context: &'static Context<'static>) {
    //     for (index, function) in context.get_function_names().into_iter().enumerate() {
    //         self.emit_bytecode(ByteCode::ForeignFunction);
    //         self.emit_byte(index as u8); // TODO: Handle overflow
    //     }
    // }

    pub fn emit(&mut self, expressions: Vec<Expr>) -> Vec<u8> {
        println!("emit with fresh environment");
        let environment = &mut HashMap::new();
        let locals = &mut HashSet::new();
        self.emit_exprs(&expressions, environment, locals);
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

    fn emit_raw_bytes(&mut self, bytes: &mut Vec<u8>) {
        self.bytes.append(bytes);
    }

    fn emit_jump_if_true(&mut self) -> usize {
        let bytes = 0_i32.to_be_bytes();
        self.emit_bytes(ByteCode::JumpIfTrue, bytes /* placeholder */);
        self.bytes.len() - bytes.len()
    }

    // fn emit_unconditional_jump(&mut self) -> usize {
    //     self.emit_bytecode(ByteCode::Jump);
    //     self.emit_byte(0); // placeholder

    //     // self.emit_bytes(
    //     //     ByteCode::JumpIfTrue,
    //     //     0_i32.to_be_bytes(), /* placeholder */
    //     // );
    //     self.bytes.len() - 1
    // }

    fn patch_jump_to_current_byte(&mut self, byte_offset: usize) {
        // byte offset is the start of 4 bytes that indicate the jump offset
        let jump_instruction_bytes = 4;
        let jump_offset = (self.bytes.len() - (byte_offset + jump_instruction_bytes)) as i32;
        jump_offset
            .to_be_bytes()
            .swap_with_slice(&mut self.bytes[byte_offset..byte_offset + 4]);
    }
}
