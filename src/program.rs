use std::collections::HashMap;

use crate::codegen;
use crate::lexer;
use crate::parser;
use crate::tokens::TokenKind;
use crate::vm;
use crate::vm::VirtualMachine;

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

pub struct Context<'a> {
    functions: HashMap<String, Box<dyn FnMut(f32) + 'a>>,
}

impl<'a> Context<'a> {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
        }
    }

    pub fn add_foreign_function(&mut self, n: String, c: impl FnMut(f32) + 'a) {
        self.functions.insert(n, Box::new(c));
    }

    fn call_foreign_function(&mut self, name: String) {
        if let Some(func) = self.functions.get_mut(&name) {
            func(3.4)
        }
        // self.functions.iter_mut().for_each(|(n, f)| {
        //     println!("function: {}", n);
        //     f(4.2)
        // });
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
        Ok(codegen::codegen(ast))
    }

    pub fn run(&self, bytecodes: Vec<u8>) -> Option<vm::Value> {
        vm::run(bytecodes /* , self.context */)
    }
}
