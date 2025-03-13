use std::collections::HashMap;

use crate::codegen;
use crate::diagnostics::Diagnostics;
use crate::lexer;
use crate::parser;
use crate::tokens::TokenKind;
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

    pub fn compile(&self, source: &str) -> Result<Vec<u8>, String> {
        let mut diagnostics = Diagnostics::new();
        let tokens = lexer::lex(source);
        let non_error_tokens = tokens
            .into_iter()
            .filter(|t| !matches!(t.kind, TokenKind::SyntaxError(_)))
            .collect();
        let ast = parser::parse(non_error_tokens)?;
        let foreign_functions = self
            .context
            .functions
            .keys()
            .cloned()
            .collect::<Vec<String>>();
        println!("foreign functions: {:?}", foreign_functions);
        Ok(codegen::codegen(&ast, &self.context, &mut diagnostics))
    }

    pub fn run(&self, bytecodes: Vec<u8>) -> Option<vm::Value> {
        vm::run(bytecodes, &self.context)
    }
}
