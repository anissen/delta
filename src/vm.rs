use std::collections::HashMap;

use crate::bytecodes::ByteCode;

// TODO(anissen): See https://github.com/brightly-salty/rox/blob/master/src/value.rs
#[derive(Debug, Copy, Clone)]
pub enum Value {
    Boolean(bool),
    Integer(i32),
    Float(f32),
}

pub struct VirtualMachine {
    program: Vec<u8>,
    program_counter: usize,
    stack: Vec<Value>,
}

pub fn run(bytes: Vec<u8>) -> Option<Value> {
    VirtualMachine::new(bytes).execute()
}

impl VirtualMachine {
    fn new(bytes: Vec<u8>) -> Self {
        Self {
            program: bytes,
            program_counter: 0,
            stack: Vec::new(),
        }
    }

    pub fn execute(&mut self) -> Option<Value> {
        let mut values = HashMap::new();

        let mut functions: Vec<usize> = Vec::new(); // function index => starting byte
        for (index, instruction) in self.program.iter().enumerate() {
            if let ByteCode::FunctionStart = ByteCode::try_from(*instruction).unwrap() {
                functions.push(index + 1);
            }
        }

        let mut prev_program_counter = 0; // TODO(anissen): Should probably be a stack of call frames

        while self.program_counter < self.program.len() {
            let instruction = ByteCode::try_from(self.program[self.program_counter]).unwrap();
            self.program_counter += 1;
            println!("=== frame === (pc: {})", self.program_counter);
            println!("Instruction: {:?}", instruction);
            match instruction {
                ByteCode::PushBoolean => {
                    let value_bytes = self.program[self.program_counter];
                    self.program_counter += 1;
                    let value = if value_bytes == 0 { false } else { true };
                    self.stack.push(Value::Boolean(value));
                }

                ByteCode::PushInteger => {
                    let value_bytes: [u8; 4] = self.program
                        [self.program_counter..self.program_counter + 4]
                        .try_into()
                        .unwrap();
                    self.program_counter += 4;
                    let raw = i32::from_be_bytes(value_bytes);
                    self.stack.push(Value::Integer(raw));
                }

                ByteCode::PushFloat => {
                    let value_bytes: [u8; 4] = self.program
                        [self.program_counter..self.program_counter + 4]
                        .try_into()
                        .unwrap();
                    let raw = u32::from_be_bytes(value_bytes);
                    let value: f32 = f32::from_bits(raw);
                    self.program_counter += 4;
                    self.push_float(value);
                }

                ByteCode::Addition => {
                    let right = self.stack.pop().unwrap();
                    let left = self.stack.pop().unwrap();
                    // println!("{} + {}", left, right);
                    // self.push_float(left + right);
                    match (right, left) {
                        (Value::Float(right), Value::Float(left)) => self.push_float(left + right),

                        (Value::Integer(right), Value::Integer(left)) => {
                            self.stack.push(Value::Integer(left + right))
                        }

                        _ => panic!("incompatible types for addition"),
                    }
                }

                ByteCode::Subtraction => {
                    let right = self.stack.pop().unwrap();
                    let left = self.stack.pop().unwrap();
                    // println!("{} - {}", left, right);
                    // self.push_float(left - right);
                    match (right, left) {
                        (Value::Float(right), Value::Float(left)) => self.push_float(left - right),

                        (Value::Integer(right), Value::Integer(left)) => {
                            self.stack.push(Value::Integer(left - right))
                        }

                        _ => panic!("incompatible types for subtraction"),
                    }
                }

                ByteCode::Multiplication => {
                    let right = self.stack.pop().unwrap();
                    let left = self.stack.pop().unwrap();
                    // println!("{} - {}", left, right);
                    // self.push_float(left * right);
                    match (right, left) {
                        (Value::Float(right), Value::Float(left)) => self.push_float(left * right),

                        (Value::Integer(right), Value::Integer(left)) => {
                            self.stack.push(Value::Integer(left * right))
                        }

                        _ => panic!("incompatible types for multiplication"),
                    }
                }

                ByteCode::Division => {
                    let right = self.stack.pop().unwrap();
                    let left = self.stack.pop().unwrap();
                    // println!("{} / {}", left, right);
                    match (right, left) {
                        (Value::Float(right), Value::Float(left)) => self.push_float(left / right),

                        (Value::Integer(right), Value::Integer(left)) => {
                            self.stack.push(Value::Integer(left / right))
                        }

                        _ => panic!("incompatible types for division"),
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

                ByteCode::GetValue => {
                    let index = self.program[self.program_counter]; // TODO(anissen): Make helper function to read bytes and increment program counter
                    self.program_counter += 1;
                    let value = values.get(&index).unwrap();
                    self.stack.push(*value);
                }

                ByteCode::SetValue => {
                    let index = self.program[self.program_counter];
                    self.program_counter += 1;
                    let value = self.stack.pop().unwrap();
                    values.insert(index, value);
                    self.stack.push(value); // TODO(anissen): This could be done with a peek instead of a pop + push
                }

                ByteCode::FunctionStart => {
                    // let index = functions.len();
                    // self.stack.push(Value::Integer(42));
                    self.program_counter += 1;
                }

                ByteCode::FunctionEnd => {
                    // reset program counter
                    self.program_counter = prev_program_counter;
                }

                ByteCode::Call => {
                    prev_program_counter = self.program_counter + 1;
                    let index = self.program[self.program_counter];
                    self.program_counter = functions[index as usize];
                }
            }
            println!("stack: {:?}", self.stack);
        }
        self.stack.pop()
    }

    // fn call(&mut self) {
    //    // ...
    // }

    fn pop_boolean(&mut self) -> bool {
        match self.stack.pop().unwrap() {
            Value::Boolean(b) => b,
            _ => panic!("expected boolean, encountered some other type"),
        }
    }

    fn push_boolean(&mut self, value: bool) {
        self.stack.push(Value::Boolean(value));
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
}
