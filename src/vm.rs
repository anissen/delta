use core::str;

use crate::bytecodes::ByteCode;

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
    arity: u8,
    ip: usize, // TODO(anissen): Is ip required?
}

#[derive(Debug)]
struct CallFrame {
    return_program_counter: usize,
    stack_index: u8,
    function: FunctionObj,
}

pub struct VirtualMachine {
    program: Vec<u8>,
    program_counter: usize,
    functions: Vec<FunctionObj>,
    stack: Vec<Value>,
    call_stack: Vec<CallFrame>,
    verbose_logging: bool,
}

pub fn run(bytes: Vec<u8>) -> Option<Value> {
    VirtualMachine::new(bytes).execute()
}

impl VirtualMachine {
    fn new(bytes: Vec<u8>) -> Self {
        Self {
            program: bytes,
            program_counter: 0,
            functions: Vec::new(),
            stack: Vec::new(),
            call_stack: Vec::new(),
            verbose_logging: false,
        }
    }

    pub fn execute(&mut self) -> Option<Value> {
        while self.program_counter < self.program.len() {
            let next = self.read_byte();
            let instruction = ByteCode::try_from(next);
            if let Ok(ByteCode::FunctionStart) = instruction {
                let _function_index = self.read_byte();
                let arity = self.read_byte();
                self.functions.push(FunctionObj {
                    arity,
                    ip: self.program_counter,
                });
            }
        }
        println!("self.functions: {:?}", self.functions);

        // Construct an initial call frame for the top-level code.
        self.call(
            FunctionObj {
                arity: 0,
                ip: self.program.len(), // TODO(anissen): Hack to avoid infinite loops
            },
            0,
        );

        self.program_counter = 0;

        while self.program_counter < self.program.len() {
            let next = self.read_byte();
            let instruction = ByteCode::try_from(next).unwrap();
            println!(
                "\n=== Instruction: {:?} === (pc: {})",
                instruction, self.program_counter
            );
            println!("Stack: {:?}", self.stack);
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
                    let string_length = self.program[self.program_counter];
                    self.program_counter += 1;
                    let value_bytes: Vec<u8> = self.program
                        [self.program_counter..self.program_counter + (string_length as usize)]
                        .into();
                    self.program_counter += string_length as usize;
                    let string = String::from_utf8(value_bytes).unwrap();
                    self.push_string(string);
                }

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

                        (Value::String(left), Value::True) => {
                            self.push_string(left + "true");
                        }

                        (Value::String(left), Value::False) => {
                            self.push_string(left + "false");
                        }

                        _ => panic!("incompatible types for string concatenation"),
                    }
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
                    println!("index is: {}", index);
                    let stack_index = self.current_call_frame().stack_index;
                    println!("slots is: {}", stack_index);
                    let value = self
                        .stack
                        .get((stack_index + index) as usize)
                        .unwrap()
                        .clone();
                    self.stack.push(value);
                }

                ByteCode::SetLocalValue => {
                    let index = self.read_byte();
                    let stack_index = self.current_call_frame().stack_index;
                    let value = self.peek(0).clone();
                    println!(
                        "set local value: index: {}, stack_index: {}, value: {:?}",
                        index, stack_index, value
                    );
                    let actual_index = (stack_index + index) as usize;
                    println!(
                        "stack length: {}, actual_index: {}",
                        self.stack.len(),
                        actual_index
                    );

                    if actual_index < self.stack.len() {
                        self.stack[actual_index] = value;
                    } else if actual_index == self.stack.len() {
                        self.stack.push(value);
                    } else {
                        panic!("Trying to set local value outside stack size");
                    }
                }

                ByteCode::FunctionStart => {
                    let function_index = self.read_byte();
                    println!("function_index: {}", function_index);

                    // jump to function end HACK!
                    while self.program_counter < self.program.len() {
                        let instruction = self.read_byte();
                        if let Ok(ByteCode::FunctionEnd) = ByteCode::try_from(instruction) {
                            break;
                        }
                    }

                    self.stack.push(Value::Function(function_index));
                }

                ByteCode::FunctionEnd => {
                    self.pop_call_frame();
                }

                ByteCode::Call => {
                    let arity = self.read_byte();
                    let is_global = self.read_byte() == 1;
                    let index = self.read_byte(); // TODO(anissen): This seems off
                    let name_length = self.read_byte();
                    let value_bytes: Vec<u8> = self.program
                        [self.program_counter..self.program_counter + (name_length as usize)]
                        .into();
                    self.program_counter += name_length as usize;
                    let name = String::from_utf8(value_bytes).unwrap();
                    println!("function name: {}", name);
                    println!("is_global: {}", is_global);
                    println!("arity: {}", arity);
                    println!("index: {}", index);

                    let corrected_index = if is_global {
                        index
                    } else {
                        self.current_call_frame().stack_index + index
                    };
                    println!("corrected_index: {}", corrected_index);
                    let value = self.stack.get(corrected_index as usize).unwrap();
                    println!("value: {:?}", value);
                    let function_index = match value {
                        Value::Function(f) => *f,
                        _ => panic!("expected function, encountered some other type"),
                    };
                    println!("functions: {:?}", self.functions);
                    println!("function_index: {:?}", function_index);
                    let function = self.functions[function_index as usize].clone(); // TODO(anissen): Clone hack
                    self.call(function, arity)
                }
            }
        }
        println!("End stack: {:?}", self.stack);
        self.stack.pop()
    }

    fn call(&mut self, function: FunctionObj, arity: u8) {
        println!("call function: {:?}", function);
        println!("call with arity: {}", arity);
        let ip = function.ip;
        self.call_stack.push(CallFrame {
            function,
            return_program_counter: self.program_counter,
            stack_index: (self.stack.len() - (arity as usize)) as u8,
        });
        println!("call: {:?}", self.current_call_frame());
        self.program_counter = ip;
    }

    fn current_call_frame(&self) -> &CallFrame {
        &self.call_stack[self.call_stack.len() - 1]
    }

    fn pop_call_frame(&mut self) {
        let result = self.stack.pop().unwrap();

        // Pop the arguments from the stack
        let arity = self.current_call_frame().function.arity;
        self.discard(arity);

        // Push the return value
        self.stack.push(result);

        self.program_counter = self.current_call_frame().return_program_counter;

        self.call_stack.pop();
    }

    // TODO(anissen): All the function below should be part of the CallFrame impl instead (see https://craftinginterpreters.com/calls-and-functions.html @ "We’ll start at the top and plow through it.")
    fn read_byte(&mut self) -> u8 {
        let byte = self.program[self.program_counter];
        self.program_counter += 1;
        if self.verbose_logging {
            println!("read_byte: {}", byte);
        }
        byte
    }

    fn read_4bytes(&mut self) -> [u8; 4] {
        let value_bytes: [u8; 4] = self.program[self.program_counter..self.program_counter + 4]
            .try_into()
            .unwrap();
        self.program_counter += 4;
        value_bytes
    }

    fn read_i32(&mut self) -> i32 {
        let raw = self.read_4bytes();
        let value = i32::from_be_bytes(raw);
        if self.verbose_logging {
            println!("read_i32: {}", value);
        }
        value
    }

    fn read_f32(&mut self) -> f32 {
        let raw = u32::from_be_bytes(self.read_4bytes());
        let value = f32::from_bits(raw);
        if self.verbose_logging {
            println!("read_f32: {}", value);
        }
        value
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

    fn pop_any(&mut self) -> Value {
        self.stack.pop().unwrap()
    }

    fn push_boolean(&mut self, value: bool) {
        let v = if value { Value::True } else { Value::False };
        self.stack.push(v);
    }

    fn pop_float(&mut self) -> f32 {
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
