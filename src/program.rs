use std::collections::HashMap;

use crate::codegen;
use crate::diagnostics;
use crate::diagnostics::Diagnostics;
use crate::lexer;
use crate::parser;
use crate::tokens;
use crate::typer;
use crate::vm;
// use crate::vm::VirtualMachine;

// struct CallContext<'a> {
//     vm: &'a mut VirtualMachine,
// }

// impl<'a> CallContext<'a> {
//     pub fn new(vm: &'a mut VirtualMachine) -> Self {
//         Self { vm }
//     }

//     pub fn pop_float(&mut self) -> f32 {
//         self.vm.pop_float()
//     }
// }

type ForeignValue<'a> = Box<dyn Fn() -> vm::Value + 'a>;
type ForeignFn<'a> = Box<dyn Fn(&Vec<vm::Value>) -> vm::Value + 'a>;

struct ForeignFunction<'a> {
    index: u8,
    function: ForeignFn<'a>,
}

pub struct Context<'a> {
    functions: HashMap<String, ForeignFunction<'a>>,
    function_count: u8,
    values: HashMap<String, ForeignValue<'a>>,
}

impl Default for Context<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Context<'a> {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            function_count: 0,
            values: HashMap::new(),
        }
    }

    pub fn add_value(&mut self, name: String, value: impl Fn() -> vm::Value + 'a) {
        self.values.insert(name, Box::new(value));
    }

    pub fn has_value(&self, name: &String) -> bool {
        self.values.contains_key(name)
    }

    pub fn get_value(&self, name: &String) -> vm::Value {
        if let Some(value_func) = self.values.get(name) {
            value_func()
        } else {
            vm::Value::False
        }
    }

    pub fn add_function(
        &mut self,
        name: String,
        function: impl Fn(&Vec<vm::Value>) -> vm::Value + 'a,
    ) {
        self.functions.insert(
            name,
            ForeignFunction {
                index: self.function_count,
                function: Box::new(function),
            },
        );
        self.function_count += 1;
    }

    pub fn has_function(&self, name: &String) -> bool {
        self.functions.contains_key(name)
    }

    pub fn get_index(&self, name: &String) -> u8 {
        self.functions.get(name).unwrap().index
    }

    pub fn get_function_names(&self) -> Vec<String> {
        self.functions.keys().cloned().collect::<Vec<String>>()
    }

    pub fn call_function(&self, name: &String, stack: &Vec<vm::Value>) -> vm::Value {
        if let Some(foreign) = self.functions.get(name) {
            let func = &foreign.function;
            func(stack)
        } else {
            vm::Value::False // TODO(anissen): Should this be an error?
        }
    }
}

pub struct Program<'a> {
    context: Context<'a>,
}

impl<'a> Program<'a> {
    pub fn new(context: Context<'a>) -> Self {
        Self { context }
    }

    pub fn compile(&self, source: &str, debug: bool) -> Result<Vec<u8>, Diagnostics> {
        println!("\n# lexing =>");
        let start = std::time::Instant::now();
        let tokens = lexer::lex(source);
        let duration = start.elapsed();
        println!("Elapsed: {:?}", duration);

        let (tokens, syntax_errors): (Vec<tokens::Token>, Vec<tokens::Token>) =
            tokens.into_iter().partition(|token| match token.kind {
                tokens::TokenKind::SyntaxError(_) => false,
                _ => true,
            });
        syntax_errors.iter().for_each(|token| match token.kind {
            tokens::TokenKind::SyntaxError(description) => {
                println!(
                    "\n⚠️ syntax error: {} at {:?} ({:?})\n",
                    description, token.lexeme, token.position
                )
            }
            _ => panic!(),
        });

        if debug {
            tokens.iter().for_each(|token| {
                println!(
                    "token: {:?} at '{}' (line {}, column: {})",
                    token.kind, token.lexeme, token.position.line, token.position.column
                )
            });
        }

        println!("\n# parsing =>");
        let start = std::time::Instant::now();
        let ast = parser::parse(tokens)?;
        let duration = start.elapsed();
        println!("Elapsed: {:?}", duration);
        if debug {
            println!("ast: {:?}", ast);
        }

        println!("\n# typing =>");
        let start = std::time::Instant::now();
        // TODO(anissen): Diagnostics should be collected in each phase
        let mut diagnostics = Diagnostics::new();
        typer::type_check(&ast, &self.context, &mut diagnostics);
        let duration = start.elapsed();
        println!("Elapsed: {:?}", duration);

        if diagnostics.has_errors() {
            eprintln!("Errors: {}", diagnostics.to_string());
        }

        let foreign_functions = self
            .context
            .functions
            .keys()
            .cloned()
            .collect::<Vec<String>>();
        println!("foreign functions: {:?}", foreign_functions);

        println!("\n# code gen =>");
        let start = std::time::Instant::now();
        let bytecodes = codegen::codegen(&ast, &self.context);
        let duration = start.elapsed();
        println!("Elapsed: {:?}", duration);

        match bytecodes {
            Ok(bytecodes) => {
                if debug {
                    println!("byte code length: {}", bytecodes.len());
                    println!("byte codes: {:?}", bytecodes);
                }
                Ok(bytecodes)
            }
            Err(diagnostics) => {
                eprintln!("Errors: {}", diagnostics.to_string());
                Err(diagnostics)
            }
        }
    }

    pub fn run(&self, bytecodes: Vec<u8>, debug: bool) -> Option<vm::Value> {
        vm::run(bytecodes, &self.context, debug)
    }
}
