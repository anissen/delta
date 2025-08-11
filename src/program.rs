use std::collections::HashMap;

use crate::codegen;
use crate::diagnostics::Diagnostics;
use crate::disassembler;
use crate::lexer;
use crate::parser;
use crate::tokens;
use crate::typer;
use crate::vm;
use crate::CompilationMetadata;
use crate::ExecutionMetadata;
use crate::ProgramMetadata;
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

    pub fn get_value_names(&self) -> Vec<String> {
        self.values.keys().cloned().collect()
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
    // source: &'a str,
    source: String,
    debug: bool,
    pub metadata: ProgramMetadata,
    pub vm: Option<vm::VirtualMachine>,
    pub bytecode: Vec<u8>,
    pub is_valid: bool,
}

impl<'a> Program<'a> {
    pub fn new(context: Context<'a>, debug: bool) -> Self {
        Self {
            context,
            source: "".to_string(),
            debug,
            metadata: ProgramMetadata::default(),
            vm: None, //vm::VirtualMachine::new(Vec::new(), debug),
            bytecode: Vec::new(),
            is_valid: false,
        }
    }

    pub fn reload(&mut self, source: String) -> Option<Diagnostics> {
        self.source = source;
        self.compile().err()
    }

    pub fn compile(&mut self) -> Result<Vec<u8>, Diagnostics> {
        println!("\n# lexing =>");
        let start = std::time::Instant::now();
        let tokens = lexer::lex(&self.source);
        let duration = start.elapsed();
        println!("Elapsed: {duration:?}");

        let (tokens, syntax_errors): (Vec<tokens::Token>, Vec<tokens::Token>) = tokens
            .into_iter()
            .partition(|token| !matches!(token.kind, tokens::TokenKind::SyntaxError(_)));
        syntax_errors.iter().for_each(|token| match token.kind {
            tokens::TokenKind::SyntaxError(description) => {
                println!(
                    "\n⚠️ syntax error: {} at {:?} ({:?})\n",
                    description, token.lexeme, token.position
                )
            }
            _ => unreachable!(),
        });

        if self.debug {
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
        println!("Elapsed: {duration:?}");
        if self.debug {
            println!("ast: {ast:?}");
        }

        println!("\n# typing =>");
        let start = std::time::Instant::now();
        // TODO(anissen): Diagnostics should be collected in each phase
        let mut diagnostics = Diagnostics::new();
        typer::type_check(&ast, &self.context, &mut diagnostics);
        let duration = start.elapsed();
        println!("Elapsed: {duration:?}");

        if diagnostics.has_errors() {
            println!("{diagnostics}");
            return Err(diagnostics);
        }

        let foreign_functions = self
            .context
            .functions
            .keys()
            .cloned()
            .collect::<Vec<String>>();
        println!("foreign functions: {foreign_functions:?}");

        println!("\n# code gen =>");
        let start = std::time::Instant::now();
        let bytecodes = codegen::codegen(&ast, &self.context);
        let duration = start.elapsed();
        println!("Elapsed: {duration:?}");

        self.is_valid = bytecodes.is_ok();

        match bytecodes {
            Ok(bytecodes) => {
                if self.debug {
                    println!("byte code length: {}", bytecodes.len());
                    println!("byte codes: {bytecodes:?}");
                }

                let mut compilation_metadata = CompilationMetadata::default();
                compilation_metadata.bytecode = bytecodes.clone();
                compilation_metadata.bytecode_length = bytecodes.len();

                if self.debug {
                    println!("\n# disassembly =>");
                    // Generate disassembled instructions and optionally print
                    disassembler::disassemble(bytecodes.clone(), &mut compilation_metadata);
                }

                self.metadata = ProgramMetadata {
                    compilation_metadata,
                    execution_metadata: ExecutionMetadata::default(),
                };

                // TODO(anissen): Don't recreate the VM on each compile
                self.bytecode = bytecodes.clone();
                self.vm = Some(vm::VirtualMachine::new(bytecodes.clone(), self.debug));

                Ok(bytecodes)
            }
            Err(diagnostics) => Err(diagnostics),
        }
    }

    pub fn run(&mut self) -> Option<vm::Value> {
        match &mut self.vm {
            Some(vm) => {
                let result = vm.execute(None, &mut self.context);
                self.metadata.execution_metadata = vm.metadata.clone();
                result
            }
            None => None,
        }
    }

    pub fn run_function(&mut self, function_name: String) -> Option<vm::Value> {
        match &mut self.vm {
            Some(vm) => {
                let result = vm.execute(Some(function_name), &mut self.context);
                self.metadata.execution_metadata = vm.metadata.clone();
                result
            }
            None => None,
        }
    }
}
