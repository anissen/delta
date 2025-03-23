use std::collections::{HashMap, HashSet};

use crate::bytecodes::ByteCode;
use crate::diagnostics::{Diagnostics, Message};
use crate::expressions::{BinaryOperator, Expr, IsArmPattern, UnaryOperator};
use crate::program::Context;
use crate::tokens::{Span, Token, TokenKind};

#[derive(Debug, Clone)]
struct FunctionChunk<'a> {
    local_count: u8,
    bytes: Vec<u8>,
    position: &'a Span,
    function_name: String,
}

pub struct Scope {
    bytecode: BytecodeBuilder,
    environment: HashMap<String, u8>,
    locals: HashSet<String>,
}

impl Scope {
    fn new() -> Self {
        Self {
            bytecode: BytecodeBuilder::new(),
            environment: HashMap::new(),
            locals: HashSet::new(),
        }
    }

    fn function(&mut self) -> Self {
        Self {
            bytecode: BytecodeBuilder::new(),
            environment: self.environment.clone(),
            locals: HashSet::new(),
        }
    }
}

pub struct Codegen<'a> {
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
            function_chunks: vec![],
            context,
            diagnostics,
        }
    }

    fn emit_exprs(&mut self, expressions: &'a Vec<Expr>, scope: &mut Scope) {
        for expr in expressions {
            self.emit_expr(expr, scope);
        }
    }

    fn emit_expr(&mut self, expr: &'a Expr, scope: &mut Scope) {
        match expr {
            Expr::Boolean(true) => {
                scope.bytecode.add_op(ByteCode::PushTrue);
            }

            Expr::Boolean(false) => {
                scope.bytecode.add_op(ByteCode::PushFalse);
            }

            Expr::Integer(i) => {
                scope.bytecode.add_op(ByteCode::PushInteger).add_i32(i);
            }

            Expr::Float(f) => {
                scope.bytecode.add_op(ByteCode::PushFloat).add_f32(f);
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
                    scope
                        .bytecode
                        .add_op(ByteCode::GetForeignValue)
                        .add_string(lexeme);
                } else if let Some(index) = scope.environment.get(lexeme) {
                    scope.bytecode.add_get_local_value(*index);
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
                scope.bytecode.add_op(ByteCode::PushString).add_string(str);
            }

            Expr::Grouping(expr) => self.emit_expr(expr, scope),

            Expr::Block { exprs } => {
                // Emit block with its own environment and locals
                let locals = scope.locals.clone();
                let environment = scope.environment.clone();
                self.emit_exprs(exprs, scope);
                scope.locals = locals;
                scope.environment = environment;
            }

            Expr::Function {
                slash,
                params,
                expr,
            } => self.emit_function(slash, None, params, expr, scope),

            Expr::Call { name, args } => {
                let arg_count = args.len();
                self.emit_exprs(args, scope);

                if self.context.has_function(name) {
                    // TODO(anissen): Maybe this should be its own Expr instead?
                    scope
                        .bytecode
                        .add_op(ByteCode::CallForeign)
                        .add_byte(self.context.get_index(name))
                        .add_byte(arg_count as u8);
                } else {
                    scope
                        .bytecode
                        .add_op(ByteCode::Call)
                        .add_byte(arg_count as u8);
                    let index = scope.environment.get(name).unwrap();
                    if scope.locals.contains(name) {
                        scope.bytecode.add_byte(0);
                    } else {
                        scope.bytecode.add_byte(1);
                    }
                    scope.bytecode.add_byte(*index);
                };

                if name.len() > 255 {
                    panic!("function name too long!");
                    // let msg = Message::new(format!("Function name too long: {}", name), ;
                }
                scope.bytecode.add_string(name);
            }

            Expr::Assignment {
                name,
                _operator: _,
                expr,
            } => {
                self.emit_assignment(name, expr, scope);
            }

            Expr::Comparison { left, token, right } => {
                self.emit_expr(left, scope);
                self.emit_expr(right, scope);

                match token.kind {
                    TokenKind::EqualEqual => {
                        scope.bytecode.add_op(ByteCode::Equals);
                    }
                    TokenKind::BangEqual => {
                        scope
                            .bytecode
                            .add_op(ByteCode::Equals)
                            .add_op(ByteCode::Not);
                    }
                    TokenKind::LeftChevron => {
                        scope.bytecode.add_op(ByteCode::LessThan);
                    }
                    TokenKind::LeftChevronEqual => {
                        scope.bytecode.add_op(ByteCode::LessThanEquals);
                    }
                    TokenKind::RightChevron => {
                        scope
                            .bytecode
                            .add_op(ByteCode::LessThanEquals)
                            .add_op(ByteCode::Not);
                    }
                    TokenKind::RightChevronEqual => {
                        scope
                            .bytecode
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
                    self.emit_expr(expr, scope);
                    scope.bytecode.add_op(ByteCode::Negation);
                }
                UnaryOperator::Not => {
                    self.emit_expr(expr, scope);
                    scope.bytecode.add_op(ByteCode::Not);
                }
            },

            Expr::Binary {
                left,
                operator,
                _token: _,
                right,
            } => {
                self.emit_expr(left, scope);
                self.emit_expr(right, scope);
                match operator {
                    BinaryOperator::Addition => scope.bytecode.add_op(ByteCode::Addition),
                    BinaryOperator::Subtraction => scope.bytecode.add_op(ByteCode::Subtraction),
                    BinaryOperator::Multiplication => {
                        scope.bytecode.add_op(ByteCode::Multiplication)
                    }
                    BinaryOperator::Division => scope.bytecode.add_op(ByteCode::Division),
                    BinaryOperator::Modulus => scope.bytecode.add_op(ByteCode::Modulo),
                    BinaryOperator::StringConcat => scope.bytecode.add_op(ByteCode::StringConcat),
                    BinaryOperator::BooleanAnd => scope.bytecode.add_op(ByteCode::BooleanAnd),
                    BinaryOperator::BooleanOr => scope.bytecode.add_op(ByteCode::BooleanOr),
                };
            }

            Expr::Is { expr, arms } => {
                let index = match **expr {
                    Expr::Value { ref name } => {
                        // If the value is already in the environment, use its index
                        let index_option = scope.environment.get(&name.lexeme);
                        *index_option.unwrap()
                    }
                    _ => {
                        // Otherwise, emit the expression and add it to the locals
                        // to avoid emitting the same value multiple times
                        self.emit_expr(expr, scope);
                        let index = scope.locals.len() as u8;
                        scope.bytecode.add_set_local_value(index);
                        index
                    }
                };

                let mut jump_to_end_offsets = vec![];
                for arm in arms {
                    match &arm.pattern {
                        IsArmPattern::Expression(pattern) => {
                            // Emit expression and pattern and compare
                            scope.bytecode.add_get_local_value(index);
                            self.emit_expr(pattern, scope);
                            scope.bytecode.add_op(ByteCode::Equals);

                            // Jump to next arm if not equal
                            let next_arm_offset = scope.bytecode.add_jump_if_false();

                            // Otherwise execute arm block
                            self.emit_expr(&arm.block, scope);

                            // Jump to end of `is` block
                            let end_offset = scope.bytecode.add_unconditional_jump();
                            jump_to_end_offsets.push(end_offset);

                            // Patch jump to next arm now that we know its position
                            scope.bytecode.patch_jump_to_current_byte(next_arm_offset);
                        }

                        IsArmPattern::Capture {
                            identifier,
                            condition,
                        } => {
                            self.emit_assignment(identifier, expr, scope);

                            if let Some(condition) = condition {
                                // Emit expression and condition and compare
                                scope.bytecode.add_get_local_value(index);
                                self.emit_expr(condition, scope);

                                // Jump to next arm if not equal
                                let next_arm_offset = scope.bytecode.add_jump_if_false();

                                // Otherwise execute arm block
                                self.emit_expr(&arm.block, scope);

                                // Jump to end of `is` block
                                let end_offset = scope.bytecode.add_unconditional_jump();
                                jump_to_end_offsets.push(end_offset);

                                // Patch jump to next arm now that we know its position
                                scope.bytecode.patch_jump_to_current_byte(next_arm_offset);
                            } else {
                                // Otherwise execute arm block
                                self.emit_expr(&arm.block, scope);

                                // Jump to end of `is` block
                                let end_offset = scope.bytecode.add_unconditional_jump();
                                jump_to_end_offsets.push(end_offset);
                            }
                        }

                        IsArmPattern::Default => {
                            self.emit_expr(&arm.block, scope);
                        }
                    };
                }

                // Patch all jumps to end of `is` block now that we know where it ends
                for offset in jump_to_end_offsets {
                    scope.bytecode.patch_jump_to_current_byte(offset);
                }
            }
        };
    }

    fn emit_assignment(&mut self, name: &Token, expr: &'a Expr, scope: &mut Scope) {
        match expr {
            Expr::Function {
                slash,
                params,
                expr,
            } => {
                self.emit_function(slash, Some(name), params, expr, scope);
            }

            _ => {
                self.emit_expr(expr, scope);
            }
        }

        let index = scope.locals.len() as u8;
        scope.environment.insert(name.lexeme.clone(), index);
        scope.locals.insert(name.lexeme.clone());

        scope.bytecode.add_set_local_value(index);
    }

    fn emit_function(
        &mut self,
        slash: &'a Token,
        name: Option<&Token>,
        params: &[Token],
        body: &'a Expr,
        scope: &mut Scope,
    ) {
        if params.len() > u8::MAX.into() {
            panic!("Too many parameters");
        }

        scope.bytecode.add_op(ByteCode::Function);
        scope.bytecode.add_byte(self.function_chunks.len() as u8);
        scope.bytecode.add_byte(params.len() as u8);

        self.create_function_chunk(name, &slash.position, params, body, &mut scope.function());
    }

    fn create_function_chunk(
        &mut self,
        name: Option<&Token>,
        position: &'a Span,
        params: &[Token],
        body: &'a Expr,
        scope: &mut Scope,
    ) {
        if params.len() > u8::MAX.into() {
            panic!("Too many parameters");
        }

        if self.function_chunks.len() >= u8::MAX.into() {
            panic!("Too many functions");
        }

        let lexeme = match name {
            Some(name) => name.lexeme.clone(),
            None => "(unnamed)".to_string(),
        };

        let function_chunk_index = self.function_chunks.len();
        let function_chunk = FunctionChunk {
            function_name: lexeme.clone(),
            position,
            local_count: params.len() as u8,
            bytes: vec![],
        };
        self.function_chunks.push(function_chunk);

        scope
            .bytecode
            .add_op(ByteCode::FunctionChunk)
            .add_string(&lexeme);

        for (index, param) in params.iter().enumerate() {
            scope.environment.insert(param.lexeme.clone(), index as u8);
            scope.locals.insert(param.lexeme.clone());
        }

        // TODO(anissen): Expr is already a block, so we shouldn't need to create new environment and locals
        self.emit_expr(body, scope);

        scope.bytecode.add_op(ByteCode::Return);

        self.function_chunks[function_chunk_index].bytes = scope.bytecode.bytes.clone();
    }

    pub fn emit(&mut self, expressions: &'a Vec<Expr>) -> Vec<u8> {
        let mut scope = Scope::new();
        scope
            .bytecode
            .add_op(ByteCode::FunctionChunk)
            .add_string("main");

        self.emit_exprs(expressions, &mut scope);

        scope.bytecode.add_op(ByteCode::Return); // TODO(anissen): I may not need this, because I know the function bytecode length

        let mut signature_builder = BytecodeBuilder::new();
        let mut signature_patches = Vec::new();

        println!("Function chunks:");
        for ele in self.function_chunks.iter() {
            println!("{:?}", ele);
            let signature_offset = signature_builder
                .add_op(ByteCode::FunctionSignature)
                .add_string(&ele.function_name)
                .add_byte(ele.local_count)
                .get_patchable_i16_offset();
            signature_patches.push(signature_offset);
        }

        {
            let mut length = signature_builder.bytes.len() + scope.bytecode.bytes.len();
            for (index, ele) in self.function_chunks.iter().enumerate() {
                signature_builder.patch_i16_offset(signature_patches[index], length as isize);
                length += ele.bytes.len();
            }
        }

        let mut bytecode = vec![];
        bytecode.append(&mut signature_builder.bytes);
        bytecode.append(&mut scope.bytecode.bytes);
        for ele in self.function_chunks.iter() {
            bytecode.append(&mut ele.bytes.clone());
        }
        bytecode
    }
}

#[derive(Clone)]
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

    fn add_i32(&mut self, value: &i32) -> &mut Self {
        self.add_bytes(&value.to_be_bytes())
    }

    fn add_f32(&mut self, value: &f32) -> &mut Self {
        self.add_bytes(&value.to_be_bytes())
    }

    fn add_byte_array(&mut self, bytes: &[u8]) -> &mut Self {
        self.bytes.extend(bytes);
        self
    }

    fn add_string(&mut self, value: &str) -> &mut Self {
        self.add_byte(value.len() as u8)
            .add_byte_array(value.as_bytes())
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

    // TODO: Create a PatchableOffset for this
    // fn add_patchable_bytes(&mut self, bytes: u8) -> PatchableBytes {
    //     let offset = self.bytes.len();
    //     for byte in 0..bytes {
    //         self.add_byte(0u8);
    //     }
    //     PatchableBytes {
    //         offset,
    //         length: bytes,
    //     }
    // }

    // fn get_patchable_bytes(&mut self, index: u32, length: u8) -> PatchableBytes {
    //     PatchableBytes {
    //         index,
    //         length,
    //     }
    // }

    fn get_patchable_i16_offset(&mut self) -> usize {
        let bytes = 0_i16.to_be_bytes();
        self.add_bytes(&bytes /* placeholder */);
        self.bytes.len() - bytes.len()
    }

    fn patch_i16_offset(&mut self, patchable_bytes: usize, new_offset: isize) {
        // byte offset is the start of 2 bytes that indicate the jump offset
        if new_offset < i16::MIN as isize {
            panic!("New offset is too small");
        } else if new_offset > i16::MAX as isize {
            panic!("New offset is too large");
        }
        (new_offset as i16)
            .to_be_bytes()
            .swap_with_slice(&mut self.bytes[patchable_bytes..patchable_bytes + 2]);
    }

    fn add_set_local_value(&mut self, index: u8) -> &mut Self {
        self.add_op(ByteCode::SetLocalValue).add_byte(index)
    }

    fn add_get_local_value(&mut self, index: u8) -> &mut Self {
        self.add_op(ByteCode::GetLocalValue).add_byte(index)
    }

    fn patch_jump_to_current_byte(&mut self, byte_offset: usize) {
        // byte offset is the start of 2 bytes that indicate the jump offset
        let jump_instruction_bytes = 2;
        let jump_offset = (self.bytes.len() - (byte_offset + jump_instruction_bytes)) as isize;
        if jump_offset < i16::MIN as isize {
            panic!("Jump offset is too small");
        } else if jump_offset > i16::MAX as isize {
            panic!("Jump offset is too large");
        }
        (jump_offset as i16)
            .to_be_bytes()
            .swap_with_slice(&mut self.bytes[byte_offset..byte_offset + 2]);
    }
}
