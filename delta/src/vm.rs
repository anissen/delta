use std::collections::HashMap;
use std::fmt::Display;

use crate::ExecutionMetadata;
use crate::bytecodes::ByteCode;
use crate::program::Context;

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
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::True => write!(f, "true")?,
            Value::False => write!(f, "false")?,
            Value::Integer(i) => write!(f, "{i}")?,
            Value::Float(d) => write!(f, "{d}")?,
            Value::String(s) => write!(f, "{s}")?,
            Value::SimpleTag(t) => write!(f, ":{t}")?,
            Value::Tag(t, a) => write!(f, ":{t}({a})")?,
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
    functions: Vec<FunctionObj>,
    stack: Vec<Value>,
    call_stack: Vec<CallFrame>,
    verbose: bool,
    pub metadata: ExecutionMetadata,
    pub world_context: HashMap<String, Value>, // TODO(anissen): Find a better name
}

pub fn run<'a>(
    bytes: Vec<u8>,
    function_name: Option<String>,
    context: &'a Context<'a>,
    verbose: bool,
) -> Option<Value> {
    VirtualMachine::new(bytes, verbose).execute(function_name, context)
}

impl VirtualMachine {
    pub fn new(bytes: Vec<u8>, verbose: bool) -> Self {
        Self {
            program: bytes,
            program_counter: 0,
            functions: Vec::new(),
            stack: Vec::new(),
            call_stack: Vec::new(),
            verbose,
            metadata: ExecutionMetadata::default(),
            world_context: HashMap::new(),
        }
    }

    fn read_header(&mut self) {
        // TODO(anissen): Read header here

        self.read_functions();
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

    pub fn execute(&mut self, function: Option<String>, context: &Context) -> Option<Value> {
        self.program_counter = 0;

        self.read_header();

        if self.program_counter >= self.program.len() {
            return None;
        }

        let main_start = self.program_counter - 1;

        // Construct an initial call frame for the top-level code.
        self.program_counter = self.program.len(); // Set return IP to EOF.

        let function_to_execute = match function {
            Some(function_name) => self
                .functions
                .iter()
                .find(|f| f.name == function_name)
                .unwrap()
                .clone(),
            None => FunctionObj {
                name: "<main>".to_string(),
                ip: main_start as u32,
            },
        };
        self.call(function_to_execute, 0);

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
                    let component_id = self.read_i32();
                    println!("push_component");
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

                // TODO: Add bytecode instructions for ECS operations
                ByteCode::SetLocalValue => {
                    let index = self.read_byte();
                    let stack_index = self.current_call_frame().stack_index;
                    let value = self.peek_top().clone();
                    let actual_index = (stack_index + index) as usize;
                    if actual_index < self.stack.len() {
                        self.stack[actual_index] = value;
                    } else if actual_index == self.stack.len() {
                        self.push_value(value);
                    } else {
                        panic!("Trying to set local value outside stack size");
                    }
                }

                ByteCode::GetContextValue => {
                    let name = self.read_string();

                    let value = self.get_context_value(name);
                    self.push_value(value);
                }

                ByteCode::SetContextValue => {
                    let name = self.read_string();

                    self.set_context_value(name, self.peek_top().clone());
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
                    self.discard(arity); // TODO(anissen): This should not be necessary. I would expect pop_many to mutate the stack

                    self.push_value(result);
                }

                ByteCode::Jump => {
                    let offset = self.read_i16();
                    self.program_counter += offset as usize;
                    self.metadata.jumps_performed += 1;
                }

                ByteCode::JumpIfTrue => {
                    let offset = self.read_i16();

                    let condition = self.pop_boolean();
                    if condition {
                        self.program_counter += offset as usize;
                        self.metadata.jumps_performed += 1;
                    }
                }

                ByteCode::JumpIfFalse => {
                    let offset = self.read_i16();

                    let condition = self.pop_boolean();
                    if !condition {
                        self.program_counter += offset as usize;
                        self.metadata.jumps_performed += 1;
                    }
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

    fn current_call_frame(&self) -> &CallFrame {
        &self.call_stack[self.call_stack.len() - 1]
    }

    fn pop_call_frame(&mut self) {
        let result = self.stack.pop().unwrap();

        // Pop the stack back to the call frame's stack index
        self.discard(self.stack.len() as u8 - self.current_call_frame().stack_index);

        // Push the return value
        self.push_value(result);

        self.program_counter = self.current_call_frame().return_program_counter;

        self.call_stack.pop();
    }

    // TODO(anissen): All the function below should be part of the CallFrame impl instead (see https://craftinginterpreters.com/calls-and-functions.html @ "We’ll start at the top and plow through it.")

    // Private helper method to track bytes read in metadata
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

    fn string_concat_values(&self, left: String, right: Value) -> String {
        match right {
            Value::String(right) => left + &right,
            Value::Integer(right) => left + &right.to_string(),
            Value::Float(right) => left + &right.to_string(),
            Value::True => left + "true",
            Value::False => left + "false",
            Value::SimpleTag(name) => left + ":" + &name,
            Value::Tag(name, value) => {
                self.string_concat_values(left + ":" + &name + "(", *value) + ")"
            }
            _ => panic!("incompatible types for string concatenation"),
        }
    }

    fn get_context_value(&self, name: String) -> Value {
        self.world_context.get(&name).unwrap().clone()
    }

    fn set_context_value(&mut self, name: String, value: Value) {
        self.world_context.insert(name, value);
    }
}
