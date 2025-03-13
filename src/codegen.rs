use std::collections::{HashMap, HashSet};

use crate::bytecodes::ByteCode;
use crate::diagnostics::{Diagnostics, Message};
use crate::expressions::{BinaryOperator, Expr, IsArmPattern, UnaryOperator};
use crate::program::Context;
use crate::tokens::{Span, Token, TokenKind};

#[derive(Debug, Clone)]
struct FunctionSignature {
    byte_position: u32, // TODO(anissen): Should be an index into the function chunks vector.
}

#[derive(Debug)]
struct FunctionChunk<'a> {
    local_count: u8,
    bytes: Vec<u8>,
    function_name: String,
    position: &'a Span,
    byte_position: u32,
}

pub struct Codegen<'a> {
    bytecode: BytecodeBuilder,
    function_signatures: Vec<FunctionSignature>,
    function_count: u8,
    function_chunks: Vec<FunctionChunk<'a>>,
    context: &'a Context<'a>,
    diagnostics: &'a mut Diagnostics<'a>,
}

pub fn codegen<'a>(
    expressions: &'a Vec<Expr>,
    context: &'a Context<'a>,
    diagnostics: &'a mut Diagnostics<'a>,
) -> Vec<u8> {
    Codegen::new(context, diagnostics).emit(expressions)
}

// TODO(anissen): Add a function overview mapping for each scope containing { name, arity, starting IP, source line number  }.
// This will be used directly in the VM as well as for debug logging.

impl<'a> Codegen<'a> {
    fn new(context: &'a Context<'a>, diagnostics: &'a mut Diagnostics<'a>) -> Self {
        Self {
            bytecode: BytecodeBuilder::new(),
            function_signatures: vec![],
            function_count: 0,
            function_chunks: vec![],
            context,
            diagnostics,
        }
    }

    fn emit_exprs(
        &mut self,
        expressions: &'a Vec<Expr>,
        environment: &mut HashMap<String, u8>,
        locals: &mut HashSet<String>,
    ) {
        for expr in expressions {
            self.emit_expr(expr, environment, locals);
        }
    }

    fn emit_expr(
        &mut self,
        expr: &'a Expr,
        environment: &mut HashMap<String, u8>,
        locals: &mut HashSet<String>,
    ) {
        match expr {
            Expr::Boolean(true) => {
                self.bytecode.add_op(ByteCode::PushTrue);
            }

            Expr::Boolean(false) => {
                self.bytecode.add_op(ByteCode::PushFalse);
            }

            Expr::Integer(i) => {
                self.bytecode.add_op(ByteCode::PushInteger).add_i32(i);
            }

            Expr::Float(f) => {
                self.bytecode.add_op(ByteCode::PushFloat).add_f32(f);
            }

            Expr::Value { name } => {
                let lexeme = &name.lexeme;
                if self.context.has_value(lexeme) {
                    // TODO(anissen): Should (also) output index
                    if lexeme.len() > 255 {
                        let message = Message::new(
                            format!("Function name too long: {}", lexeme),
                            &name.position,
                        );
                        self.diagnostics.add_error(message);
                    }
                    self.bytecode
                        .add_op(ByteCode::GetForeignValue)
                        .add_byte(lexeme.len() as u8)
                        .add_byte_array(lexeme.as_bytes());
                } else if let Some(index) = environment.get(lexeme) {
                    self.emit_get_local_value(*index);
                } else {
                    let msg = Message::new(
                        format!("Name not found in scope: {}", lexeme),
                        &name.position,
                    );
                    self.diagnostics.add_error(msg);
                }
            }

            Expr::String(str) => {
                if str.len() > 255 {
                    // TODO(anissen): Should add error to a error reporter instead
                    panic!("string too long!");
                }
                self.bytecode
                    .add_op(ByteCode::PushString)
                    .add_byte(str.len() as u8)
                    .add_byte_array(str.as_bytes());
            }

            Expr::Grouping(expr) => self.emit_expr(expr, environment, locals),

            Expr::Block { exprs } => {
                let mut block_environment = environment.clone();
                let mut block_locals = locals.clone();

                // Emit block with its own environment and locals
                self.emit_exprs(exprs, &mut block_environment, &mut block_locals);
            }

            Expr::Function {
                slash,
                params,
                expr,
            } => self.emit_function(slash, None, params, expr, environment, locals),

            Expr::Call { name, args } => {
                let arg_count = args.len();
                self.emit_exprs(args, environment, locals);

                if self.context.has_function(name) {
                    // TODO(anissen): Maybe this should be its own Expr instead?
                    self.bytecode
                        .add_op(ByteCode::CallForeign)
                        .add_byte(self.context.get_index(name))
                        .add_byte(arg_count as u8);
                } else {
                    self.bytecode
                        .add_op(ByteCode::Call)
                        .add_byte(arg_count as u8);
                    let index = environment.get(name).unwrap();
                    if locals.contains(name) {
                        self.bytecode.add_byte(0);
                    } else {
                        self.bytecode.add_byte(1);
                    }
                    self.bytecode.add_byte(*index);
                };

                if name.len() > 255 {
                    panic!("function name too long!");
                    // let msg = Message::new(format!("Function name too long: {}", name), ;
                }
                self.bytecode
                    .add_byte(name.len() as u8)
                    .add_byte_array(name.as_bytes());
            }

            Expr::Assignment {
                name,
                _operator: _,
                expr,
            } => {
                self.emit_assignment(name, expr, environment, locals);
            }

            Expr::Comparison { left, token, right } => {
                self.emit_expr(left, environment, locals);
                self.emit_expr(right, environment, locals);

                match token.kind {
                    TokenKind::EqualEqual => {
                        self.bytecode.add_op(ByteCode::Equals);
                    }
                    TokenKind::BangEqual => {
                        self.bytecode.add_op(ByteCode::Equals).add_op(ByteCode::Not);
                    }
                    TokenKind::LeftChevron => {
                        self.bytecode.add_op(ByteCode::LessThan);
                    }
                    TokenKind::LeftChevronEqual => {
                        self.bytecode.add_op(ByteCode::LessThanEquals);
                    }
                    TokenKind::RightChevron => {
                        self.bytecode
                            .add_op(ByteCode::LessThanEquals)
                            .add_op(ByteCode::Not);
                    }
                    TokenKind::RightChevronEqual => {
                        self.bytecode
                            .add_op(ByteCode::LessThan)
                            .add_op(ByteCode::Not);
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
                    self.bytecode.add_op(ByteCode::Negation);
                }
                UnaryOperator::Not => {
                    self.emit_expr(expr, environment, locals);
                    self.bytecode.add_op(ByteCode::Not);
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
                    BinaryOperator::Addition => self.bytecode.add_op(ByteCode::Addition),
                    BinaryOperator::Subtraction => self.bytecode.add_op(ByteCode::Subtraction),
                    BinaryOperator::Multiplication => {
                        self.bytecode.add_op(ByteCode::Multiplication)
                    }
                    BinaryOperator::Division => self.bytecode.add_op(ByteCode::Division),
                    BinaryOperator::Modulus => self.bytecode.add_op(ByteCode::Modulo),
                    BinaryOperator::StringConcat => self.bytecode.add_op(ByteCode::StringConcat),
                    BinaryOperator::BooleanAnd => self.bytecode.add_op(ByteCode::BooleanAnd),
                    BinaryOperator::BooleanOr => self.bytecode.add_op(ByteCode::BooleanOr),
                };
            }

            Expr::Is { expr, arms } => {
                let index = match **expr {
                    Expr::Value { ref name } => {
                        // If the value is already in the environment, use its index
                        let index_option = environment.get(&name.lexeme);
                        *index_option.unwrap()
                    }
                    _ => {
                        // Otherwise, emit the expression and add it to the locals
                        // to avoid emitting the same value multiple times
                        self.emit_expr(expr, environment, locals);
                        let index = locals.len() as u8;
                        self.emit_set_local_value(index);
                        index
                    }
                };

                let mut jump_to_end_offsets = vec![];
                for arm in arms {
                    match &arm.pattern {
                        IsArmPattern::Expression(pattern) => {
                            // Emit expression and pattern and compare
                            self.emit_get_local_value(index);
                            self.emit_expr(pattern, environment, locals);
                            self.bytecode.add_op(ByteCode::Equals);

                            // Jump to next arm if not equal
                            let next_arm_offset = self.bytecode.add_jump_if_false();

                            // Otherwise execute arm block
                            self.emit_expr(&arm.block, environment, locals);

                            // Jump to end of `is` block
                            let end_offset = self.bytecode.add_unconditional_jump();
                            jump_to_end_offsets.push(end_offset);

                            // Patch jump to next arm now that we know its position
                            self.bytecode.patch_jump_to_current_byte(next_arm_offset);
                        }

                        IsArmPattern::Capture {
                            identifier,
                            condition,
                        } => {
                            self.emit_assignment(identifier, expr, environment, locals);

                            if let Some(condition) = condition {
                                // Emit expression and condition and compare
                                self.emit_get_local_value(index);
                                self.emit_expr(condition, environment, locals);

                                // Jump to next arm if not equal
                                let next_arm_offset = self.bytecode.add_jump_if_false();

                                // Otherwise execute arm block
                                self.emit_expr(&arm.block, environment, locals);

                                // Jump to end of `is` block
                                let end_offset = self.bytecode.add_unconditional_jump();
                                jump_to_end_offsets.push(end_offset);

                                // Patch jump to next arm now that we know its position
                                self.bytecode.patch_jump_to_current_byte(next_arm_offset);
                            } else {
                                // Otherwise execute arm block
                                self.emit_expr(&arm.block, environment, locals);

                                // Jump to end of `is` block
                                let end_offset = self.bytecode.add_unconditional_jump();
                                jump_to_end_offsets.push(end_offset);
                            }
                        }

                        IsArmPattern::Default => {
                            self.emit_expr(&arm.block, environment, locals);
                        }
                    };
                }

                // Patch all jumps to end of `is` block now that we know where it ends
                for offset in jump_to_end_offsets {
                    self.bytecode.patch_jump_to_current_byte(offset);
                }
            }
        };
    }

    fn emit_assignment(
        &mut self,
        name: &Token,
        expr: &'a Expr,
        environment: &mut HashMap<String, u8>,
        locals: &mut HashSet<String>,
    ) {
        match expr {
            Expr::Function {
                slash,
                params,
                expr,
            } => {
                self.emit_function(slash, Some(name), params, expr, environment, locals);
            }

            _ => {
                self.emit_expr(expr, environment, locals);
            }
        }

        let index = locals.len() as u8;
        environment.insert(name.lexeme.clone(), index);
        locals.insert(name.lexeme.clone());

        self.emit_set_local_value(index);
    }

    fn emit_set_local_value(&mut self, index: u8) {
        self.bytecode
            .add_op(ByteCode::SetLocalValue)
            .add_byte(index);
    }

    fn emit_get_local_value(&mut self, index: u8) {
        self.bytecode
            .add_op(ByteCode::GetLocalValue)
            .add_byte(index);
    }

    fn emit_function(
        &mut self,
        slash: &'a Token,
        name: Option<&Token>,
        params: &[Token],
        expr: &'a Expr,
        environment: &mut HashMap<String, u8>,
        locals: &mut HashSet<String>,
    ) {
        let mut function_environment = environment.clone();
        let mut function_locals = HashSet::new();

        self.bytecode.add_op(ByteCode::Function);
        self.function_signatures.push(FunctionSignature {
            byte_position: self.bytecode.bytes.len() as u32 - 1,
        });

        self.create_function_chunk(name, &slash.position, params, expr, environment, locals);

        for (index, param) in params.iter().enumerate() {
            function_environment.insert(param.lexeme.clone(), index as u8);

            function_locals.insert(param.lexeme.clone());
        }
        // bytecodes: function start, function index, param count, function body, function end

        self.bytecode.add_byte(self.function_count);
        self.function_count += 1;

        self.bytecode.add_byte(params.len() as u8); // TODO(anissen): Guard against overflow

        let jump_to_end = self.bytecode.add_unconditional_jump();

        // TODO(anissen): Expr is already a block, so we shouldn't need to create new environment and locals
        self.emit_expr(expr, &mut function_environment, &mut function_locals);

        self.bytecode.add_op(ByteCode::Return);

        self.bytecode.patch_jump_to_current_byte(jump_to_end);
    }

    fn create_function_chunk(
        &mut self,
        name: Option<&Token>,
        position: &'a Span,
        params: &[Token],
        expr: &Expr,
        environment: &mut HashMap<String, u8>,
        locals: &mut HashSet<String>,
    ) {
        if params.len() > u8::MAX.into() {
            panic!("Too many parameters");
        }

        let lexeme = match name {
            Some(name) => name.lexeme.clone(),
            None => "(unnamed)".to_string(),
        };

        let mut function_environment = environment.clone();
        let mut function_locals = HashSet::new();

        let byte_position = self.bytecode.bytes.len() as u32 - 1;

        let mut bytecode = BytecodeBuilder::new();
        bytecode.add_op(ByteCode::Function);

        for (index, param) in params.iter().enumerate() {
            function_environment.insert(param.lexeme.clone(), index as u8);
            function_locals.insert(param.lexeme.clone());
        }

        // TODO(anissen): Expr is already a block, so we shouldn't need to create new environment and locals
        // TODO(anissen): Combine bytecode, environment and locals into a single struct
        // self.emit_expr(
        //     expr,
        //     &mut bytecode,
        //     &mut function_environment,
        //     &mut function_locals,
        // );

        let function_chunk = FunctionChunk {
            function_name: lexeme,
            position,
            local_count: params.len() as u8,
            byte_position,
            bytes: vec![],
        };

        self.function_chunks.push(function_chunk);
    }

    pub fn emit(&mut self, expressions: &'a Vec<Expr>) -> Vec<u8> {
        let environment = &mut HashMap::new();
        let locals = &mut HashSet::new();
        self.emit_exprs(expressions, environment, locals);

        let mut signature_builder = BytecodeBuilder::new();

        for ele in self.function_signatures.clone() {
            signature_builder.add_op(ByteCode::FunctionSignature);
            for byte in ele.byte_position.to_be_bytes() {
                signature_builder.add_byte(byte);
            }
        }

        println!("Function chunks:");
        for ele in &self.function_chunks {
            println!("{:?}", ele);
        }

        [signature_builder.bytes, self.bytecode.bytes.clone()].concat()
    }
}

struct BytecodeBuilder {
    bytes: Vec<u8>,
}

impl BytecodeBuilder {
    fn new() -> Self {
        Self { bytes: Vec::new() }
    }

    fn add_byte(&mut self, byte: u8) -> &mut Self {
        self.bytes.push(byte);
        self
    }

    fn add_op(&mut self, code: ByteCode) -> &mut Self {
        self.add_byte(code.into());
        self
    }

    fn add_bytes<const COUNT: usize>(&mut self, value: &[u8; COUNT]) -> &mut Self {
        self.bytes.extend_from_slice(value);
        self
    }

    // fn add_u16(&mut self, value: u16) -> &mut Self {
    //     self.add_bytes(&value.to_be_bytes())
    // }

    // fn add_u32(&mut self, value: u32) -> &mut Self {
    //     self.add_bytes(&value.to_be_bytes())
    // }

    fn add_i32(&mut self, value: &i32) -> &mut Self {
        self.add_bytes(&value.to_be_bytes())
    }

    fn add_f32(&mut self, value: &f32) -> &mut Self {
        self.add_bytes(&value.to_be_bytes())
    }

    fn add_byte_array(&mut self, bytes: &[u8]) {
        self.bytes.extend(bytes);
    }

    fn add_jump_if_false(&mut self) -> usize {
        let bytes = 0_i16.to_be_bytes();
        self.add_op(ByteCode::JumpIfFalse)
            .add_bytes(&bytes /* placeholder */);
        self.bytes.len() - bytes.len()
    }

    fn add_unconditional_jump(&mut self) -> usize {
        let bytes = 0_i16.to_be_bytes();
        self.add_op(ByteCode::Jump)
            .add_bytes(&bytes /* placeholder */);
        self.bytes.len() - bytes.len()
    }

    fn patch_jump_to_current_byte(&mut self, byte_offset: usize) {
        // byte offset is the start of 2 bytes that indicate the jump offset
        let jump_instruction_bytes = 2;
        let jump_offset = (self.bytes.len() - (byte_offset + jump_instruction_bytes)) as isize;
        if jump_offset < i16::MIN as isize {
            dbg!(jump_offset);
            panic!("Jump offset is too small");
        } else if jump_offset > i16::MAX as isize {
            panic!("Jump offset is too large");
        }
        (jump_offset as i16)
            .to_be_bytes()
            .swap_with_slice(&mut self.bytes[byte_offset..byte_offset + 2]);
    }
}
