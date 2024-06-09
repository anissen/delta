use crate::bytecodes::ByteCode;

pub struct Disassembler {
    program: Vec<u8>,
    program_counter: usize,
}

pub fn disassemble(bytes: Vec<u8>) -> Vec<Vec<String>> {
    Disassembler::new(bytes).disassemble()
}

impl Disassembler {
    fn new(bytes: Vec<u8>) -> Self {
        Self {
            program: bytes,
            program_counter: 0,
        }
    }

    pub fn disassemble(&mut self) -> Vec<Vec<String>> {
        // let mut values = HashMap::new();
        let mut res = vec![];

        while self.program_counter < self.program.len() {
            let instruction = ByteCode::try_from(self.program[self.program_counter]).unwrap();
            self.program_counter += 1;
            match instruction {
                ByteCode::PushBoolean => {
                    let value_bytes = self.program[self.program_counter];
                    self.program_counter += 1;
                    // res.push("push_boolean".to_string());
                    let value = if value_bytes == 0 { "false" } else { "true" };
                    // res.push(value.to_string());
                    res.push(vec![
                        "push_boolean".to_string(),
                        format!("(value: {})", value),
                    ]);
                }

                ByteCode::PushInteger => {
                    let value_bytes: [u8; 4] = self.program
                        [self.program_counter..self.program_counter + 4]
                        .try_into()
                        .unwrap();
                    self.program_counter += 4;
                    let raw = i32::from_be_bytes(value_bytes);
                    res.push(vec![
                        "push_integer".to_string(),
                        format!("(value: {})", raw),
                    ]);
                }

                ByteCode::PushFloat => {
                    let value_bytes: [u8; 4] = self.program
                        [self.program_counter..self.program_counter + 4]
                        .try_into()
                        .unwrap();
                    let raw = u32::from_be_bytes(value_bytes);
                    let value: f32 = f32::from_bits(raw);
                    self.program_counter += 4;
                    res.push(vec![
                        "push_float".to_string(),
                        format!("(value: {})", value),
                    ]);
                }

                ByteCode::Addition => {
                    res.push(vec!["add".to_string()]);
                }

                ByteCode::Subtraction => {
                    res.push(vec!["sub".to_string()]);
                }

                ByteCode::Multiplication => {
                    res.push(vec!["mult".to_string()]);
                }

                ByteCode::Division => {
                    res.push(vec!["div".to_string()]);
                }

                ByteCode::Negation => {
                    res.push(vec!["neg".to_string()]);
                }

                ByteCode::Not => {
                    res.push(vec!["not".to_string()]);
                }

                ByteCode::GetValue => {
                    let index = self.program[self.program_counter]; // TODO(anissen): Make helper function to read bytes and increment program counter
                    self.program_counter += 1;
                    // let value = values.get(&index).unwrap();
                    res.push(vec!["get_value".to_string(), format!("(index: {})", index)]);
                }

                ByteCode::SetValue => {
                    let index = self.program[self.program_counter];
                    self.program_counter += 1;
                    // values.insert(index, value);
                    res.push(vec!["set_value".to_string(), format!("(index: {})", index)]);
                }

                ByteCode::FunctionStart => {
                    // let index = functions.len();
                    // self.stack.push(Value::Integer(42));
                    let function_index = self.program[self.program_counter];
                    self.program_counter += 1;
                    let param_count = self.program[self.program_counter];
                    self.program_counter += 1;
                    res.push(vec![
                        "function".to_string(),
                        format!("(function index: {})", function_index),
                        format!("(params: {})", param_count),
                    ]);
                }

                ByteCode::FunctionEnd => {
                    res.push(vec!["ret".to_string()]);
                }

                ByteCode::Call => {
                    let index = self.program[self.program_counter];
                    self.program_counter += 1;
                    res.push(vec![
                        "call".to_string(),
                        format!("(function index: {})", index),
                    ]);
                }
            }
        }
        res
    }
}
