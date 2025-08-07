use wasm_bindgen::prelude::*;
use crate::{Lexer, Parser, PerlGenerator, RustGenerator, PythonGenerator};

#[wasm_bindgen]
pub struct Debashc;

#[wasm_bindgen]
impl Debashc {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self
    }

    /// Tokenize a shell script
    pub fn lex(&self, input: &str) -> Result<String, JsValue> {
        let mut lexer = Lexer::new(input);
        let mut tokens = Vec::new();
        
        while let Some(token) = lexer.next() {
            tokens.push(format!("{:?}", token));
        }
        
        Ok(tokens.join("\n"))
    }

    /// Parse a shell script to AST
    pub fn parse(&mut self, input: &str) -> Result<String, JsValue> {
        let mut parser = Parser::new(input);
        match parser.parse() {
            Ok(commands) => {
                let mut result = String::new();
                for (i, command) in commands.iter().enumerate() {
                    result.push_str(&format!("Command {}: {:?}\n", i + 1, command));
                }
                Ok(result)
            }
            Err(e) => Err(JsValue::from_str(&format!("Parse error: {}", e))),
        }
    }

    /// Convert shell script to Perl
    pub fn to_perl(&mut self, input: &str) -> Result<String, JsValue> {
        let mut parser = Parser::new(input);
        match parser.parse() {
            Ok(commands) => {
                let mut generator = PerlGenerator::new();
                Ok(generator.generate(&commands))
            }
            Err(e) => Err(JsValue::from_str(&format!("Parse error: {}", e))),
        }
    }

    /// Convert shell script to Rust
    pub fn to_rust(&mut self, input: &str) -> Result<String, JsValue> {
        let mut parser = Parser::new(input);
        match parser.parse() {
            Ok(commands) => {
                let mut generator = RustGenerator::new();
                Ok(generator.generate(&commands))
            }
            Err(e) => Err(JsValue::from_str(&format!("Parse error: {}", e))),
        }
    }

    /// Convert shell script to Python
    pub fn to_python(&mut self, input: &str) -> Result<String, JsValue> {
        let mut parser = Parser::new(input);
        match parser.parse() {
            Ok(commands) => {
                let mut generator = PythonGenerator::new();
                Ok(generator.generate(&commands))
            }
            Err(e) => Err(JsValue::from_str(&format!("Parse error: {}", e))),
        }
    }
}

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}
