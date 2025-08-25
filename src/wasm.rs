use wasm_bindgen::prelude::*;
use crate::{Lexer, Parser, Generator};

#[wasm_bindgen]
pub struct Debashc;

#[wasm_bindgen]
impl Debashc {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self
    }

    /// Get all examples as JSON - now handled by JavaScript
    pub fn get_examples(&self) -> String {
        "[]".to_string()
    }

    /// Get a specific example by name - now handled by JavaScript
    pub fn get_example(&self, _name: &str) -> Result<String, JsValue> {
        Err(JsValue::from_str("Examples are now handled by JavaScript. Use the examples.js module instead."))
    }

    /// Get the total number of examples - now handled by JavaScript
    pub fn get_example_count(&self) -> usize {
        0
    }

    /// Get all example names as a JSON array - now handled by JavaScript
    pub fn get_example_names(&self) -> String {
        "[]".to_string()
    }

    /// Tokenize a shell script
    pub fn lex(&self, input: &str) -> Result<String, JsValue> {
        let mut lexer = Lexer::new(input);
        let mut tokens = Vec::new();
        
        while let Some(token) = lexer.peek() {
            let token_text = lexer.get_current_text().unwrap_or_else(|| "".to_string());
            tokens.push(format!("{:?}('{}')", token, token_text));
            lexer.next(); // Advance to next token
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
                let mut generator = Generator::new();
                Ok(generator.generate(&commands))
            }
            Err(e) => Err(JsValue::from_str(&format!("Parse error: {}", e))),
        }
    }


}

#[wasm_bindgen]
pub fn examples_json() -> String {
    "[]".to_string()
}

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}
