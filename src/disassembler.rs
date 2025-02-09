use crate::bytecodes::ByteCode;

pub struct Disassembler {
    program: Vec<u8>,
    program_counter: usize,
    last_program_counter: usize,
}

pub fn disassemble(bytes: Vec<u8>) {
    Disassembler::new(bytes).disassemble()
}

impl Disassembler {
    fn new(bytes: Vec<u8>) -> Self {
        Self {
            program: bytes,
            program_counter: 0,
            last_program_counter: 0,
        }
    }

    fn read_i32(&mut self) -> i32 {
        let value_bytes: [u8; 4] = self.program[self.program_counter..self.program_counter + 4]
            .try_into()
            .unwrap();
        self.program_counter += 4;
        i32::from_be_bytes(value_bytes)
    }

    fn print(&mut self, values: Vec<String>) {
        println!("{} \t{:?}", self.last_program_counter, values);
    }

    pub fn disassemble(&mut self) {
        while self.program_counter < self.program.len() {
            let instruction = ByteCode::try_from(self.program[self.program_counter]).unwrap();
            self.last_program_counter = self.program_counter;
            self.program_counter += 1;
            // self.print(vec![format!("> byte: {}", self.program_counter)]);
            match instruction {
                ByteCode::PushTrue => self.print(vec!["push_true".to_string()]),

                ByteCode::PushFalse => self.print(vec!["push_false".to_string()]),

                ByteCode::PushInteger => {
                    let value = self.read_i32();
                    self.print(vec![
                        "push_integer".to_string(),
                        format!("(value: {})", value),
                    ]);
                }

                ByteCode::PushFloat => {
                    let value = self.read_i32();
                    self.print(vec![
                        "push_float".to_string(),
                        format!("(value: {})", value),
                    ]);
                }

                ByteCode::PushString => {
                    let string_length = self.program[self.program_counter];
                    self.program_counter += 1;
                    let value_bytes: Vec<u8> = self.program
                        [self.program_counter..self.program_counter + (string_length as usize)]
                        .into();
                    self.program_counter += string_length as usize;
                    let string = String::from_utf8(value_bytes).unwrap();

                    self.print(vec![format!("push_string: {}", string)]);
                }

                ByteCode::Addition => {
                    self.print(vec!["add".to_string()]);
                }

                ByteCode::Subtraction => {
                    self.print(vec!["sub".to_string()]);
                }

                ByteCode::Multiplication => {
                    self.print(vec!["mult".to_string()]);
                }

                ByteCode::Division => {
                    self.print(vec!["div".to_string()]);
                }

                ByteCode::Modulo => {
                    self.print(vec!["mod".to_string()]);
                }

                ByteCode::StringConcat => {
                    self.print(vec!["str_concat".to_string()]);
                }

                ByteCode::Equals => {
                    self.print(vec!["eq".to_string()]);
                }

                ByteCode::LessThan => {
                    self.print(vec!["lt".to_string()]);
                }

                ByteCode::LessThanEquals => {
                    self.print(vec!["lte".to_string()]);
                }

                ByteCode::Negation => {
                    self.print(vec!["neg".to_string()]);
                }

                ByteCode::Not => {
                    self.print(vec!["not".to_string()]);
                }

                ByteCode::GetLocalValue => {
                    let index = self.program[self.program_counter]; // TODO(anissen): Make helper function to read bytes and increment program counter
                    self.program_counter += 1;
                    self.print(vec!["get_value".to_string(), format!("(index: {})", index)]);
                }

                ByteCode::GetForeignValue => {
                    let name_length = self.program[self.program_counter];
                    self.program_counter += 1;
                    let value_bytes: Vec<u8> = self.program
                        [self.program_counter..self.program_counter + (name_length as usize)]
                        .into();
                    self.program_counter += name_length as usize;
                    let name = String::from_utf8(value_bytes).unwrap();

                    self.print(vec![
                        "get_foreign_value".to_string(),
                        format!("(name: {})", name),
                    ]);
                }

                ByteCode::SetLocalValue => {
                    let index = self.program[self.program_counter];
                    self.program_counter += 1;
                    self.print(vec!["set_value".to_string(), format!("(index: {})", index)]);
                }

                ByteCode::FunctionStart => {
                    let function_index = self.program[self.program_counter];
                    self.program_counter += 1;
                    let param_count = self.program[self.program_counter];
                    self.program_counter += 1;
                    self.print(vec![
                        format!("function"),
                        format!("(function index: {})", function_index),
                        format!("(params: {})", param_count),
                    ]);
                }

                ByteCode::FunctionEnd => {
                    self.print(vec!["ret".to_string()]);
                }

                ByteCode::Call => {
                    let arg_count = self.program[self.program_counter];
                    self.program_counter += 1;
                    let is_global = self.program[self.program_counter];
                    self.program_counter += 1;
                    let index = self.program[self.program_counter];
                    self.program_counter += 1;

                    let name_length = self.program[self.program_counter];
                    self.program_counter += 1;
                    let value_bytes: Vec<u8> = self.program
                        [self.program_counter..self.program_counter + (name_length as usize)]
                        .into();
                    self.program_counter += name_length as usize;
                    let name = String::from_utf8(value_bytes).unwrap();

                    self.print(vec![
                        format!("call {} (is_global: {})", name, is_global),
                        format!("(arg count: {}, function index: {})", arg_count, index),
                    ]);
                }

                ByteCode::CallForeign => {
                    let foreign_index = self.program[self.program_counter];
                    self.program_counter += 1;
                    let arg_count = self.program[self.program_counter];
                    self.program_counter += 1;

                    let name_length = self.program[self.program_counter];
                    self.program_counter += 1;
                    let value_bytes: Vec<u8> = self.program
                        [self.program_counter..self.program_counter + (name_length as usize)]
                        .into();
                    self.program_counter += name_length as usize;
                    let name = String::from_utf8(value_bytes).unwrap();

                    self.print(vec![
                        format!("call foreign function {}", name),
                        format!(
                            "(arg count: {}, foreign_index: {})",
                            arg_count, foreign_index
                        ),
                    ]);
                }

                ByteCode::JumpIfTrue => {
                    let offset = self.read_i32();
                    self.print(vec![format!(
                        "jump if true (offset: {}, to byte {})",
                        offset,
                        self.program_counter + offset as usize
                    )]);
                }
            }
        }
    }
}
