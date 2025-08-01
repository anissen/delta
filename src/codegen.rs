use std::collections::{HashMap, HashSet};

use crate::bytecodes::ByteCode;
use crate::diagnostics::Diagnostics;
use crate::errors::Error;
use crate::expressions::{
    ArithmeticOperations, BinaryOperator, BooleanOperations, Comparisons, EqualityOperations, Expr,
    IsArmPattern, StringOperations, UnaryOperator, ValueType,
};
use crate::program::Context;
use crate::tokens::{Position, Token};

#[derive(Debug, Clone)]
struct FunctionChunk<'a> {
    local_count: u8,
    bytes: Vec<u8>,
    _position: &'a Position,
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
    diagnostics: Diagnostics,
}

pub fn codegen<'a>(expression: &'a Expr, context: &'a Context<'a>) -> Result<Vec<u8>, Diagnostics> {
    Codegen::new(context).emit(expression)
}

// TODO(anissen): Add a function overview mapping for each scope containing { name, arity, starting IP, source line number  }.
// This will be used directly in the VM as well as for debug logging.

impl<'a> Codegen<'a> {
    fn new(context: &'a Context<'a>) -> Self {
        Self {
            function_chunks: vec![],
            context,
            diagnostics: Diagnostics::new(),
        }
    }

    fn emit_exprs(&mut self, expressions: &'a Vec<Expr>, scope: &mut Scope) {
        for expr in expressions {
            self.emit_expr(expr, scope);
        }
    }

    // TODO(anissen): Should this be a method on scope instead?
    fn emit_expr(&mut self, expr: &'a Expr, scope: &mut Scope) {
        match expr {
            Expr::Value {
                value: ValueType::Boolean(true),
                token: _,
            } => {
                scope.bytecode.add_op(ByteCode::PushTrue);
            }

            Expr::Value {
                value: ValueType::Boolean(false),
                token: _,
            } => {
                scope.bytecode.add_op(ByteCode::PushFalse);
            }

            Expr::Value {
                value: ValueType::Integer(i),
                token: _,
            } => {
                scope.bytecode.add_op(ByteCode::PushInteger).add_i32(i);
            }

            Expr::Value {
                value: ValueType::Float(f),
                token: _,
            } => {
                scope.bytecode.add_op(ByteCode::PushFloat).add_f32(f);
            }

            Expr::Identifier { name } => {
                let lexeme = &name.lexeme;
                if self.context.has_value(lexeme) {
                    // TODO(anissen): Should (also) output index
                    if lexeme.len() > 255 {
                        self.diagnostics.add_error(Error::FunctionNameTooLong {
                            token: name.clone(),
                        });
                    }
                    scope
                        .bytecode
                        .add_op(ByteCode::GetForeignValue)
                        .add_string(lexeme);
                } else if let Some(index) = scope.environment.get(lexeme) {
                    scope.bytecode.add_get_local_value(*index);
                } else {
                    panic!("Name not found in scope");
                }
            }

            Expr::Value {
                value: ValueType::String(str),
                token: _,
            } => {
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

            Expr::Value {
                value: ValueType::Function { params, expr },
                token,
            } => self.emit_function(token, None, params, expr, scope),

            Expr::Value {
                value: ValueType::Tag { name, payload },
                token,
            } => {
                if name.lexeme.len() > 255 {
                    panic!("string too long!");
                }
                if let Some(payload) = &**payload {
                    self.emit_expr(payload, scope);
                    scope
                        .bytecode
                        .add_op(ByteCode::PushTag)
                        .add_string(&name.lexeme);
                } else {
                    scope
                        .bytecode
                        .add_op(ByteCode::PushSimpleTag)
                        .add_string(&name.lexeme);
                };
            }

            Expr::Call { name, args } => {
                let lexeme = &name.lexeme;
                let arg_count = args.len();
                self.emit_exprs(args, scope);

                if self.context.has_function(lexeme) {
                    // TODO(anissen): Maybe this should be its own Expr instead?
                    scope
                        .bytecode
                        .add_op(ByteCode::CallForeign)
                        .add_byte(self.context.get_index(lexeme))
                        .add_byte(arg_count as u8);
                } else {
                    match scope.environment.get(lexeme) {
                        Some(index) => {
                            scope
                                .bytecode
                                .add_op(ByteCode::Call)
                                .add_byte(arg_count as u8);
                            if scope.locals.contains(lexeme) {
                                scope.bytecode.add_byte(0);
                            } else {
                                scope.bytecode.add_byte(1);
                            }
                            scope.bytecode.add_byte(*index);
                        }
                        None => {
                            panic!("Unknown function");
                        }
                    }
                };

                if lexeme.len() > 255 {
                    panic!("function name too long!");
                    // let msg = Message::new(format!("Function name too long: {}", name), ;
                }
                scope.bytecode.add_string(lexeme);
            }

            Expr::Assignment {
                name,
                _operator: _,
                expr,
            } => {
                self.emit_assignment(name, expr, scope);
            }

            Expr::Unary {
                operator,
                token: _,
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
                token: _,
                right,
            } => {
                self.emit_expr(left, scope);
                self.emit_expr(right, scope);
                match operator {
                    BinaryOperator::IntegerOperation(integer_operation) => {
                        match integer_operation {
                            ArithmeticOperations::Addition => {
                                scope.bytecode.add_op(ByteCode::IntegerAddition)
                            }
                            ArithmeticOperations::Subtraction => {
                                scope.bytecode.add_op(ByteCode::IntegerSubtraction)
                            }
                            ArithmeticOperations::Multiplication => {
                                scope.bytecode.add_op(ByteCode::IntegerMultiplication)
                            }
                            ArithmeticOperations::Division => {
                                scope.bytecode.add_op(ByteCode::IntegerDivision)
                            }
                            ArithmeticOperations::Modulus => {
                                scope.bytecode.add_op(ByteCode::IntegerModulo)
                            }
                        }
                    }
                    BinaryOperator::FloatOperation(float_operation) => match float_operation {
                        ArithmeticOperations::Addition => {
                            scope.bytecode.add_op(ByteCode::FloatAddition)
                        }
                        ArithmeticOperations::Subtraction => {
                            scope.bytecode.add_op(ByteCode::FloatSubtraction)
                        }
                        ArithmeticOperations::Multiplication => {
                            scope.bytecode.add_op(ByteCode::FloatMultiplication)
                        }
                        ArithmeticOperations::Division => {
                            scope.bytecode.add_op(ByteCode::FloatDivision)
                        }
                        ArithmeticOperations::Modulus => {
                            scope.bytecode.add_op(ByteCode::FloatModulo)
                        }
                    },
                    BinaryOperator::BooleanOperation(boolean_operation) => {
                        match boolean_operation {
                            BooleanOperations::And => scope.bytecode.add_op(ByteCode::BooleanAnd),
                            BooleanOperations::Or => scope.bytecode.add_op(ByteCode::BooleanOr),
                        }
                    }
                    BinaryOperator::StringOperation(string_operation) => match string_operation {
                        StringOperations::StringConcat => {
                            scope.bytecode.add_op(ByteCode::StringConcat)
                        }
                    },
                    BinaryOperator::IntegerComparison(integer_comparison) => {
                        match integer_comparison {
                            Comparisons::LessThan => {
                                scope.bytecode.add_op(ByteCode::IntegerLessThan)
                            }
                            Comparisons::LessThanEqual => {
                                scope.bytecode.add_op(ByteCode::IntegerLessThanEquals)
                            }
                            Comparisons::GreaterThan => scope
                                .bytecode
                                .add_op(ByteCode::IntegerLessThanEquals)
                                .add_op(ByteCode::Not),
                            Comparisons::GreaterThanEqual => scope
                                .bytecode
                                .add_op(ByteCode::IntegerLessThan)
                                .add_op(ByteCode::Not),
                        }
                    }
                    BinaryOperator::FloatComparison(float_comparison) => match float_comparison {
                        Comparisons::LessThan => scope.bytecode.add_op(ByteCode::FloatLessThan),
                        Comparisons::LessThanEqual => {
                            scope.bytecode.add_op(ByteCode::FloatLessThanEquals)
                        }
                        Comparisons::GreaterThan => scope
                            .bytecode
                            .add_op(ByteCode::FloatLessThanEquals)
                            .add_op(ByteCode::Not),
                        Comparisons::GreaterThanEqual => scope
                            .bytecode
                            .add_op(ByteCode::FloatLessThan)
                            .add_op(ByteCode::Not),
                    },
                    BinaryOperator::Equality(equality) => match equality {
                        EqualityOperations::Equal => scope.bytecode.add_op(ByteCode::Equals),
                        EqualityOperations::NotEqual => scope
                            .bytecode
                            .add_op(ByteCode::Equals)
                            .add_op(ByteCode::Not),
                    },
                };
            }

            Expr::Is { expr, arms } => {
                let index = match **expr {
                    Expr::Identifier { ref name } => {
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

                let locals_count = scope.locals.len() as u8;

                let mut jump_to_end_offsets = vec![];

                for (arm_index, arm) in arms.iter().enumerate() {
                    let is_last_arm = arm_index == arms.len() - 1;

                    // Handle pattern matching logic
                    let mut pattern_jump_offsets = vec![];

                    match &arm.pattern {
                        IsArmPattern::Expression(pattern) => {
                            // Emit expression and pattern and compare
                            scope.bytecode.add_get_local_value(index);
                            self.emit_expr(pattern, scope);
                            scope.bytecode.add_op(ByteCode::Equals);

                            // Jump to next arm if not equal
                            let next_arm_offset = scope.bytecode.add_jump_if_false();
                            pattern_jump_offsets.push(next_arm_offset);
                        }
                        IsArmPattern::Capture { identifier } => {
                            self.emit_assignment(identifier, expr, scope);
                        }
                        IsArmPattern::CaptureTagPayload { expr, identifier } => {
                            let tag_name = match expr {
                                Expr::Value {
                                    value: ValueType::Tag { name, payload },
                                    token,
                                } => name.lexeme.clone(),
                                _ => unreachable!(),
                            };

                            scope
                                .bytecode
                                .add_op(ByteCode::GetTagName)
                                .add_op(ByteCode::PushString)
                                .add_string(&tag_name)
                                .add_op(ByteCode::Equals);

                            // Jump to next arm if not equal
                            let next_arm_offset = scope.bytecode.add_jump_if_false();
                            pattern_jump_offsets.push(next_arm_offset);

                            scope.bytecode.add_op(ByteCode::GetTagPayload);

                            // TODO: Is this right?!?
                            scope
                                .environment
                                .insert(identifier.lexeme.clone(), locals_count);
                            scope.bytecode.add_set_local_value(locals_count);
                        }
                        IsArmPattern::Default => {
                            // No pattern matching needed for default case
                        }
                    }

                    // Handle guard condition if present
                    if let Some(guard) = &arm.guard {
                        // Check if-guard
                        self.emit_expr(&guard.condition, scope);
                        let guard_jump_offset = scope.bytecode.add_jump_if_false();
                        pattern_jump_offsets.push(guard_jump_offset);
                    }

                    // Execute arm block
                    self.emit_expr(&arm.block, scope);

                    if !is_last_arm {
                        // Jump to end of `is` block
                        let end_offset = scope.bytecode.add_unconditional_jump();
                        jump_to_end_offsets.push(end_offset);
                    }

                    // Patch all jumps to next arm now that we know the position
                    for offset in pattern_jump_offsets {
                        scope.bytecode.patch_jump_to_current_byte(offset);
                    }
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
            Expr::Value {
                value: ValueType::Function { params, expr },
                token,
            } => {
                // save function name to environment before entering function definition
                let index = scope.locals.len() as u8;
                scope.environment.insert(name.lexeme.clone(), index);
                scope.locals.insert(name.lexeme.clone());

                self.emit_function(token, Some(name), params, expr, scope);
                scope.bytecode.add_set_local_value(index);
            }

            _ => {
                self.emit_expr(expr, scope);

                let index = scope.locals.len() as u8;
                scope.environment.insert(name.lexeme.clone(), index);
                scope.locals.insert(name.lexeme.clone());
                scope.bytecode.add_set_local_value(index);
            }
        }
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
        position: &'a Position,
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
            _position: position,
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

    pub fn emit(&mut self, expression: &'a Expr) -> Result<Vec<u8>, Diagnostics> {
        let mut scope = Scope::new();
        scope
            .bytecode
            .add_op(ByteCode::FunctionChunk)
            .add_string("main");

        self.emit_expr(expression, &mut scope);
        scope.bytecode.add_op(ByteCode::Return); // TODO(anissen): I may not need this, because I know the function bytecode length

        if !self.diagnostics.has_errors() {
            Ok(self.create_bytecode(&mut scope))
        } else {
            Err(self.diagnostics.clone())
        }
    }

    fn create_bytecode(&mut self, scope: &mut Scope) -> Vec<u8> {
        let mut signature_builder = BytecodeBuilder::new();
        let mut signature_patches = Vec::new();

        // println!("Function chunks:");
        for ele in self.function_chunks.iter() {
            // println!("{:?}", ele);
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
