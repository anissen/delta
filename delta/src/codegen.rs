use std::collections::{HashMap, HashSet};

use crate::bytecodes::ByteCode;
use crate::diagnostics::Diagnostics;
use crate::errors::Error;
use crate::expressions::{
    ArithmeticOperations, BinaryOperator, BooleanOperations, Comparisons, EqualityOperations, Expr,
    IsArm, IsArmPattern, MaybeNamedType, PropertyDefinition, StringOperations, UnaryOperator,
    ValueType,
};
use crate::program::Context;
use crate::tokens::{Position, Token, TokenKind};
use crate::unification::Type;

#[derive(Debug, Clone)]
struct FunctionChunk<'a> {
    local_count: u8,
    bytes: Vec<u8>,
    _position: &'a Position,
    function_name: String,
}

#[derive(Debug)]
pub struct Scope {
    bytecode: BytecodeBuilder,
    environment: HashMap<String, u8>,
    locals: HashSet<String>,

    /// Mapping local component variables to their component types (e.g. "p" => "Position")
    local_component_mapping: HashMap<String, Token>,
}

impl Scope {
    fn new() -> Self {
        Self {
            bytecode: BytecodeBuilder::new(),
            environment: HashMap::new(),
            locals: HashSet::new(),
            local_component_mapping: HashMap::new(),
        }
    }

    fn function(&mut self) -> Self {
        Self {
            bytecode: BytecodeBuilder::new(),
            environment: self.environment.clone(),
            locals: HashSet::new(),
            local_component_mapping: self.local_component_mapping.clone(),
        }
    }
}

#[derive(Debug)]
struct ComponentMetadata<'a> {
    id: u8,
    properties: &'a Vec<crate::expressions::PropertyDefinition>,
}

pub struct Codegen<'a> {
    function_chunks: Vec<FunctionChunk<'a>>,
    context: &'a Context<'a>,
    components: HashMap<String, ComponentMetadata<'a>>,
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
            components: HashMap::new(),
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
            Expr::Value { value, token } => self.emit_value(value, token, scope),

            Expr::Grouping(expr) => self.emit_expr(expr, scope),

            Expr::Block { exprs } => {
                // Emit block with its own environment and locals
                let locals = scope.locals.clone();
                let environment = scope.environment.clone();
                self.emit_exprs(exprs, scope);
                scope.locals = locals;
                scope.environment = environment;
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

            Expr::ContextIdentifier { context: _, name } => {
                scope.bytecode.add_get_context_value(&name.lexeme);
            }

            Expr::FieldAccess {
                identifier,
                field_name,
            } => {
                let lexeme = &identifier.lexeme;
                if let Some(index) = scope.environment.get(lexeme) {
                    let field_access_index =
                        self.get_component_field_index(scope, identifier, field_name);
                    scope
                        .bytecode
                        .add_get_field_value(*index, field_access_index as u8);
                } else {
                    dbg!(&identifier);
                    panic!("Name not found in scope");
                }
            }

            Expr::Context { name: _ } => {
                todo!("Implement context expression")
            }

            Expr::ComponentDefinition { name, properties } => {
                self.components.insert(
                    name.lexeme.clone(),
                    ComponentMetadata {
                        id: self.components.len() as u8,
                        properties,
                    },
                );
            }

            Expr::Call { name, args } => {
                let lexeme = &name.lexeme;
                let arg_count = args.len();
                self.emit_exprs(args, scope);

                // TODO(anissen): Hack
                if lexeme == "get_list_element_at_index" {
                    scope.bytecode.add_op(ByteCode::GetListElementAtIndex);
                } else if lexeme == "get_array_length" {
                    scope.bytecode.add_op(ByteCode::GetArrayLength);
                } else if lexeme == "append" {
                    scope.bytecode.add_op(ByteCode::ArrayAppend);
                } else if lexeme == "log" {
                    scope.bytecode.add_op(ByteCode::Log);
                } else {
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
                };
            }

            Expr::Assignment {
                target,
                _operator: _,
                expr,
            } => match **target {
                Expr::Identifier { ref name } => self.emit_assignment(name, expr, scope),
                Expr::ContextIdentifier {
                    context: _,
                    ref name,
                } => {
                    self.emit_expr(expr, scope);
                    scope.bytecode.add_set_context_value(&name.lexeme);
                }
                Expr::FieldAccess {
                    ref identifier,
                    ref field_name,
                } => {
                    self.emit_expr(expr, scope);
                    let index = scope.environment.get(&identifier.lexeme).unwrap();
                    let field_index = self.get_component_field_index(scope, identifier, field_name);
                    scope
                        .bytecode
                        .add_op(ByteCode::SetFieldValue)
                        .add_byte(*index)
                        .add_byte(field_index as u8);
                }

                _ => panic!("Invalid assignment target"),
            },

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
            } => self.emit_binary(left, operator, right, scope),

            Expr::Is { expr, arms } => self.emit_is(expr, arms, scope),

            Expr::Query {
                include_components: include_compoments,
                exclude_components: exclude_compoments,
                expr,
            } => self.emit_query(include_compoments, exclude_compoments, expr, scope),

            Expr::Create {
                token: _,
                arguments,
            } => {
                self.emit_expr(arguments, scope);
                scope.bytecode.add_op(ByteCode::Create);
            }

            Expr::Destroy { token: _, argument } => {
                self.emit_expr(argument, scope);
                scope.bytecode.add_op(ByteCode::Destroy);
            }
        };
    }

    fn get_component_field_index(
        &mut self,
        scope: &Scope,
        component_identifier: &Token,
        field_name: &Token,
    ) -> usize {
        let component_name = scope
            .local_component_mapping
            .get(&component_identifier.lexeme)
            .unwrap();
        let component_properties = self.components.get(&component_name.lexeme).unwrap();

        component_properties
            .properties
            .iter()
            .enumerate()
            .find(|(_, field)| field.name.lexeme == field_name.lexeme)
            .map(|field_index| field_index.0)
            .unwrap()
    }

    fn emit_value(&mut self, value: &'a ValueType, token: &'a Token, scope: &mut Scope) {
        match value {
            ValueType::Boolean(true) => {
                scope.bytecode.add_op(ByteCode::PushTrue);
            }

            ValueType::Boolean(false) => {
                scope.bytecode.add_op(ByteCode::PushFalse);
            }

            ValueType::Integer(i) => {
                scope.bytecode.add_op(ByteCode::PushInteger).add_i32(i);
            }

            ValueType::Float(f) => {
                scope.bytecode.add_op(ByteCode::PushFloat).add_f32(f);
            }

            ValueType::List(exprs) => {
                self.emit_exprs(exprs, scope);
                scope
                    .bytecode
                    .add_op(ByteCode::PushList)
                    .add_i32(&(exprs.len() as i32));
            }

            ValueType::String(str) => {
                if str.len() > 255 {
                    // TODO(anissen): Should add error to a error reporter instead
                    panic!("string too long!");
                }
                scope.bytecode.add_op(ByteCode::PushString).add_string(str);
            }

            ValueType::Function { params, expr } => {
                self.emit_function(token, None, params, expr, scope)
            }

            ValueType::Tag { name, payload } => {
                if name.lexeme.len() > 255 {
                    panic!("string too long!");
                }
                if let Some(payload) = payload {
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

            ValueType::Component { name, properties } => {
                // Output property values in sorted order wrt. the definition
                let component_properties = self.components.get(&name.lexeme).unwrap();
                component_properties
                    .properties
                    .iter()
                    .for_each(|property_definition| {
                        let property = properties
                            .iter()
                            .find(|p| p.name.lexeme == property_definition.name.lexeme)
                            .unwrap();
                        self.emit_expr(&property.value, scope);
                    });

                let component_id = self.components.get(&name.lexeme).unwrap().id;
                scope
                    .bytecode
                    .add_op(ByteCode::PushComponent)
                    .add_byte(component_id)
                    .add_byte(properties.len() as u8);
            }
        }
    }

    fn emit_binary(
        &mut self,
        left: &'a Expr,
        operator: &BinaryOperator,
        right: &'a Expr,
        scope: &mut Scope,
    ) {
        self.emit_expr(left, scope);
        self.emit_expr(right, scope);
        match operator {
            BinaryOperator::IntegerOperation(integer_operation) => match integer_operation {
                ArithmeticOperations::Addition => scope.bytecode.add_op(ByteCode::IntegerAddition),
                ArithmeticOperations::Subtraction => {
                    scope.bytecode.add_op(ByteCode::IntegerSubtraction)
                }
                ArithmeticOperations::Multiplication => {
                    scope.bytecode.add_op(ByteCode::IntegerMultiplication)
                }
                ArithmeticOperations::Division => scope.bytecode.add_op(ByteCode::IntegerDivision),
                ArithmeticOperations::Modulus => scope.bytecode.add_op(ByteCode::IntegerModulo),
            },
            BinaryOperator::FloatOperation(float_operation) => match float_operation {
                ArithmeticOperations::Addition => scope.bytecode.add_op(ByteCode::FloatAddition),
                ArithmeticOperations::Subtraction => {
                    scope.bytecode.add_op(ByteCode::FloatSubtraction)
                }
                ArithmeticOperations::Multiplication => {
                    scope.bytecode.add_op(ByteCode::FloatMultiplication)
                }
                ArithmeticOperations::Division => scope.bytecode.add_op(ByteCode::FloatDivision),
                ArithmeticOperations::Modulus => scope.bytecode.add_op(ByteCode::FloatModulo),
            },
            BinaryOperator::BooleanOperation(boolean_operation) => match boolean_operation {
                BooleanOperations::And => scope.bytecode.add_op(ByteCode::BooleanAnd),
                BooleanOperations::Or => scope.bytecode.add_op(ByteCode::BooleanOr),
            },
            BinaryOperator::StringOperation(string_operation) => match string_operation {
                StringOperations::StringConcat => scope.bytecode.add_op(ByteCode::StringConcat),
            },
            BinaryOperator::IntegerComparison(integer_comparison) => match integer_comparison {
                Comparisons::LessThan => scope.bytecode.add_op(ByteCode::IntegerLessThan),
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
            },
            BinaryOperator::FloatComparison(float_comparison) => match float_comparison {
                Comparisons::LessThan => scope.bytecode.add_op(ByteCode::FloatLessThan),
                Comparisons::LessThanEqual => scope.bytecode.add_op(ByteCode::FloatLessThanEquals),
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

    fn emit_is(&mut self, expr: &'a Expr, arms: &'a Vec<IsArm>, scope: &mut Scope) {
        let index = match *expr {
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
                    scope.environment.insert(identifier.lexeme.clone(), index);
                    scope.locals.insert(identifier.lexeme.clone());
                }
                IsArmPattern::CaptureTagPayload {
                    tag_name,
                    identifier,
                } => {
                    scope
                        .bytecode
                        .add_op(ByteCode::GetTagName)
                        .add_op(ByteCode::PushString)
                        .add_string(&tag_name.lexeme)
                        .add_op(ByteCode::Equals);

                    // Jump to next arm if not equal
                    let next_arm_offset = scope.bytecode.add_jump_if_false();
                    pattern_jump_offsets.push(next_arm_offset);

                    scope.bytecode.add_op(ByteCode::GetTagPayload);

                    // TODO: Is this right?!?
                    scope
                        .environment
                        .insert(identifier.lexeme.clone(), locals_count);
                    // scope.locals.insert(identifier.lexeme.clone());
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

    fn emit_query(
        &mut self,
        include_components: &Vec<MaybeNamedType>,
        exclude_components: &Vec<Token>,
        expr: &'a Expr,
        scope: &mut Scope,
    ) {
        let query_end_offset = scope
            .bytecode
            .add_op(ByteCode::ContextQuery)
            .get_patchable_i16_offset();

        let (entity_components, include_components): (Vec<&MaybeNamedType>, Vec<&MaybeNamedType>) =
            include_components
                .iter()
                .partition(|component| component.type_.lexeme == "Entity");

        scope.bytecode.add_byte(include_components.len() as u8);
        scope.bytecode.add_byte(exclude_components.len() as u8);

        let old_environment = scope.environment.clone();
        let old_locals = scope.locals.clone();
        scope.locals.clear();
        let old_local_component_mapping = scope.local_component_mapping.clone();

        let mut sorted_includes = include_components.iter().collect::<Vec<_>>();
        sorted_includes.sort_by(|a, b| {
            let component_id_a = self.components.get(&a.type_.lexeme).unwrap().id;
            let component_id_b = self.components.get(&b.type_.lexeme).unwrap().id;

            component_id_a.cmp(&component_id_b)
        });

        let mut component_variable_index = 1; // Entity always exists at index 0

        if let Some(entity_component) = entity_components.first() {
            if let Some(ref name) = entity_component.name {
                let lexeme = name.lexeme.clone();
                scope.environment.insert(lexeme.clone(), 0);
                scope.locals.insert(lexeme.clone());
                scope
                    .local_component_mapping
                    .insert(lexeme.clone(), entity_component.type_.clone());
            }
        }

        sorted_includes.iter().for_each(|component| {
            let component_type_name = component.type_.lexeme.clone();

            let component_id = self.components.get(&component_type_name).unwrap().id;
            scope
                .bytecode
                .add_byte(component_id)
                .add_string(&component_type_name);

            if let Some(ref name) = component.name {
                let lexeme = name.lexeme.clone();
                scope
                    .environment
                    .insert(lexeme.clone(), component_variable_index);
                component_variable_index += 1;
                scope.locals.insert(lexeme.clone());
                scope
                    .local_component_mapping
                    .insert(lexeme.clone(), component.type_.clone());
            }
        });

        let mut sorted_excludes = exclude_components.clone();
        sorted_excludes.sort_by(|a, b| {
            let component_id_a = self.components.get(&a.lexeme).unwrap().id;
            let component_id_b = self.components.get(&b.lexeme).unwrap().id;

            component_id_a.cmp(&component_id_b)
        });

        sorted_excludes.iter().for_each(|component| {
            let component_type_name = component.lexeme.clone();

            let component_id = self.components.get(&component_type_name).unwrap().id;
            scope
                .bytecode
                .add_byte(component_id)
                .add_string(&component_type_name);
        });

        /*
        context_query (queries components and jumps to :end if empty, pushes query frame otherwise)
        :start
        set_next_component_column_or_jump (set/replaces components or pops query frame when done)
        [expr]
        jump_to_label start
        :end
        */

        let start_label = scope.bytecode.bytes.len() as i16;

        scope
            .bytecode
            .add_op(ByteCode::SetNextComponentColumnOrJump);

        self.emit_expr(expr, scope);
        scope.locals = old_locals;
        scope.environment = old_environment;
        scope.local_component_mapping = old_local_component_mapping;

        // Unconditional jump to start label
        scope.bytecode.add_op(ByteCode::Jump);
        let start_offset =
            start_label - (scope.bytecode.bytes.len() as i16 + 2/* jump offset bytes */);
        scope.bytecode.add_i16(&start_offset);

        scope.bytecode.patch_jump_to_current_byte(query_end_offset);
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

        scope
            .bytecode
            .add_op(ByteCode::Function)
            .add_byte(self.function_chunks.len() as u8)
            .add_byte(params.len() as u8);

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
        let mut header_builder = BytecodeBuilder::new();

        let mut sorted_map = self.components.values().collect::<Vec<_>>();
        sorted_map.sort_by(|a, b| a.id.cmp(&b.id));

        header_builder.add_byte(self.components.len() as u8);
        for component_metadata in &sorted_map {
            header_builder.add_byte(component_metadata.id);
            header_builder.add_byte(component_metadata.properties.len() as u8);
            for property in component_metadata.properties.iter() {
                header_builder.add_string(&property.name.lexeme);
                let type_id: u8 = match property.type_ {
                    // TODO(anissen): Implement this convertion on `Type`
                    Type::Boolean => 0,
                    Type::Integer => 1,
                    Type::Float => 2,
                    Type::String => 3,
                    _ => panic!("Unknown type"),
                };
                header_builder.add_byte(type_id);
                let size: u16 = match property.type_ {
                    // TODO(anissen): Implement this method on `Type`
                    Type::Boolean => 1,
                    Type::Integer => 4,
                    Type::Float => 4,
                    Type::String => 1 /* length of string */ + 32, /* fixed max size of string */ // TODO(anissen): Make this size dynamic
                    _ => panic!("Unknown/unhandled type"),
                };
                header_builder.add_u16(&size);
            }
        }

        let mut signature_patches = Vec::new();
        // println!("Function chunks:");
        for ele in self.function_chunks.iter() {
            // println!("{:?}", ele);
            let signature_offset = header_builder
                .add_op(ByteCode::FunctionSignature)
                .add_string(&ele.function_name)
                .add_byte(ele.local_count)
                .get_patchable_i16_offset();
            signature_patches.push(signature_offset);
        }

        {
            let mut length = header_builder.bytes.len() + scope.bytecode.bytes.len();
            for (index, ele) in self.function_chunks.iter().enumerate() {
                header_builder.patch_i16_offset(signature_patches[index], length as isize);
                length += ele.bytes.len();
            }
        }

        let mut bytecode = vec![];
        bytecode.append(&mut header_builder.bytes);
        bytecode.append(&mut scope.bytecode.bytes);
        for ele in self.function_chunks.iter() {
            bytecode.append(&mut ele.bytes.clone());
        }
        bytecode
    }
}

#[derive(Debug, Clone)]
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

    fn add_i16(&mut self, value: &i16) -> &mut Self {
        self.add_bytes(&value.to_be_bytes())
    }

    fn add_u16(&mut self, value: &u16) -> &mut Self {
        self.add_bytes(&value.to_be_bytes())
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

    fn add_get_field_value(&mut self, index: u8, field_index: u8) -> &mut Self {
        self.add_op(ByteCode::GetFieldValue)
            .add_byte(index)
            .add_byte(field_index)
    }

    fn add_get_context_value(&mut self, name: &str) -> &mut Self {
        self.add_op(ByteCode::GetContextValue).add_string(name)
    }

    fn add_set_context_value(&mut self, name: &str) -> &mut Self {
        self.add_op(ByteCode::SetContextValue).add_string(name)
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
