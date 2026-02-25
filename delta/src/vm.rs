use std::collections::HashMap;
use std::fmt::Display;

use crate::ExecutionMetadata;
use crate::bytecodes::ByteCode;
use crate::program::Context;
use crate::program::PersistentData;

use elements::ComponentLayout;
use elements::Entity;
use elements::FieldLayout;
use elements::world::QueryResult;

// TODO(anissen): See https://github.com/brightly-salty/rox/blob/master/src/value.rs
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    True,
    False,
    Integer(i32),
    Float(f32),
    String(String),
    SimpleTag(String),
    Tag(String, Box<Value>),
    List(Vec<Value>),
    Function(u8),
    Component { id: u8, properties: Vec<Value> },
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::True => write!(f, "true")?,
            Value::False => write!(f, "false")?,
            Value::Integer(i) => write!(f, "{i}")?,
            Value::Float(d) => write!(f, "{d:.2}")?,
            Value::String(s) => write!(f, "{s}")?,
            Value::SimpleTag(t) => write!(f, "{t}")?,
            Value::Tag(t, a) => write!(f, "{t}({a})")?,
            Value::List(l) => {
                let mut first = true;
                write!(f, "[")?;
                for v in l {
                    if !first {
                        write!(f, ", ")?;
                    }
                    write!(f, "{v}")?;
                    first = false;
                }
                write!(f, "]")?;
            }
            Value::Function(i) => write!(f, "<fn {i}>")?,
            Value::Component { id, properties } => {
                let properties_str = properties
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "component({id}, {properties_str})")?;
            }
        };
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct FunctionObj {
    name: String,
    ip: u32,
}

#[derive(Debug)]
struct CallFrame {
    return_program_counter: usize,
    stack_index: u8,
}

pub struct VirtualMachine {
    program: Vec<u8>,
    program_counter: usize,
    main_chunk_program_counter: usize,
    functions: Vec<FunctionObj>,
    stack: Vec<Value>,
    call_stack: Vec<CallFrame>,
    verbose: bool,
    pub metadata: ExecutionMetadata,
}

pub fn run<'a>(
    bytes: Vec<u8>,
    function_name: Option<(String, Vec<Value>)>,
    context: &'a Context<'a>,
    data: &mut PersistentData,
    verbose: bool,
) -> Option<Value> {
    VirtualMachine::new(bytes, data, verbose).execute(function_name, context, data)
}

static EMPTY_VALUE: Value = Value::False; // Only used when a function returns no result

impl VirtualMachine {
    pub fn new(bytes: Vec<u8>, data: &mut PersistentData, verbose: bool) -> Self {
        let mut vm = Self {
            program: bytes,
            program_counter: 0,
            functions: Vec::new(),
            stack: Vec::new(),
            call_stack: Vec::new(),
            verbose,
            main_chunk_program_counter: 0,
            metadata: ExecutionMetadata::default(),
        };
        vm.read_header(data);
        vm
    }

    pub fn update_bytecode(&mut self, bytes: Vec<u8>, data: &mut PersistentData) {
        self.program = bytes;
        self.functions.clear();
        self.stack.clear();
        self.call_stack.clear();
        self.read_header(data);
    }

    fn read_header(&mut self, data: &mut PersistentData) {
        // TODO(anissen): Read bytecode header here

        self.read_component_data(data);
        self.read_functions();

        self.main_chunk_program_counter = self.program_counter;
    }

    fn read_component_data(&mut self, data: &mut PersistentData) {
        // TODO(anissen): Check that the new components matches the old

        let component_count = self.read_byte();
        for _ in 0..component_count {
            let id = self.read_byte();
            let field_count = self.read_byte();
            let mut fields = Vec::with_capacity(field_count as usize);
            for _ in 0..field_count {
                let name = self.read_string();
                let type_id = self.read_byte();
                let size = self.read_u16();
                fields.push(FieldLayout {
                    name,
                    type_id,
                    size,
                });
            }

            data.elements
                .world
                .register_component(id as u32, ComponentLayout::new(fields));
        }
    }

    fn get_next_bytecode(&mut self) -> Result<ByteCode, ()> {
        if self.program_counter < self.program.len() {
            ByteCode::try_from(self.read_byte())
        } else {
            Err(())
        }
    }

    fn read_functions(&mut self) {
        while let Ok(ByteCode::FunctionSignature) = self.get_next_bytecode() {
            let name = self.read_string();
            let _local_count = self.read_byte();
            let function_position = self.read_i16();

            self.functions.push(FunctionObj {
                name,
                ip: function_position as u32,
            });
        }
    }

    pub fn execute(
        &mut self,
        function: Option<(String, Vec<Value>)>,
        context: &Context,
        data: &mut PersistentData,
    ) -> Option<Value> {
        self.program_counter = self.main_chunk_program_counter;

        if self.program_counter >= self.program.len() {
            return None;
        }

        let main_start = self.program_counter - 1;

        // Construct an initial call frame for the top-level code.
        self.program_counter = self.program.len(); // Set return IP to EOF.

        if let Some((function_name, args)) = function {
            let arity = args.len() as u8;
            self.stack = args;
            let function_to_execute = self
                .functions
                .iter()
                .find(|f| f.name == function_name)
                .unwrap()
                .clone();
            self.call(function_to_execute, arity);
        } else {
            self.call(
                FunctionObj {
                    name: "<main>".to_string(),
                    ip: main_start as u32,
                },
                0,
            );
        }

        let mut query_results: Option<QueryResult> = None;
        let mut active_entity = None;
        let mut create_components_asap = Vec::new();
        let mut destroy_entities_asap = Vec::new();

        while self.program_counter < self.program.len() {
            let next = self.read_byte();
            let instruction = ByteCode::try_from(next).unwrap();
            self.metadata.instructions_executed += 1;
            if self.verbose {
                println!(
                    "\n=== Instruction: {:?} === (pc: {})",
                    instruction,
                    self.program_counter - 1
                );
                println!("Stack: {:?}", self.stack);
            }
            match instruction {
                ByteCode::PushTrue => self.push_value(Value::True),

                ByteCode::PushFalse => self.push_value(Value::False),

                ByteCode::PushInteger => {
                    let value = self.read_i32();
                    self.push_value(Value::Integer(value));
                }

                ByteCode::PushFloat => {
                    let value = self.read_f32();
                    self.push_float(value);
                }

                ByteCode::PushString => {
                    let string = self.read_string();
                    self.push_string(string);
                }

                ByteCode::PushSimpleTag => {
                    let name = self.read_string();
                    self.push_simple_tag(name);
                }

                ByteCode::PushTag => {
                    let name = self.read_string();
                    let value = self.pop_any();
                    self.push_tag(name, value);
                }

                ByteCode::PushList => {
                    let length = self.read_i32();

                    let mut list = Vec::new();
                    for _ in 0..length {
                        list.insert(0, self.pop_any()); // TODO(anissen): Is there a more performant approach?
                    }
                    self.push_list(list);
                }

                ByteCode::PushComponent => {
                    let component_id = self.read_byte();
                    let property_count = self.read_byte();
                    let properties = self.pop_many(property_count);
                    self.push_component(component_id, properties);
                }

                ByteCode::GetTagName => {
                    let tag = self.peek_top();
                    match tag {
                        Value::Tag(name, _) => self.push_string(name.clone()),
                        Value::SimpleTag(name) => self.push_string(name.clone() + "!"), // Hack to distinguish between simple and complex tags
                        _ => unreachable!(),
                    }
                }

                ByteCode::GetTagPayload => {
                    let tag = self.peek_top();
                    match tag {
                        Value::Tag(_, payload) => self.push_value(*payload.clone()),
                        _ => unreachable!(),
                    }
                }

                ByteCode::IntegerAddition => {
                    let right = self.pop_integer();
                    let left = self.pop_integer();
                    self.push_integer(left + right);
                }

                ByteCode::IntegerSubtraction => {
                    let right = self.pop_integer();
                    let left = self.pop_integer();
                    self.push_integer(left - right);
                }

                ByteCode::IntegerMultiplication => {
                    let right = self.pop_integer();
                    let left = self.pop_integer();
                    self.push_integer(left * right);
                }

                ByteCode::IntegerDivision => {
                    let right = self.pop_integer();
                    let left = self.pop_integer();
                    if right != 0 {
                        self.push_integer(left / right);
                    } else {
                        self.push_integer(0);
                    }
                }

                ByteCode::IntegerModulo => {
                    let modulus = self.pop_integer();
                    let value = self.pop_integer();
                    self.push_integer(value % modulus);
                }

                ByteCode::IntegerLessThan => {
                    let right = self.pop_integer();
                    let left = self.pop_integer();
                    self.push_boolean(left < right);
                }

                ByteCode::IntegerLessThanEquals => {
                    let right = self.pop_integer();
                    let left = self.pop_integer();
                    self.push_boolean(left <= right);
                }

                ByteCode::FloatAddition => {
                    let right = self.pop_float();
                    let left = self.pop_float();
                    self.push_float(left + right);
                }

                ByteCode::FloatSubtraction => {
                    let right = self.pop_float();
                    let left = self.pop_float();
                    self.push_float(left - right);
                }

                ByteCode::FloatMultiplication => {
                    let right = self.pop_float();
                    let left = self.pop_float();
                    self.push_float(left * right);
                }

                ByteCode::FloatDivision => {
                    let right = self.pop_float();
                    let left = self.pop_float();
                    if right != 0.0 {
                        self.push_float(left / right);
                    } else {
                        self.push_float(0.0);
                    }
                }

                ByteCode::FloatModulo => {
                    let modulus = self.pop_float();
                    let value = self.pop_float();
                    self.push_float(value % modulus);
                }

                ByteCode::FloatLessThan => {
                    let right = self.pop_float();
                    let left = self.pop_float();
                    self.push_boolean(left < right);
                }

                ByteCode::FloatLessThanEquals => {
                    let right = self.pop_float();
                    let left = self.pop_float();
                    self.push_boolean(left <= right);
                }

                ByteCode::StringConcat => {
                    let right = self.pop_any();
                    let left = self.pop_string();
                    self.push_string(self.string_concat_values(left, right));
                }

                ByteCode::BooleanAnd => {
                    let right = self.pop_boolean();
                    let left = self.pop_boolean();
                    self.push_boolean(left && right)
                }

                ByteCode::BooleanOr => {
                    let right = self.pop_boolean();
                    let left = self.pop_boolean();
                    self.push_boolean(left || right)
                }

                ByteCode::Equals => {
                    let right = self.pop_any();
                    let left = self.pop_any();
                    self.push_boolean(left == right)
                }

                ByteCode::Negation => {
                    let value = self.pop_float();
                    self.push_float(-value);
                }

                ByteCode::Not => {
                    let value = self.pop_boolean();
                    self.push_boolean(!value);
                }

                ByteCode::GetLocalValue => {
                    let index = self.read_byte();
                    let stack_index = self.current_call_frame().stack_index;
                    let value = self
                        .stack
                        .get((stack_index + index) as usize)
                        .unwrap_or_else(|| {
                            panic!(
                                "Stack underflow: tried to access index {} but stack size is {}",
                                (stack_index + index) as usize,
                                self.stack.len()
                            )
                        })
                        .clone();
                    self.push_value(value);
                }

                ByteCode::GetForeignValue => {
                    let name = self.read_string();
                    let value = context.get_value(&name);

                    self.push_value(value);
                }

                ByteCode::SetLocalValue => {
                    let index = self.read_byte();
                    let stack_index = self.current_call_frame().stack_index;
                    let actual_index = (stack_index + index) as usize;

                    let stack_top_index = self.stack.len() - 1;

                    // If we would assign to the current stack top, we are already done
                    if actual_index != stack_top_index {
                        let value = self.pop_any();
                        if actual_index < self.stack.len() {
                            self.stack[actual_index] = value;
                        } else if actual_index == self.stack.len() {
                            self.push_value(value);
                        } else {
                            panic!(
                                "Trying to set local value outside stack size. Index: {}, stack size: {}",
                                actual_index,
                                self.stack.len()
                            );
                        }
                    }
                }

                ByteCode::GetFieldValue => {
                    let index = self.read_byte();
                    let field_index = self.read_byte();
                    let stack_index = self.current_call_frame().stack_index;
                    // println!(
                    //     "Index: {}, field_index: {}, stack_index: {}",
                    //     index, field_index, stack_index
                    // );
                    let object = self
                        .stack
                        .get((stack_index + index) as usize)
                        .unwrap_or_else(|| {
                            panic!(
                                "Stack underflow: tried to access index {} but stack size is {}",
                                (stack_index + index) as usize,
                                self.stack.len()
                            )
                        });
                    let value = match object {
                        Value::Component { id: _, properties } => {
                            properties[field_index as usize].clone()
                        }
                        _ => {
                            dbg!(&self.stack);
                            panic!("Trying to get field value from non-object")
                        }
                    };
                    self.push_value(value);
                }

                ByteCode::SetFieldValue => {
                    let index = self.read_byte();
                    let field_index = self.read_byte();
                    let stack_index = self.current_call_frame().stack_index;
                    let stack_size = self.stack.len();
                    let new_value = self.pop_any();
                    let object = self
                        .stack
                        .get_mut((stack_index + index) as usize)
                        .unwrap_or_else(|| {
                            panic!(
                                "Stack underflow: tried to access index {} but stack size is {}",
                                (stack_index + index) as usize,
                                stack_size
                            )
                        });
                    match object {
                        Value::Component { id, properties } => {
                            // Update the value on stack
                            properties[field_index as usize] = new_value;

                            // Update the world representation
                            if let Some(ref mut query) = query_results {
                                // Find the column for this component in the active query
                                if let Some(column) =
                                    query.columns.iter_mut().find(|c| c.id == *id as u32)
                                {
                                    let bytes = get_bytes_from_values(properties, &column.layout);
                                    column.insert(active_entity.unwrap(), &bytes);
                                }
                            } else {
                                panic!("Trying to update component value without active query");
                            }
                        }
                        _ => panic!("Trying to get field value from non-object"),
                    };
                }

                ByteCode::GetContextValue => {
                    let name = self.read_string();

                    let value = get_context_value(&data.world_context, name);
                    self.push_value(value);
                }

                ByteCode::SetContextValue => {
                    let name = self.read_string();

                    set_context_value(&mut data.world_context, name, self.peek_top().clone());
                    // TODO(anissen): Is peek top correct here?
                }

                ByteCode::GetListElementAtIndex => {
                    let index = self.pop_integer();
                    let list = self.pop_list();
                    self.push_value(list[index as usize].clone());
                }

                ByteCode::GetArrayLength => match self.peek_top() {
                    Value::List(list) => self.push_integer(list.len() as i32),
                    _ => panic!("Expected list found something else"),
                },

                ByteCode::ArrayAppend => {
                    let value = self.pop_any();
                    // TODO(anissen): This could mutate the list in-place instead.
                    let mut list = self.pop_list();
                    list.push(value);
                    self.push_list(list);
                }

                ByteCode::Log => {
                    let value = self.peek_top();
                    println!("Log: {value}");
                }

                ByteCode::FunctionSignature => {
                    panic!("FunctionSignature: this shouldn't happen")
                }

                ByteCode::FunctionChunk => {
                    let name = self.read_string();
                    if self.verbose {
                        println!("FunctionChunk: {name}");
                    }
                }

                ByteCode::Function => {
                    let function_index = self.read_byte();
                    self.read_byte(); // arity

                    self.push_value(Value::Function(function_index));
                }

                ByteCode::Return => {
                    self.pop_call_frame();
                }

                ByteCode::Call => {
                    let arity = self.read_byte();
                    let is_global = self.read_byte() == 1;
                    let index = self.read_byte(); // TODO(anissen): This seems off
                    let name = self.read_string();
                    if self.verbose {
                        println!("function name: {name}");
                        println!("is_global: {is_global}");
                        println!("arity: {arity}");
                        println!("index: {index}");
                    }

                    let value = self.stack.get(index as usize).unwrap();
                    let function_index = match value {
                        Value::Function(f) => *f,
                        _ => panic!("expected function, encountered some other type"),
                    };
                    let function = self.functions[function_index as usize].clone(); // TODO(anissen): Clone hack
                    self.call(function, arity)
                }

                ByteCode::CallForeign => {
                    let _foreign_index = self.read_byte();
                    let arity = self.read_byte();
                    let name = self.read_string();

                    let function_stack = self.pop_many(arity);
                    let result = context.call_function(&name, &function_stack); // TODO(anissen): Should use index instead

                    self.push_value(result);
                }

                ByteCode::Jump => {
                    let offset = self.read_i16();
                    self.jump_offset(offset);
                }

                ByteCode::JumpIfTrue => {
                    let offset = self.read_i16();

                    let condition = self.pop_boolean();
                    if condition {
                        self.jump_offset(offset);
                    }
                }

                ByteCode::JumpIfFalse => {
                    let offset = self.read_i16();

                    let condition = self.pop_boolean();
                    if !condition {
                        self.jump_offset(offset);
                    }
                }

                ByteCode::ContextQuery => {
                    if query_results.is_some() {
                        println!("*** No noes, nested queries are not yet supported! ***");
                        panic!("Nested queries are not yet supported!");
                    }

                    let jump_offset = self.read_i16();
                    let pc = self.program_counter;
                    let include_component_count = self.read_byte();
                    let exclude_component_count = self.read_byte();

                    // collect all component ids and names for printing
                    let mut include_component_ids = Vec::new();
                    for _ in 0..include_component_count {
                        let component_id = self.read_byte();
                        include_component_ids.push(component_id as u32);
                        let _component_name = self.read_string();
                    }

                    let mut exclude_component_ids = Vec::new();
                    for _ in 0..exclude_component_count {
                        let component_id = self.read_byte();
                        exclude_component_ids.push(component_id as u32);
                        let _component_name = self.read_string();
                    }

                    // TODO(anissen): Alternatively, create a structure to encapsulate a query-execution-state, allowing component scope to be expressed for the borrow checker
                    let end_pc = get_jump_offset(pc, jump_offset);

                    // Drop any previous query_entities to release the mutable borrow
                    query_results = None;

                    // Get the mutable query iterator
                    let query_iter = data
                        .elements
                        .world
                        .query(&include_component_ids, &exclude_component_ids);

                    // Check if there are any results by checking if columns are empty
                    // If there are columns, there should be results
                    let has_results = !query_iter.columns.is_empty();

                    if has_results {
                        query_results = Some(query_iter);
                        self.push_query_frame(end_pc);
                    } else {
                        self.jump(end_pc);
                    }
                }

                ByteCode::SetNextComponentColumnOrJump => {
                    if let Some(ref mut result) = query_results
                        && let Some(entity) = result.next()
                    {
                        active_entity = Some(entity); // TODO(anissen): This is a hack
                        let stack_start = self.current_call_frame().stack_index;
                        let is_first_query_result = self.stack.len() as u8 == stack_start;
                        let components = result.columns.iter().map(|column| {
                            let component_id = column.id as u8;
                            let data = column.get(entity).unwrap();
                            let values = get_value_from_bytes(data, &column.layout);

                            Value::Component {
                                id: component_id,
                                properties: values,
                            }
                        });

                        if is_first_query_result {
                            // Push components on the stack
                            components.for_each(|component| self.push_value(component));
                        } else {
                            // Replace components on the stack
                            components.enumerate().for_each(|(index, component)| {
                                self.stack[stack_start as usize + index] = component;
                            });
                        }
                    } else if query_results.is_some() {
                        // No query is active
                        query_results = None;
                        active_entity = None;
                        self.pop_query_frame();

                        destroy_entities_asap
                            .iter()
                            .for_each(|entity| destroy_entity(data, *entity));
                        destroy_entities_asap.clear();

                        create_components_asap
                            .iter()
                            .for_each(|components: &Vec<Value>| create_entity(data, components));
                        create_components_asap.clear();
                    }
                }

                ByteCode::Create => {
                    let components = self.pop_list();
                    match query_results {
                        Some(_) => {
                            // Create the entity when the query goes out of scope
                            create_components_asap.push(components)
                        }
                        None => {
                            query_results = None; // Redundant but helps the borrow checker
                            create_entity(data, &components);
                        }
                    }

                    // self.push_integer(entity as i32);
                }

                ByteCode::Destroy => {
                    // let entity_id = match self.pop_any() {
                    //     Value::Component { id, properties } => {
                    //         properties.find
                    //     }
                    //     panic!("Expected a component")
                    // };
                    let entity = self.pop_integer() as Entity;
                    match query_results {
                        Some(_) => {
                            // Destroy the entity when the query goes out of scope
                            destroy_entities_asap.push(entity)
                        }
                        None => {
                            println!("destroy the entity immediately");
                            dbg!(&entity);
                            query_results = None; // Redundant but helps the borrow checker
                            destroy_entity(data, entity);
                        }
                    }

                    // self.push_integer(entity as i32);
                }
            }
            if self.verbose {
                println!("Stack: {:?}", self.stack);
            }
        }
        if self.verbose {
            println!("End stack: {:?}", self.stack);
        }
        self.stack.pop()
    }

    fn call(&mut self, function: FunctionObj, arity: u8) {
        let ip = function.ip;
        self.call_stack.push(CallFrame {
            return_program_counter: self.program_counter,
            stack_index: (self.stack.len() - (arity as usize)) as u8,
        });
        self.program_counter = ip as usize;
    }

    fn push_query_frame(&mut self, return_program_counter: usize) {
        self.call_stack.push(CallFrame {
            return_program_counter,
            stack_index: self.stack.len() as u8,
        });
    }

    fn pop_query_frame(&mut self) {
        self.pop_call_frame();
    }

    fn current_call_frame(&self) -> &CallFrame {
        &self.call_stack[self.call_stack.len() - 1]
    }

    fn pop_call_frame(&mut self) {
        let result = self.stack.pop().unwrap_or(EMPTY_VALUE.clone());

        // Pop the stack back to the call frame's stack index
        self.discard(self.stack.len() as u8 - self.current_call_frame().stack_index);

        // Push the return value
        self.push_value(result);

        self.program_counter = self.current_call_frame().return_program_counter;

        self.call_stack.pop();
    }

    // TODO(anissen): All the function below should be part of the CallFrame impl instead (see https://craftinginterpreters.com/calls-and-functions.html @ "We’ll start at the top and plow through it.")

    fn track_bytes_read(&mut self, bytes: usize) {
        self.metadata.bytes_read += bytes;
    }

    fn read_byte(&mut self) -> u8 {
        let byte = self.program[self.program_counter];
        self.program_counter += 1;
        self.track_bytes_read(1);
        byte
    }

    fn read_2bytes(&mut self) -> [u8; 2] {
        let value_bytes: [u8; 2] = self.program[self.program_counter..self.program_counter + 2]
            .try_into()
            .unwrap();
        self.program_counter += 2;
        self.track_bytes_read(2);
        value_bytes
    }

    fn read_4bytes(&mut self) -> [u8; 4] {
        let value_bytes: [u8; 4] = self.program[self.program_counter..self.program_counter + 4]
            .try_into()
            .unwrap();
        self.program_counter += 4;
        self.track_bytes_read(4);
        value_bytes
    }

    fn read_i16(&mut self) -> i16 {
        let raw = self.read_2bytes();
        i16::from_be_bytes(raw)
    }

    fn read_u16(&mut self) -> u16 {
        let raw = self.read_2bytes();
        u16::from_be_bytes(raw)
    }

    fn read_i32(&mut self) -> i32 {
        let raw = self.read_4bytes();
        i32::from_be_bytes(raw)
    }

    fn read_f32(&mut self) -> f32 {
        let raw = u32::from_be_bytes(self.read_4bytes());
        f32::from_bits(raw)
    }

    fn read_string(&mut self) -> String {
        let length = self.read_byte();
        self.read_string_bytes(length as usize)
    }

    fn read_string_bytes(&mut self, length: usize) -> String {
        let bytes: Vec<u8> =
            self.program[self.program_counter..self.program_counter + length].into();
        self.program_counter += length;
        self.track_bytes_read(length);
        String::from_utf8(bytes).unwrap()
    }

    fn jump(&mut self, pc: usize) {
        self.program_counter = pc;
        self.metadata.jumps_performed += 1;
    }

    fn jump_offset(&mut self, offset: i16) {
        self.jump(get_jump_offset(self.program_counter, offset));
    }

    fn pop_boolean(&mut self) -> bool {
        match self.stack.pop().unwrap() {
            Value::True => true,
            Value::False => false,
            _ => panic!("expected boolean, encountered some other type"),
        }
    }

    fn peek_top(&self) -> &Value {
        self.peek(0)
    }

    fn peek(&self, distance: u8) -> &Value {
        self.stack
            .get(self.stack.len() - 1 - distance as usize)
            .unwrap()
    }

    fn discard(&mut self, count: u8) {
        for _ in 0..count {
            self.stack.pop();
        }
    }

    fn pop_many(&mut self, count: u8) -> Vec<Value> {
        self.stack.split_off(self.stack.len() - (count as usize))
    }

    fn pop_any(&mut self) -> Value {
        self.stack.pop().unwrap()
    }

    fn pop_string(&mut self) -> String {
        match self.stack.pop().unwrap() {
            Value::String(s) => s,
            _ => panic!("expected string, encountered some other type"),
        }
    }

    fn push_boolean(&mut self, value: bool) {
        let v = if value { Value::True } else { Value::False };
        self.push_value(v);
    }

    fn pop_integer(&mut self) -> i32 {
        match self.stack.pop().unwrap() {
            Value::Integer(i) => i,
            _ => panic!("expected integer, encountered some other type"),
        }
    }

    fn pop_list(&mut self) -> Vec<Value> {
        match self.stack.pop().unwrap() {
            Value::List(l) => l,
            _ => panic!("expected list, encountered some other type"),
        }
    }

    fn pop_float(&mut self) -> f32 {
        match self.stack.pop().unwrap() {
            Value::Float(f) => f,
            _ => panic!("expected float, encountered some other type"),
        }
    }

    // Private helper method to track stack operations in metadata
    fn track_stack_push(&mut self) {
        self.metadata.stack_allocations += 1;
        if self.stack.len() > self.metadata.max_stack_height {
            self.metadata.max_stack_height = self.stack.len();
        }
    }

    fn push_value(&mut self, value: Value) {
        self.stack.push(value);
        self.track_stack_push();
    }

    fn push_float(&mut self, value: f32) {
        self.push_value(Value::Float(value));
    }

    fn push_integer(&mut self, value: i32) {
        self.push_value(Value::Integer(value));
    }

    fn push_string(&mut self, value: String) {
        self.push_value(Value::String(value));
    }

    fn push_simple_tag(&mut self, name: String) {
        self.push_value(Value::SimpleTag(name));
    }

    fn push_tag(&mut self, name: String, payload: Value) {
        self.push_value(Value::Tag(name, Box::new(payload)));
    }

    fn push_list(&mut self, list: Vec<Value>) {
        self.push_value(Value::List(list));
    }

    fn push_component(&mut self, id: u8, properties: Vec<Value>) {
        self.push_value(Value::Component { id, properties });
    }

    fn string_concat_values(&self, left: String, right: Value) -> String {
        match right {
            Value::String(right) => left + &right,
            Value::Integer(right) => left + &right.to_string(),
            Value::Float(right) => left + &right.to_string(),
            Value::True => left + "true",
            Value::False => left + "false",
            Value::SimpleTag(name) => left + &name,
            Value::Tag(name, value) => self.string_concat_values(left + &name + "(", *value) + ")",
            _ => panic!("incompatible types for string concatenation"),
        }
    }
}

fn create_entity(data: &mut PersistentData, components: &Vec<Value>) {
    let entity = data.elements.entity_manager.create();

    for component in components {
        match component {
            Value::Component { id, properties } => {
                if let Some(layout) = data.elements.world.get_component_layout(*id as u32) {
                    let bytes = get_bytes_from_values(&properties, layout);
                    data.elements.world.insert(*id as u32, entity, &bytes);
                }
            }
            _ => {
                println!("Expected component type, found {:?}", component);
                panic!("Expected component type")
            }
        }
    }
}

fn destroy_entity(data: &mut PersistentData, entity: Entity) {
    data.elements.world.destroy(entity);
}

fn get_jump_offset(pc: usize, offset: i16) -> usize {
    pc.strict_add_signed(offset as isize)
}

fn get_context_value(world_context: &HashMap<String, Value>, name: String) -> Value {
    world_context.get(&name).unwrap().clone()
}

fn set_context_value(world_context: &mut HashMap<String, Value>, name: String, value: Value) {
    world_context.insert(name, value);
}

fn read_f32(b: &[u8]) -> f32 {
    f32::from_be_bytes(b.try_into().unwrap())
}

fn read_i32(b: &[u8]) -> i32 {
    i32::from_be_bytes(b.try_into().unwrap())
}

fn read_byte(b: &[u8]) -> u8 {
    b[0]
}

fn read_string(b: &[u8]) -> String {
    let length = b[0] as usize;
    let bytes: Vec<u8> = b[1..length + 1].into();

    String::from_utf8(bytes).unwrap()
}

fn get_value_from_bytes(data: &[u8], layout: &ComponentLayout) -> Vec<Value> {
    let mut offset = 0;
    layout
        .fields
        .iter()
        .map(|field| {
            let size = field.size as usize;
            let bytes = &data[offset..offset + size];
            offset += size;
            match field.type_id {
                0 => {
                    if read_byte(bytes) != 0 {
                        Value::True
                    } else {
                        Value::False
                    }
                }
                1 => Value::Integer(read_i32(bytes)),
                2 => Value::Float(read_f32(bytes)),
                3 => Value::String(read_string(bytes)),
                _ => panic!("unknown type id"),
            }
        })
        .collect()
}

fn get_bytes_from_values(values: &Vec<Value>, layout: &ComponentLayout) -> Vec<u8> {
    let mut bytes = Vec::new();
    layout.fields.iter().enumerate().for_each(|(index, field)| {
        let value = &values[index];
        let value_bytes = get_bytes_from_value(value, field);
        bytes.extend_from_slice(&value_bytes);
    });
    bytes
}

fn get_bytes_from_value(value: &Value, field_layout: &FieldLayout) -> Vec<u8> {
    let mut bytes = Vec::new();
    match field_layout.type_id {
        0 => match value {
            Value::True => bytes.push(1),
            Value::False => bytes.push(0),
            _ => panic!("Expected boolean property"),
        },
        1 => match value {
            Value::Integer(value) => bytes.extend_from_slice(&value.to_be_bytes()),
            _ => panic!("Expected integer property"),
        },
        2 => match value {
            Value::Float(value) => bytes.extend_from_slice(&value.to_be_bytes()),
            _ => panic!("Expected float property"),
        },
        3 => match value {
            Value::String(value) => {
                if value.len() > 32 {
                    panic!("String too long");
                }
                bytes.push(value.len() as u8);
                bytes.extend_from_slice(value.as_bytes()); // TODO(anissen): What about byte order?
                for _ in bytes.len()..33 {
                    bytes.push(0);
                }
            }
            _ => panic!("Expected string property"),
        },
        _ => panic!("Unsupported type"),
    };
    bytes.to_vec()
}
