use std::collections::HashMap;

use crate::bytecodes::ByteCode;

// TODO(anissen): See https://github.com/brightly-salty/rox/blob/master/src/value.rs
#[derive(Debug, Copy, Clone)]
pub enum Value {
    Boolean(bool),
    Integer(i32),
    Float(f32),
    Function(u8),
}

struct CallFrame {
    return_program_counter: usize,
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
            if let Ok(ByteCode::FunctionStart) = ByteCode::try_from(*instruction) {
                functions.push(index + 1);
            }
        }

        let mut prev_program_counter = 0; // TODO(anissen): Should probably be a stack of call frames!

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

                    // TODO(anissen): Division by zero => 0
                    match (right, left) {
                        (Value::Float(right), Value::Float(left)) => self.push_float(left / right),

                        (Value::Integer(right), Value::Integer(left)) => {
                            self.stack.push(Value::Float((left / right) as f32))
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
                    println!("index is: {}", index);
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

                // https://godbolt.org/#g:!((g:!((g:!((h:codeEditor,i:(filename:'1',fontScale:14,fontUsePx:'0',j:1,lang:scala,selection:(endColumn:1,endLineNumber:16,positionColumn:1,positionLineNumber:16,selectionStartColumn:1,selectionStartLineNumber:16,startColumn:1,startLineNumber:16),source:'@main%0Adef+main()+%3D+%7B%0A++println(%22hello%22)%0A%0A++val+y+%3D+3%0A++def+twice(v:+Float)+%3D+%7B%0A++++v+*+2+%2B+y%0A++%7D%0A%0A++println(%22world%22)%0A%0A++val+x+%3D+twice(5)%0A%0A++println(x)%0A%7D%0A'),l:'5',n:'1',o:'Scala+source+%231',t:'0')),k:50,l:'4',n:'0',o:'',s:0,t:'0'),(g:!((h:compiler,i:(compiler:scalac300,filters:(b:'0',binary:'1',binaryObject:'1',commentOnly:'0',debugCalls:'1',demangle:'0',directives:'0',execute:'1',intel:'0',libraryCode:'0',trim:'1',verboseDemangling:'0'),flagsViewOpen:'1',fontScale:14,fontUsePx:'0',j:1,lang:scala,libs:!(),options:'',overrides:!(),selection:(endColumn:14,endLineNumber:65,positionColumn:14,positionLineNumber:65,selectionStartColumn:14,selectionStartLineNumber:65,startColumn:14,startLineNumber:65),source:1),l:'5',n:'0',o:'+scalac+3.0.0+(Editor+%231)',t:'0')),k:50,l:'4',n:'0',o:'',s:0,t:'0')),l:'2',n:'0',o:'',t:'0')),version:4
                ByteCode::FunctionStart => {
                    let function_index = self.program[self.program_counter];
                    self.program_counter += 1;

                    let param_count = self.program[self.program_counter];
                    self.program_counter += 1;
                    for param in 0..param_count {
                        // ???
                    }

                    // jump to function end HACK!
                    while self.program_counter < self.program.len() {
                        let instruction = self.program[self.program_counter];
                        self.program_counter += 1;
                        if let Ok(ByteCode::FunctionEnd) = ByteCode::try_from(instruction) {
                            break;
                        }
                    }

                    self.stack.push(Value::Function(function_index));
                }

                ByteCode::FunctionEnd => {
                    // reset program counter
                    self.program_counter = prev_program_counter + 1;
                }

                ByteCode::Call => {
                    prev_program_counter = self.program_counter;
                    let index =
                        self.program[self.program_counter] /* TODO(anissen): HACK! ==> */ - 1;
                    // let function_index = self.pop_function();
                    self.program_counter = functions[index as usize] + 1 /* function index */ + 1 /* param count */;
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

    fn pop_function(&mut self) -> u8 {
        match self.stack.pop().unwrap() {
            Value::Function(f) => f,
            _ => panic!("expected function, encountered some other type"),
        }
    }
}
