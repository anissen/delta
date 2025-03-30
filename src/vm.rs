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
    Function(u8),
}

#[derive(Debug, Clone)]
struct FunctionObj {
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
}

pub fn run<'a>(bytes: Vec<u8>, context: &'a Context<'a>, verbose: bool) -> Option<Value> {
    VirtualMachine::new(bytes, verbose).execute(context)
}

impl VirtualMachine {
    fn new(bytes: Vec<u8>, verbose: bool) -> Self {
        Self {
            program: bytes,
            program_counter: 0,
            functions: Vec::new(),
            stack: Vec::new(),
            call_stack: Vec::new(),
            verbose,
        }
    }

    fn read_header(&mut self) {
        // TODO(anissen): Read header here

        self.read_functions();
    }

    fn read_functions(&mut self) {
        while let Ok(ByteCode::FunctionSignature) = ByteCode::try_from(self.read_byte()) {
            let name = self.read_string();
            let local_count = self.read_byte();
            let function_position = self.read_i16();

            self.functions.push(FunctionObj {
                ip: function_position as u32,
            });
        }
    }

    pub fn execute<'a>(&mut self, context: &'a Context<'a>) -> Option<Value> {
        self.read_header();

        if self.program_counter >= self.program.len() {
            return None;
        }

        let main_start = self.program_counter - 1;

        // Construct an initial call frame for the top-level code.
        self.program_counter = self.program.len(); // Set return IP to EOF.
        self.call(
            FunctionObj {
                ip: main_start as u32,
            },
            0,
        );

        while self.program_counter < self.program.len() {
            let next = self.read_byte();
            let instruction = ByteCode::try_from(next).unwrap();
            if self.verbose {
                println!(
                    "\n=== Instruction: {:?} === (pc: {})",
                    instruction,
                    self.program_counter - 1
                );
                println!("Stack: {:?}", self.stack);
            }
            match instruction {
                ByteCode::PushTrue => self.stack.push(Value::True),

                ByteCode::PushFalse => self.stack.push(Value::False),

                ByteCode::PushInteger => {
                    let value = self.read_i32();
                    self.stack.push(Value::Integer(value));
                }

                ByteCode::PushFloat => {
                    let value = self.read_f32();
                    self.push_float(value);
                }

                ByteCode::PushString => {
                    let string = self.read_string();
                    self.push_string(string);
                }

                // TODO(anissen): Should this be split into add_int + add_float for optimization?
                ByteCode::Addition => {
                    let right = self.pop_any();
                    let left = self.pop_any();
                    match (left, right) {
                        (Value::Float(left), Value::Float(right)) => self.push_float(left + right),

                        (Value::Integer(left), Value::Integer(right)) => {
                            self.stack.push(Value::Integer(left + right))
                        }

                        _ => panic!("incompatible types for addition"),
                    }
                }

                ByteCode::Subtraction => {
                    let right = self.pop_any();
                    let left = self.pop_any();
                    match (left, right) {
                        (Value::Float(left), Value::Float(right)) => self.push_float(left - right),

                        (Value::Integer(left), Value::Integer(right)) => {
                            self.stack.push(Value::Integer(left - right))
                        }

                        _ => panic!("incompatible types for subtraction"),
                    }
                }

                ByteCode::Multiplication => {
                    let right = self.pop_any();
                    let left = self.pop_any();
                    match (left, right) {
                        (Value::Float(left), Value::Float(right)) => self.push_float(left * right),

                        (Value::Integer(left), Value::Integer(right)) => {
                            self.push_integer(left * right)
                        }

                        _ => panic!("incompatible types for multiplication"),
                    }
                }

                ByteCode::Division => {
                    let right = self.pop_any();
                    let left = self.pop_any();
                    match (left, right) {
                        (Value::Float(left), Value::Float(right)) => {
                            if right == 0.0 {
                                self.push_float(0.0);
                            } else {
                                self.push_float(left / right)
                            }
                        }

                        (Value::Integer(left), Value::Integer(right)) => {
                            if right == 0 {
                                self.push_integer(0);
                            } else {
                                self.push_integer(left / right)
                            }
                        }

                        _ => panic!("incompatible types for division"),
                    }
                }

                ByteCode::Modulo => {
                    let modulus = self.pop_any();
                    let value = self.pop_any();
                    match (value, modulus) {
                        (Value::Float(value), Value::Float(modulus)) => {
                            self.push_float(value % modulus)
                        }

                        (Value::Integer(value), Value::Integer(modulus)) => {
                            self.push_integer(value % modulus)
                        }

                        _ => panic!("incompatible types for multiplication"),
                    }
                }

                ByteCode::StringConcat => {
                    let right = self.pop_any();
                    let left = self.pop_any();
                    match (left, right) {
                        (Value::String(left), Value::String(right)) => {
                            self.push_string(left + &right);
                        }

                        (Value::String(left), Value::Integer(right)) => {
                            self.push_string(left + &right.to_string());
                        }

                        (Value::String(left), Value::Float(right)) => {
                            self.push_string(left + &right.to_string());
                        }

                        (Value::String(left), Value::True) => {
                            self.push_string(left + "true");
                        }

                        (Value::String(left), Value::False) => {
                            self.push_string(left + "false");
                        }

                        _ => panic!("incompatible types for string concatenation"),
                    }
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

                ByteCode::LessThan => {
                    let right = self.pop_any();
                    let left = self.pop_any();
                    match (left, right) {
                        (Value::Float(left), Value::Float(right)) => {
                            self.push_boolean(left < right);
                        }

                        (Value::Integer(left), Value::Integer(right)) => {
                            self.push_boolean(left < right);
                        }

                        _ => panic!("incompatible types for less than comparison"),
                    }
                }

                ByteCode::LessThanEquals => {
                    let right = self.pop_any();
                    let left = self.pop_any();
                    match (left, right) {
                        (Value::Float(left), Value::Float(right)) => {
                            self.push_boolean(left <= right);
                        }

                        (Value::Integer(left), Value::Integer(right)) => {
                            self.push_boolean(left <= right);
                        }

                        _ => panic!("incompatible types for less than equals comparison"),
                    }
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
                        .unwrap()
                        .clone();
                    self.stack.push(value);
                }

                ByteCode::GetForeignValue => {
                    let name = self.read_string();
                    let value = context.get_value(&name);

                    self.stack.push(value);
                }

                ByteCode::SetLocalValue => {
                    let index = self.read_byte();
                    let stack_index = self.current_call_frame().stack_index;
                    let value = self.peek(0).clone();
                    let actual_index = (stack_index + index) as usize;
                    if actual_index < self.stack.len() {
                        self.stack[actual_index] = value;
                    } else if actual_index == self.stack.len() {
                        self.stack.push(value);
                    } else {
                        panic!("Trying to set local value outside stack size");
                    }
                }

                ByteCode::FunctionSignature => {
                    panic!("FunctionSignature: this shouldn't happen")
                }

                ByteCode::FunctionChunk => {
                    let name = self.read_string();
                    if self.verbose {
                        println!("FunctionChunk: {}", name);
                    }
                }

                ByteCode::Function => {
                    let function_index = self.read_byte();
                    self.read_byte(); // arity

                    self.stack.push(Value::Function(function_index));
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
                        println!("function name: {}", name);
                        println!("is_global: {}", is_global);
                        println!("arity: {}", arity);
                        println!("index: {}", index);
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

                    self.stack.push(result);
                }

                ByteCode::Jump => {
                    let offset = self.read_i16();
                    self.program_counter += offset as usize;
                }

                ByteCode::JumpIfTrue => {
                    let offset = self.read_i16();

                    let condition = self.pop_boolean();
                    if condition {
                        self.program_counter += offset as usize;
                    }
                }

                ByteCode::JumpIfFalse => {
                    let offset = self.read_i16();

                    let condition = self.pop_boolean();
                    if !condition {
                        self.program_counter += offset as usize;
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
        self.stack.push(result);

        self.program_counter = self.current_call_frame().return_program_counter;

        self.call_stack.pop();
    }

    // TODO(anissen): All the function below should be part of the CallFrame impl instead (see https://craftinginterpreters.com/calls-and-functions.html @ "We’ll start at the top and plow through it.")
    fn read_byte(&mut self) -> u8 {
        let byte = self.program[self.program_counter];
        self.program_counter += 1;
        byte
    }

    fn read_2bytes(&mut self) -> [u8; 2] {
        let value_bytes: [u8; 2] = self.program[self.program_counter..self.program_counter + 2]
            .try_into()
            .unwrap();
        self.program_counter += 2;
        value_bytes
    }

    fn read_4bytes(&mut self) -> [u8; 4] {
        let value_bytes: [u8; 4] = self.program[self.program_counter..self.program_counter + 4]
            .try_into()
            .unwrap();
        self.program_counter += 4;
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

    fn read_u32(&mut self) -> u32 {
        let raw = self.read_4bytes();
        u32::from_be_bytes(raw)
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
        String::from_utf8(bytes).unwrap()
    }

    fn pop_boolean(&mut self) -> bool {
        match self.stack.pop().unwrap() {
            Value::True => true,
            Value::False => false,
            _ => panic!("expected boolean, encountered some other type"),
        }
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

    fn push_boolean(&mut self, value: bool) {
        let v = if value { Value::True } else { Value::False };
        self.stack.push(v);
    }

    pub fn pop_float(&mut self) -> f32 {
        match self.stack.pop().unwrap() {
            Value::Float(f) => f,
            _ => panic!("expected float, encountered some other type"),
        }
    }

    fn push_float(&mut self, value: f32) {
        self.stack.push(Value::Float(value));
    }

    fn push_integer(&mut self, value: i32) {
        self.stack.push(Value::Integer(value));
    }

    fn push_string(&mut self, value: String) {
        self.stack.push(Value::String(value));
    }
}
