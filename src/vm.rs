use crate::bytecodes::ByteCode;

pub struct VirtualMachine {
    program: Vec<u8>,
    program_counter: usize,
}

pub fn run(bytes: Vec<u8>) -> Option<f32> {
    VirtualMachine::new(bytes).execute()
}

impl VirtualMachine {
    fn new(bytes: Vec<u8>) -> Self {
        Self {
            program: bytes,
            program_counter: 0,
        }
    }

    pub fn execute(&mut self) -> Option<f32> {
        let mut stack: Vec<f32> = vec![];
        while self.program_counter < self.program.len() {
            let instruction = ByteCode::try_from(self.program[self.program_counter]).unwrap();
            self.program_counter += 1;
            println!("=== frame === (pc: {})", self.program_counter);
            match instruction {
                ByteCode::PushInteger => {
                    let value_bytes: [u8; 4] = self.program
                        [self.program_counter..self.program_counter + 4]
                        .try_into()
                        .unwrap();
                    let raw = u32::from_be_bytes(value_bytes);
                    self.program_counter += 4;
                    stack.push(raw as f32);
                }

                ByteCode::PushFloat => {
                    let value_bytes: [u8; 4] = self.program
                        [self.program_counter..self.program_counter + 4]
                        .try_into()
                        .unwrap();
                    let raw = u32::from_be_bytes(value_bytes);
                    let value: f32 = f32::from_bits(raw);
                    self.program_counter += 4;
                    stack.push(value);
                }

                ByteCode::Addition => {
                    let right = stack.pop().unwrap();
                    let left = stack.pop().unwrap();
                    println!("{} + {}", left, right);
                    stack.push(left + right);
                }

                ByteCode::Subtraction => {
                    let right = stack.pop().unwrap();
                    let left = stack.pop().unwrap();
                    println!("{} - {}", left, right);
                    stack.push(left - right);
                } // _ => println!("unhandled instruction: {:?}", instruction),
            }
            println!("stack: {:?}", stack);
        }
        stack.pop()
    }
}
