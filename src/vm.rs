use std::collections::HashMap;

use crate::bytecodes::ByteCode;

#[derive(Debug)]
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
        // let mut stack: Vec<Value> = vec![];
        let mut values = HashMap::new();

        while self.program_counter < self.program.len() {
            let instruction = ByteCode::try_from(self.program[self.program_counter]).unwrap();
            self.program_counter += 1;
            println!("=== frame === (pc: {})", self.program_counter);
            match instruction {
                ByteCode::PushBoolean => {
                    let value_bytes = self.program[self.program_counter];
                    self.program_counter += 1;
                    // let value = if value_bytes == 0 { false } else { true };
                    let value = if value_bytes == 0 { 0.0 } else { 1.0 };
                    self.stack.push(Value::Float(value)); // TODO(anissen): Should be Value::Boolean -- fix me!
                }

                ByteCode::PushInteger => {
                    let value_bytes: [u8; 4] = self.program
                        [self.program_counter..self.program_counter + 4]
                        .try_into()
                        .unwrap();
                    self.program_counter += 4;
                    let raw = i32::from_be_bytes(value_bytes);
                    self.stack.push(Value::Float(raw as f32)); // TODO(anissen): Should be Value::Integer -- fix me!
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
                    let right = self.pop_float();
                    let left = self.pop_float();
                    println!("{} + {}", left, right);
                    self.push_float(left + right);
                }

                ByteCode::Subtraction => {
                    let right = self.pop_float();
                    let left = self.pop_float();
                    println!("{} - {}", left, right);
                    self.push_float(left - right);
                }

                ByteCode::Multiplication => {
                    let right = self.pop_float();
                    let left = self.pop_float();
                    println!("{} * {}", left, right);
                    self.push_float(left * right);
                }

                ByteCode::Division => {
                    let right = self.pop_float();
                    let left = self.pop_float();
                    println!("{} / {}", left, right);
                    self.push_float(left / right);
                }

                ByteCode::Negation => {
                    let value = self.pop_float();
                    self.push_float(-value);
                }

                ByteCode::GetValue => {
                    let index = self.program[self.program_counter]; // TODO(anissen): Make helper function to read bytes and increment program counter
                    self.program_counter += 1;
                    let value = values.get(&index).unwrap();
                    self.push_float(*value);
                }

                ByteCode::SetValue => {
                    let index = self.program[self.program_counter];
                    self.program_counter += 1;
                    let value = self.pop_float();
                    values.insert(index, value);
                    self.push_float(value); // TODO(anissen): This could be done with a peek instead of a pop + push
                }
            }
            println!("stack: {:?}", self.stack);
        }
        self.stack.pop()
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
