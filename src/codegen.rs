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

    fn nested(&mut self) -> Self {
        Self {
            bytecode: self.bytecode.clone(),
            environment: self.environment.clone(),
            locals: self.locals.clone(),
        }
    }

    fn function(&mut self) -> Self {
        Self {
            bytecode: self.bytecode.clone(),
            environment: self.environment.clone(),
            locals: HashSet::new(),
        }
    }
}

pub struct Codegen<'a> {
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
            function_signatures: vec![],
            function_count: 0,
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
                        .add_byte(lexeme.len() as u8)
                        .add_byte_array(lexeme.as_bytes());
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
                scope
                    .bytecode
                    .add_op(ByteCode::PushString)
                    .add_byte(str.len() as u8)
                    .add_byte_array(str.as_bytes());
            }

            Expr::Grouping(expr) => self.emit_expr(expr, scope),

            Expr::Block { exprs } => {
                // Emit block with its own environment and locals
                self.emit_exprs(exprs, &mut scope.nested());
            }

            Expr::Function {
                slash,
                params,
                expr,
            } => self.emit_function(slash, None, params, expr, &mut scope.function()),

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
                scope
                    .bytecode
                    .add_byte(name.len() as u8)
                    .add_byte_array(name.as_bytes());
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
                self.emit_function(slash, Some(name), params, expr, &mut scope.function());
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

    // fn emit_set_local_value(&mut self, index: u8) {
    //     scope
    //         .bytecode
    //         .add_op(ByteCode::SetLocalValue)
    //         .add_byte(index);
    // }

    // fn emit_get_local_value(&mut self, index: u8) {
    //     scope
    //         .bytecode
    //         .add_op(ByteCode::GetLocalValue)
    //         .add_byte(index);
    // }

    fn emit_function(
        &mut self,
        slash: &'a Token,
        name: Option<&Token>,
        params: &[Token],
        expr: &'a Expr,
        scope: &mut Scope,
    ) {
        scope.bytecode.add_op(ByteCode::Function);
        self.function_signatures.push(FunctionSignature {
            byte_position: scope.bytecode.bytes.len() as u32 - 1,
        });

        self.create_function_chunk(name, &slash.position, params, expr, scope);

        for (index, param) in params.iter().enumerate() {
            scope.environment.insert(param.lexeme.clone(), index as u8);
            scope.locals.insert(param.lexeme.clone());
        }
        // bytecodes: function start, function index, param count, function body, function end

        scope.bytecode.add_byte(self.function_count);
        self.function_count += 1;

        scope.bytecode.add_byte(params.len() as u8); // TODO(anissen): Guard against overflow

        let jump_to_end = scope.bytecode.add_unconditional_jump();

        // TODO(anissen): Expr is already a block, so we shouldn't need to create new environment and locals
        self.emit_expr(expr, scope);

        scope.bytecode.add_op(ByteCode::Return);

        scope.bytecode.patch_jump_to_current_byte(jump_to_end);
    }

    fn create_function_chunk(
        &mut self,
        name: Option<&Token>,
        position: &'a Span,
        params: &[Token],
        expr: &Expr,
        scope: &mut Scope,
    ) {
        if params.len() > u8::MAX.into() {
            panic!("Too many parameters");
        }

        let lexeme = match name {
            Some(name) => name.lexeme.clone(),
            None => "(unnamed)".to_string(),
        };

        let mut function_environment = scope.environment.clone();
        let mut function_locals = HashSet::new();

        let byte_position = scope.bytecode.bytes.len() as u32 - 1;

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
        let mut scope = Scope::new();
        self.emit_exprs(expressions, &mut scope);

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

        [signature_builder.bytes, scope.bytecode.bytes.clone()].concat()
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
