use crate::CompilationMetadata;
use crate::bytecodes::ByteCode;

pub struct Disassembler {
    program: Vec<u8>,
    program_counter: usize,
    last_program_counter: usize,
}

pub fn disassemble(bytes: Vec<u8>, metadata: &mut CompilationMetadata) {
    Disassembler::new(bytes).disassemble(metadata)
}

impl Disassembler {
    fn new(program: Vec<u8>) -> Self {
        Self {
            program,
            program_counter: 0,
            last_program_counter: 0,
        }
    }

    fn read_i16(&mut self) -> i16 {
        let bytes = [
            self.program[self.program_counter],
            self.program[self.program_counter + 1],
        ];
        self.program_counter += 2;
        i16::from_be_bytes(bytes)
    }

    fn read_i32(&mut self) -> i32 {
        let bytes = [
            self.program[self.program_counter],
            self.program[self.program_counter + 1],
            self.program[self.program_counter + 2],
            self.program[self.program_counter + 3],
        ];
        self.program_counter += 4;
        i32::from_be_bytes(bytes)
    }

    fn read_f32(&mut self) -> f32 {
        let bytes = [
            self.program[self.program_counter],
            self.program[self.program_counter + 1],
            self.program[self.program_counter + 2],
            self.program[self.program_counter + 3],
        ];
        self.program_counter += 4;
        f32::from_be_bytes(bytes)
    }

    fn read_byte(&mut self) -> u8 {
        let byte = self.program[self.program_counter];
        self.program_counter += 1;
        byte
    }

    fn read_string(&mut self) -> String {
        let string_length = self.read_byte();
        self.read_string_bytes(string_length)
    }

    fn read_string_bytes(&mut self, string_length: u8) -> String {
        let value_bytes: Vec<u8> = self.program
            [self.program_counter..self.program_counter + (string_length as usize)]
            .into();
        self.program_counter += string_length as usize;
        String::from_utf8(value_bytes).unwrap()
    }

    fn print(&mut self, values: Vec<String>) -> String {
        let formatted = format!("{} \t{}\n", self.last_program_counter, values.join(" "));
        print!("{formatted}");
        formatted
    }

    pub fn disassemble(&mut self, metadata: &mut CompilationMetadata) {
        let mut result = String::new();

        while self.program_counter < self.program.len() {
            let instruction = ByteCode::try_from(self.program[self.program_counter]).unwrap();
            self.last_program_counter = self.program_counter;
            self.program_counter += 1;

            let instruction_str = match instruction {
                ByteCode::PushTrue => self.print(vec!["push_true".to_string()]),

                ByteCode::PushFalse => self.print(vec!["push_false".to_string()]),

                ByteCode::PushInteger => {
                    let value = self.read_i32();
                    self.print(vec![
                        "push_integer".to_string(),
                        format!("(value: {})", value),
                    ])
                }

                ByteCode::PushFloat => {
                    let value = self.read_f32();
                    self.print(vec![
                        "push_float".to_string(),
                        format!("(value: {})", value),
                    ])
                }

                ByteCode::PushString => {
                    let string_length = self.read_byte();
                    let value_bytes: Vec<u8> = self.program
                        [self.program_counter..self.program_counter + (string_length as usize)]
                        .into();
                    self.program_counter += string_length as usize;
                    let string = String::from_utf8(value_bytes).unwrap();

                    self.print(vec![
                        "push_string".to_string(),
                        format!("(value: '{}')", string),
                    ])
                }

                ByteCode::PushSimpleTag => {
                    let tag_name = self.read_string();

                    self.print(vec![
                        "push_simple_tag".to_string(),
                        format!("(value: '{tag_name}')"),
                    ])
                }

                ByteCode::PushTag => {
                    let tag_name = self.read_string();

                    self.print(vec![
                        "push_tag".to_string(),
                        format!("(value: '{}')", tag_name),
                    ])
                }

                ByteCode::PushList => {
                    let list_length = self.read_i32();
                    self.print(vec![
                        "push_list".to_string(),
                        format!("(length: {list_length})"),
                    ])
                }

                ByteCode::PushComponent => {
                    let component_id = self.read_byte();
                    let property_count = self.read_byte();
                    self.print(vec![
                        "push_component".to_string(),
                        format!("(id: {component_id}, properties: {property_count})"),
                    ])
                }

                ByteCode::SetNextComponentColumnOrJump => {
                    let jump_offset = self.read_i16();
                    self.print(vec![
                        "set_next_component_column_or_jump".to_string(),
                        format!("(offset: {jump_offset})"),
                    ])
                }

                ByteCode::GetTagName => self.print(vec!["get_tag_name".to_string()]),

                ByteCode::GetTagPayload => self.print(vec!["get_tag_payload".to_string()]),

                ByteCode::IntegerAddition => self.print(vec!["int_add".to_string()]),

                ByteCode::IntegerSubtraction => self.print(vec!["int_sub".to_string()]),

                ByteCode::IntegerMultiplication => self.print(vec!["int_mult".to_string()]),

                ByteCode::IntegerDivision => self.print(vec!["int_div".to_string()]),

                ByteCode::IntegerModulo => self.print(vec!["int_mod".to_string()]),

                ByteCode::IntegerLessThan => self.print(vec!["int_lt".to_string()]),

                ByteCode::IntegerLessThanEquals => self.print(vec!["int_lte".to_string()]),

                ByteCode::FloatAddition => self.print(vec!["float_add".to_string()]),

                ByteCode::FloatSubtraction => self.print(vec!["float_sub".to_string()]),

                ByteCode::FloatMultiplication => self.print(vec!["float_mult".to_string()]),

                ByteCode::FloatDivision => self.print(vec!["float_div".to_string()]),

                ByteCode::FloatModulo => self.print(vec!["float_mod".to_string()]),

                ByteCode::FloatLessThan => self.print(vec!["float_lt".to_string()]),

                ByteCode::FloatLessThanEquals => self.print(vec!["float_lte".to_string()]),

                ByteCode::StringConcat => self.print(vec!["str_concat".to_string()]),

                ByteCode::BooleanAnd => self.print(vec!["and".to_string()]),

                ByteCode::BooleanOr => self.print(vec!["or".to_string()]),

                ByteCode::Equals => self.print(vec!["eq".to_string()]),

                ByteCode::Negation => self.print(vec!["neg".to_string()]),

                ByteCode::Not => self.print(vec!["not".to_string()]),

                ByteCode::GetLocalValue => {
                    let index = self.program[self.program_counter]; // TODO(anissen): Make helper function to read bytes and increment program counter
                    self.program_counter += 1;
                    self.print(vec!["get_value".to_string(), format!("(index: {})", index)])
                }

                ByteCode::GetForeignValue => {
                    let name = self.read_string();

                    self.print(vec![
                        "get_foreign_value".to_string(),
                        format!("(name: {})", name),
                    ])
                }

                ByteCode::SetLocalValue => {
                    let index = self.read_byte();
                    self.print(vec!["set_value".to_string(), format!("(index: {})", index)])
                }

                ByteCode::GetFieldValue => {
                    let index = self.read_byte();
                    let field_index = self.read_byte();
                    self.print(vec![
                        "get_field_value".to_string(),
                        format!("(index: {}, field_index: {})", index, field_index),
                    ])
                }

                ByteCode::SetFieldValue => {
                    let index = self.read_byte();
                    let field_index = self.read_byte();
                    self.print(vec![
                        "set_field_value".to_string(),
                        format!("(index: {}, field_index: {})", index, field_index),
                    ])
                }

                ByteCode::GetContextValue => {
                    let name = self.read_string();
                    self.print(vec![
                        "get_context_value".to_string(),
                        format!("(name: {})", name),
                    ])
                }

                ByteCode::SetContextValue => {
                    let name = self.read_string();
                    self.print(vec![
                        "set_context_value".to_string(),
                        format!("(name: {})", name),
                    ])
                }

                ByteCode::GetListElementAtIndex => {
                    self.print(vec!["get_list_element_at_index".to_string()])
                }

                ByteCode::GetArrayLength => self.print(vec!["get_array_length".to_string()]),

                ByteCode::ArrayAppend => self.print(vec!["append".to_string()]),

                ByteCode::Log => self.print(vec!["log".to_string()]),

                ByteCode::FunctionSignature => {
                    let name = self.read_string();
                    let local_count = self.read_byte();
                    let function_position = self.read_i16();
                    self.print(vec![
                        format!("function signature"),
                        format!("(name: {})", name),
                        format!("(local count: {})", local_count),
                        format!("(function position: {})", function_position),
                    ])
                }

                ByteCode::FunctionChunk => {
                    let name = self.read_string();
                    let formatted = self.print(vec![format!("=== function chunk: {} ===", name)]);
                    format!("\n{formatted}")
                }

                ByteCode::Function => {
                    let function_index = self.read_byte();
                    let param_count = self.read_byte();
                    self.print(vec![
                        format!("function"),
                        format!("(function index: {})", function_index),
                        format!("(params: {})", param_count),
                    ])
                }

                ByteCode::Return => self.print(vec!["ret".to_string()]),

                ByteCode::Call => {
                    let arg_count = self.read_byte();
                    let is_global = self.read_byte();
                    let index = self.read_byte();
                    let name = self.read_string();

                    self.print(vec![
                        format!("call {} (is_global: {})", name, is_global),
                        format!("(arg count: {}, function index: {})", arg_count, index),
                    ])
                }

                ByteCode::CallForeign => {
                    let foreign_index = self.read_byte();
                    let arg_count = self.read_byte();
                    let name = self.read_string();

                    self.print(vec![
                        format!("call foreign function {}", name),
                        format!(
                            "(arg count: {}, foreign_index: {})",
                            arg_count, foreign_index
                        ),
                    ])
                }

                ByteCode::Jump => {
                    // let pc = self.program_counter;
                    let offset = self.read_i16();
                    self.print(vec![format!(
                        "jump (offset: {}, to byte {})",
                        offset,
                        self.program_counter as i16 + offset
                    )])
                }

                ByteCode::JumpIfTrue => {
                    let offset = self.read_i16();
                    self.print(vec![format!(
                        "jump if true (offset: {}, to byte {})",
                        offset,
                        self.program_counter + offset as usize
                    )])
                }

                ByteCode::JumpIfFalse => {
                    let offset = self.read_i16();
                    self.print(vec![format!(
                        "jump if false (offset: {}, to byte {})",
                        offset,
                        self.program_counter + offset as usize
                    )])
                }

                ByteCode::ContextQuery => {
                    let component_count = self.read_byte();
                    let mut components = Vec::new();
                    // collect all component ids and names for printing
                    for _ in 0..component_count {
                        let component_id = self.read_byte();
                        let component_name = self.read_string();
                        components.push(format!("{} ({})", component_id, component_name));
                    }
                    self.print(vec![format!("query components: {}", components.join(", "))])
                }

                ByteCode::Create => self.print(vec![format!("create entity")]),
            };

            result.push_str(&instruction_str);
        }

        metadata.disassembled_instructions = result;
    }
}
