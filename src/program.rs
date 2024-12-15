use std::collections::HashMap;

use crate::codegen;
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

struct ForeignFunction<'a> {
    index: u8,
    function: Box<dyn Fn(&Vec<vm::Value>) + 'a>,
}

pub struct Context<'a> {
    functions: HashMap<String, ForeignFunction<'a>>,
    function_count: u8,
}

impl<'a> Context<'a> {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            function_count: 0,
        }
    }

    pub fn add_foreign_function(&mut self, name: String, function: impl Fn(&Vec<vm::Value>) + 'a) {
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
        self.functions
            .iter()
            .map(|(k, _v)| k.clone())
            .collect::<Vec<String>>()
    }

    pub fn call_foreign_function(&self, name: String, stack: &Vec<vm::Value>) {
        if let Some(foreign) = self.functions.get(&name) {
            let func = &foreign.function;
            func(stack);
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

    pub fn compile(&self, source: &String) -> Result<Vec<u8>, String> {
        let tokens = lexer::lex(&source);
        let non_error_tokens = tokens
            .into_iter()
            .filter(|t| !matches!(t.kind, TokenKind::SyntaxError(_)))
            .collect();
        let ast = parser::parse(non_error_tokens)?;
        // context.call_foreign_function("dummy".to_string());
        let foreign_functions = self
            .context
            .functions
            .iter()
            .map(|(k, _v)| k.clone())
            .collect::<Vec<String>>();
        println!("foreign functions: {:?}", foreign_functions);
        Ok(codegen::codegen(ast, &self.context))
    }

    pub fn run(&self, bytecodes: Vec<u8>) -> Option<vm::Value> {
        vm::run(bytecodes, &self.context)
    }
}
