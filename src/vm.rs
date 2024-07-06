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

#[derive(Debug, Clone)]
struct FunctionObj {
    name: String,
    arity: u8,
    code: Vec<u8>,
    ip: usize, // TODO(anissen): Is ip required?
}

#[derive(Debug)]
struct CallFrame {
    ip: usize,
    slots: u8,
    function: FunctionObj,
}

pub struct VirtualMachine {
    program: Vec<u8>,
    program_counter: usize,
    functions: Vec<FunctionObj>,
    stack: Vec<Value>,
    call_frames: Vec<CallFrame>,
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
            call_frames: Vec::new(),
        }
    }

    pub fn execute(&mut self) -> Option<Value> {
        let mut values = HashMap::new();

        // let cp = self.program.clone(); // TODO(anissen): Hack!
        while self.program_counter < self.program.len() {
            let next = self.read_byte();
            let instruction = ByteCode::try_from(next);
            // for (index, instruction) in self.program.iter().enumerate() {
            if let Ok(ByteCode::FunctionStart) = instruction {
                let _function_index = self.read_byte();
                let arity = self.read_byte();
                self.functions.push(FunctionObj {
                    name: "placeholder".to_string(),
                    arity,
                    ip: self.program_counter,
                    code: vec![], // TODO(anissen): Get the code!
                });
            }
        }
        println!("self.functions: {:?}", self.functions);

        // TODO: Construct an initial call frame for the top-level code
        self.call(
            FunctionObj {
                name: "main".to_string(),
                arity: 0,
                code: vec![],           // TODO(anissen): Get the code
                ip: self.program.len(), // TODO(anissen): Hack to avoid infinite loops
            },
            0,
        );

        self.program_counter = 0;

        while self.program_counter < self.program.len() {
            println!("=== frame === (pc: {})", self.program_counter);
            let next = self.read_byte();
            let instruction = ByteCode::try_from(next).unwrap();
            println!("Instruction: {:?}", instruction);
            println!("stack: {:?}", self.stack);
            match instruction {
                ByteCode::PushBoolean => {
                    let value_bytes = self.read_byte();
                    let value = value_bytes != 0;
                    self.stack.push(Value::Boolean(value));
                }

                ByteCode::PushInteger => {
                    let value = self.read_i32();
                    self.stack.push(Value::Integer(value));
                }

                ByteCode::PushFloat => {
                    let value = self.read_f32();
                    self.push_float(value);
                }

                ByteCode::Addition => {
                    let right = self.pop_any();
                    let left = self.pop_any();
                    match (right, left) {
                        (Value::Float(right), Value::Float(left)) => self.push_float(left + right),

                        (Value::Integer(right), Value::Integer(left)) => {
                            self.stack.push(Value::Integer(left + right))
                        }

                        _ => panic!("incompatible types for addition"),
                    }
                }

                ByteCode::Subtraction => {
                    let right = self.pop_any();
                    let left = self.pop_any();
                    match (right, left) {
                        (Value::Float(right), Value::Float(left)) => self.push_float(left - right),

                        (Value::Integer(right), Value::Integer(left)) => {
                            self.stack.push(Value::Integer(left - right))
                        }

                        _ => panic!("incompatible types for subtraction"),
                    }
                }

                ByteCode::Multiplication => {
                    let right = self.pop_any();
                    let left = self.pop_any();
                    match (right, left) {
                        (Value::Float(right), Value::Float(left)) => self.push_float(left * right),

                        (Value::Integer(right), Value::Integer(left)) => {
                            self.stack.push(Value::Integer(left * right))
                        }

                        _ => panic!("incompatible types for multiplication"),
                    }
                }

                ByteCode::Division => {
                    let right = self.pop_any();
                    let left = self.pop_any();

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
                    let index = self.read_byte();
                    println!("values: {:?}", values);
                    println!("index is: {}", index);
                    // let value = self.peek(index).unwrap(); // values.get(&index).unwrap();
                    // self.stack.push(value);
                    let value = self
                        .stack
                        .get(self.stack.len() - 1 - index as usize)
                        .unwrap();
                    self.stack.push(*value);
                }

                ByteCode::SetValue => {
                    let index = self.read_byte();
                    let value = *self.peek(0).unwrap();
                    println!("value: insert {:?} at index {}", value, index);
                    values.insert(index, value);
                }

                // https://godbolt.org/#g:!((g:!((g:!((h:codeEditor,i:(filename:'1',fontScale:14,fontUsePx:'0',j:1,lang:scala,selection:(endColumn:1,endLineNumber:16,positionColumn:1,positionLineNumber:16,selectionStartColumn:1,selectionStartLineNumber:16,startColumn:1,startLineNumber:16),source:'@main%0Adef+main()+%3D+%7B%0A++println(%22hello%22)%0A%0A++val+y+%3D+3%0A++def+twice(v:+Float)+%3D+%7B%0A++++v+*+2+%2B+y%0A++%7D%0A%0A++println(%22world%22)%0A%0A++val+x+%3D+twice(5)%0A%0A++println(x)%0A%7D%0A'),l:'5',n:'1',o:'Scala+source+%231',t:'0')),k:50,l:'4',n:'0',o:'',s:0,t:'0'),(g:!((h:compiler,i:(compiler:scalac300,filters:(b:'0',binary:'1',binaryObject:'1',commentOnly:'0',debugCalls:'1',demangle:'0',directives:'0',execute:'1',intel:'0',libraryCode:'0',trim:'1',verboseDemangling:'0'),flagsViewOpen:'1',fontScale:14,fontUsePx:'0',j:1,lang:scala,libs:!(),options:'',overrides:!(),selection:(endColumn:14,endLineNumber:65,positionColumn:14,positionLineNumber:65,selectionStartColumn:14,selectionStartLineNumber:65,startColumn:14,startLineNumber:65),source:1),l:'5',n:'0',o:'+scalac+3.0.0+(Editor+%231)',t:'0')),k:50,l:'4',n:'0',o:'',s:0,t:'0')),l:'2',n:'0',o:'',t:'0')),version:4
                ByteCode::FunctionStart => {
                    let function_index = self.read_byte();
                    println!("function_index: {}", function_index);

                    // let param_count = self.read_byte();
                    // for _param in 0..param_count {
                    //     // ???
                    // }

                    // jump to function end HACK!
                    while self.program_counter < self.program.len() {
                        let instruction = self.read_byte();
                        if let Ok(ByteCode::FunctionEnd) = ByteCode::try_from(instruction) {
                            break;
                        }
                    }

                    // self.discard(param_count);
                    self.stack.push(Value::Function(function_index));
                }

                ByteCode::FunctionEnd => {
                    // reset program counter
                    println!(
                        "slots: {}",
                        self.call_frames[self.call_frames.len() - 1].slots
                    );
                    // let slots = self.call_frames[self.call_frames.len() - 1].slots;
                    // let discard_count = slots;
                    // self.discard(discard_count); // TODO: Instead we need a per-call-frame stack
                    self.call_frames.pop();
                    let frame = &self.call_frames[self.call_frames.len() - 1];
                    self.program_counter = frame.ip;
                }

                ByteCode::Call => {
                    let arg_count = self.read_byte();
                    println!("arg_count: {}", arg_count);
                    println!("function_index: {:?}", self.peek(arg_count).unwrap());
                    let function_index = match self.peek(arg_count).unwrap() {
                        Value::Function(f) => *f,
                        _ => panic!("expected function, encountered some other type"),
                    };
                    let function = self.functions[function_index as usize].clone(); // TODO(anissen): Clone hack
                    self.call(function, arg_count)
                }
            }
            println!("stack: {:?}", self.stack);
        }
        self.stack.pop()
    }

    // TODO(anissen): Is `arg_count` required?
    fn call(&mut self, function: FunctionObj, arg_count: u8) {
        let ip = function.ip;
        self.call_frames.push(CallFrame {
            function,
            ip,
            slots: arg_count, //(self.stack.len() - 1 - (arg_count as usize)) as u8,
        });
        println!("call: {:?}", self.call_frames[self.call_frames.len() - 1]);
        self.program_counter = ip;
    }

    fn read_byte(&mut self) -> u8 {
        let byte = self.program[self.program_counter];
        self.program_counter += 1;
        println!("read_byte: {}", byte);
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
        println!("read_i32: {}", value);
        value
    }

    fn read_f32(&mut self) -> f32 {
        let raw = u32::from_be_bytes(self.read_4bytes());
        let value = f32::from_bits(raw);
        println!("read_f32: {}", value);
        value
    }

    fn pop_boolean(&mut self) -> bool {
        match self.stack.pop().unwrap() {
            Value::Boolean(b) => b,
            _ => panic!("expected boolean, encountered some other type"),
        }
    }

    fn peek(&mut self, distance: u8) -> Option<&Value> {
        self.stack.get(self.stack.len() - 1 - distance as usize)
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
