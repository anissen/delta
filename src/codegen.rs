use std::collections::{HashMap, HashSet};

use crate::bytecodes::ByteCode;
use crate::expressions::{BinaryOperator, Expr, UnaryOperator};

pub struct Codegen {
    bytes: Vec<u8>,
    function_count: u8,
}

pub fn codegen(expressions: Vec<Expr>) -> Vec<u8> {
    Codegen::new().emit(expressions)
}

// TODO(anissen): Add a function overview mapping for each scope containing { name, arity, starting IP, source line number  }.
// This will be used directly in the VM as well as for debug logging.

impl Codegen {
    fn new() -> Self {
        Self {
            bytes: vec![],
            function_count: 0,
        }
    }

    // TODO(anissen): We need a mapping from variable to an index

    fn do_emit(
        &mut self,
        expressions: Vec<Expr>,
        environment: &mut HashMap<String, u8>,
        locals: &mut HashSet<String>,
    ) {
        // TODO(anissen): Make this a proper return type.
        for expr in expressions {
            println!("do_emit with expr: {:?}", expr);
            match expr {
                Expr::Boolean(b) => {
                    self.emit_bytecode(ByteCode::PushBoolean);
                    self.emit_byte(b as u8);
                }

                Expr::Integer(i) => self.emit_bytes(ByteCode::PushInteger, i.to_be_bytes()),

                Expr::Float(f) => self.emit_bytes(ByteCode::PushFloat, f.to_be_bytes()),

                Expr::Variable(name) => {
                    println!("read variable: {}", name);
                    let index = environment.get(&name).unwrap();
                    // println!("locals: {:?}", locals);
                    // println!("does locals contain {}?", name);
                    // if locals.contains(&name) {
                    // println!("yes!");
                    self.emit_bytecode(ByteCode::GetLocalValue);
                    // } else {
                    //     println!("no!");
                    //     println!("environment: {:?}", environment);
                    //     self.emit_bytecode(ByteCode::GetGlobalValue);
                    // }
                    self.emit_byte(*index);
                }

                Expr::Grouping(expr) => self.do_emit(vec![*expr], environment, locals),

                Expr::Block { exprs } => self.do_emit(exprs, environment, locals),

                Expr::Function { params, expr } => {
                    let mut function_environment = environment.clone();
                    let mut function_locals = HashSet::new();

                    self.emit_bytecode(ByteCode::FunctionStart);

                    let mut index = 0;
                    for param in params.iter() {
                        println!("set param: {}", param.lexeme);
                        function_environment.insert(param.lexeme.clone(), index);
                        index += 1;

                        function_locals.insert(param.lexeme.clone());
                    }
                    // bytecodes: function start, function index, param count, function body, function end

                    self.emit_byte(self.function_count);
                    self.function_count += 1;

                    self.emit_byte(params.len() as u8); // TODO(anissen): Guard against overflow

                    // emit function signatures here?
                    // e.g.
                    // function_signatures = []
                    // bytes = emit(expr, ...) // also populates function_signatures
                    // write_bytes(function_signatures)
                    // write_bytes(bytes)

                    println!("function_environment: {:?}", function_environment);
                    self.do_emit(vec![*expr], &mut function_environment, &mut function_locals);

                    self.emit_bytecode(ByteCode::FunctionEnd);
                }

                Expr::Call { name, args } => {
                    let arg_count = args.len(); // + 1 /* pre-argument */;
                    self.do_emit(args, environment, locals);
                    self.emit_bytecode(ByteCode::Call);
                    self.emit_byte(arg_count as u8);
                    // self.emit_byte(*self.value_index.get(&name).unwrap());
                    println!(
                        "call function '{}' with environment: {:?}",
                        name, environment
                    );
                    let index = environment.get(&name).unwrap();
                    // println!("*** call locals: {:?}", locals);
                    if locals.contains(&name) {
                        self.emit_byte(0);
                    } else {
                        self.emit_byte(1);
                    }
                    self.emit_byte(*index);

                    if name.len() > 64 {
                        panic!("function name too long!");
                    }
                    self.emit_byte(name.len() as u8);
                    self.emit_raw_bytes(&mut name.as_bytes().to_vec());
                }

                Expr::Assignment {
                    variable,
                    token: _,
                    expr,
                } => {
                    println!("assignment with environment: {:?}", environment);
                    self.do_emit(vec![*expr], environment, locals);
                    self.emit_bytecode(ByteCode::SetLocalValue);

                    let index = environment.len() as u8;
                    println!("insert variable {} at index {}", variable, index);
                    environment.insert(variable.clone(), index);
                    locals.insert(variable.clone());
                    self.emit_byte(index);
                }

                Expr::Unary {
                    operator,
                    token: _,
                    expr,
                } => match operator {
                    UnaryOperator::Negation => {
                        self.do_emit(vec![*expr], environment, locals);
                        self.emit_bytecode(ByteCode::Negation);
                    }

                    UnaryOperator::Not => {
                        self.do_emit(vec![*expr], environment, locals);
                        self.emit_bytecode(ByteCode::Not);
                    }
                },

                Expr::Binary {
                    left,
                    operator,
                    token: _,
                    right,
                } => {
                    self.do_emit(vec![*left, *right], environment, locals);
                    match operator {
                        BinaryOperator::Addition => self.emit_bytecode(ByteCode::Addition),

                        BinaryOperator::Subtraction => self.emit_bytecode(ByteCode::Subtraction),

                        BinaryOperator::Multiplication => {
                            self.emit_bytecode(ByteCode::Multiplication)
                        }

                        BinaryOperator::Division => self.emit_bytecode(ByteCode::Division),
                    }
                }
            };
        }
    }

    pub fn emit(&mut self, expressions: Vec<Expr>) -> Vec<u8> {
        println!("emit with fresh environment");
        self.do_emit(expressions, &mut HashMap::new(), &mut HashSet::new());
        self.bytes.clone()
    }

    fn emit_byte(&mut self, byte: u8) {
        self.bytes.push(byte);
    }

    fn emit_bytecode(&mut self, code: ByteCode) {
        self.emit_byte(code.into());
    }

    fn emit_bytes(&mut self, code: ByteCode, value: [u8; 4]) {
        self.emit_bytecode(code);
        for byte in value {
            self.emit_byte(byte);
        }
    }

    fn emit_raw_bytes(&mut self, bytes: &mut Vec<u8>) {
        self.bytes.append(bytes);
    }
}
