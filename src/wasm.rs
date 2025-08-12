use wasm_bindgen::prelude::*;
use crate::{Lexer, Parser, PerlGenerator};
use serde::Serialize;

#[wasm_bindgen]
pub struct Debashc;

#[wasm_bindgen]
impl Debashc {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self
    }

    /// Get all examples as JSON
    pub fn get_examples(&self) -> String {
        examples_json()
    }

    /// Get a specific example by name
    pub fn get_example(&self, name: &str) -> Result<String, JsValue> {
        let examples = all_examples();
        for example in examples {
            if example.name == name {
                return Ok(example.content.to_string());
            }
        }
        Err(JsValue::from_str(&format!("Example '{}' not found", name)))
    }

    /// Get the total number of examples
    pub fn get_example_count(&self) -> usize {
        all_examples().len()
    }

    /// Get all example names as a JSON array
    pub fn get_example_names(&self) -> String {
        let names: Vec<&str> = all_examples().iter().map(|e| e.name).collect();
        serde_json::to_string(&names).unwrap_or_else(|_| "[]".to_string())
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
    pub fn to_rust(&mut self, _input: &str) -> Result<String, JsValue> {
        Err(JsValue::from_str("Rust generator not available"))
    }

    /// Convert shell script to Python
    pub fn to_python(&mut self, _input: &str) -> Result<String, JsValue> {
        Err(JsValue::from_str("Python generator not available"))
    }

    /// Convert shell script to Lua
    pub fn to_lua(&mut self, _input: &str) -> Result<String, JsValue> {
        Err(JsValue::from_str("Lua generator not available"))
    }

    /// Convert shell script to C
    pub fn to_c(&mut self, _input: &str) -> Result<String, JsValue> {
        Err(JsValue::from_str("C generator not available"))
    }

    /// Convert shell script to JavaScript
    pub fn to_js(&mut self, _input: &str) -> Result<String, JsValue> {
        Err(JsValue::from_str("JS generator not available"))
    }

    /// Convert shell script to English pseudocode
    pub fn to_english(&mut self, _input: &str) -> Result<String, JsValue> {
        Err(JsValue::from_str("English generator not available"))
    }

    /// Convert shell script to French pseudocode
    pub fn to_french(&mut self, _input: &str) -> Result<String, JsValue> {
        Err(JsValue::from_str("French generator not available"))
    }

    /// Convert shell script to Windows Batch
    pub fn to_bat(&mut self, _input: &str) -> Result<String, JsValue> {
        Err(JsValue::from_str("Batch generator not available"))
    }

    /// Convert shell script to PowerShell
    pub fn to_ps(&mut self, _input: &str) -> Result<String, JsValue> {
        Err(JsValue::from_str("PowerShell generator not available"))
    }
}

#[derive(Serialize)]
struct ExampleEntry {
    name: &'static str,
    content: &'static str,
}

fn all_examples() -> Vec<ExampleEntry> {
    vec![
        // Basic examples
        ExampleEntry { name: "args.sh", content: include_str!("../examples/args.sh") },
        ExampleEntry { name: "simple.sh", content: include_str!("../examples/simple.sh") },
        ExampleEntry { name: "simple_backup.sh", content: include_str!("../examples/simple_backup.sh") },
        ExampleEntry { name: "misc.sh", content: include_str!("../examples/misc.sh") },
        ExampleEntry { name: "subprocess.sh", content: include_str!("../examples/subprocess.sh") },
        ExampleEntry { name: "test_quoted.sh", content: include_str!("../examples/test_quoted.sh") },
        ExampleEntry { name: "cat_EOF.sh", content: include_str!("../examples/cat_EOF.sh") },
        ExampleEntry { name: "file.txt", content: include_str!("../examples/file.txt") },
        ExampleEntry { name: "cd..sh", content: include_str!("../examples/cd..sh") },
        ExampleEntry { name: "test_ls_star_dot_sh.sh", content: include_str!("../examples/test_ls_star_dot_sh.sh") },
        
        // Control flow examples
        ExampleEntry { name: "control_flow.sh", content: include_str!("../examples/control_flow.sh") },
        ExampleEntry { name: "control_flow_if.sh", content: include_str!("../examples/control_flow_if.sh") },
        ExampleEntry { name: "control_flow_loops.sh", content: include_str!("../examples/control_flow_loops.sh") },
        ExampleEntry { name: "control_flow_function.sh", content: include_str!("../examples/control_flow_function.sh") },
        
        // Pipeline examples
        ExampleEntry { name: "pipeline.sh", content: include_str!("../examples/pipeline.sh") },
        
        // Local and environment examples
        ExampleEntry { name: "local.sh", content: include_str!("../examples/local.sh") },
        
        // Parameter expansion examples
        ExampleEntry { name: "parameter_expansion.sh", content: include_str!("../examples/parameter_expansion.sh") },
        ExampleEntry { name: "parameter_expansion_advanced.sh", content: include_str!("../examples/parameter_expansion_advanced.sh") },
        ExampleEntry { name: "parameter_expansion_case.sh", content: include_str!("../examples/parameter_expansion_case.sh") },
        ExampleEntry { name: "parameter_expansion_defaults.sh", content: include_str!("../examples/parameter_expansion_defaults.sh") },
        ExampleEntry { name: "parameter_expansion_more.sh", content: include_str!("../examples/parameter_expansion_more.sh") },
        
        // Brace expansion examples
        ExampleEntry { name: "brace_expansion.sh", content: include_str!("../examples/brace_expansion.sh") },
        ExampleEntry { name: "brace_expansion_basic.sh", content: include_str!("../examples/brace_expansion_basic.sh") },
        ExampleEntry { name: "brace_expansion_advanced.sh", content: include_str!("../examples/brace_expansion_advanced.sh") },
        ExampleEntry { name: "brace_expansion_practical.sh", content: include_str!("../examples/brace_expansion_practical.sh") },
        
        // Arrays examples
        ExampleEntry { name: "arrays.sh", content: include_str!("../examples/arrays.sh") },
        ExampleEntry { name: "arrays_indexed.sh", content: include_str!("../examples/arrays_indexed.sh") },
        ExampleEntry { name: "arrays_associative.sh", content: include_str!("../examples/arrays_associative.sh") },
        
        // Pattern matching examples
        ExampleEntry { name: "pattern_matching.sh", content: include_str!("../examples/pattern_matching.sh") },
        ExampleEntry { name: "pattern_matching_basic.sh", content: include_str!("../examples/pattern_matching_basic.sh") },
        ExampleEntry { name: "pattern_matching_extglob.sh", content: include_str!("../examples/pattern_matching_extglob.sh") },
        ExampleEntry { name: "pattern_matching_nocase.sh", content: include_str!("../examples/pattern_matching_nocase.sh") },
        
        // Process substitution examples
        ExampleEntry { name: "process_substitution.sh", content: include_str!("../examples/process_substitution.sh") },
        ExampleEntry { name: "process_substitution_advanced.sh", content: include_str!("../examples/process_substitution_advanced.sh") },
        ExampleEntry { name: "process_substitution_comm.sh", content: include_str!("../examples/process_substitution_comm.sh") },
        ExampleEntry { name: "process_substitution_mapfile.sh", content: include_str!("../examples/process_substitution_mapfile.sh") },
        ExampleEntry { name: "process_substitution_here.sh", content: include_str!("../examples/process_substitution_here.sh") },
        
        // ANSI quoting examples
        ExampleEntry { name: "ansi_quoting.sh", content: include_str!("../examples/ansi_quoting.sh") },
        ExampleEntry { name: "ansi_quoting_basic.sh", content: include_str!("../examples/ansi_quoting_basic.sh") },
        ExampleEntry { name: "ansi_quoting_escape.sh", content: include_str!("../examples/ansi_quoting_escape.sh") },
        ExampleEntry { name: "ansi_quoting_practical.sh", content: include_str!("../examples/ansi_quoting_practical.sh") },
        ExampleEntry { name: "ansi_quoting_unicode.sh", content: include_str!("../examples/ansi_quoting_unicode.sh") },
        
        // Grep examples (from todo directory)
        ExampleEntry { name: "grep_basic.sh", content: include_str!("../examples/todo/grep_basic.sh") },
        ExampleEntry { name: "grep_advanced.sh", content: include_str!("../examples/todo/grep_advanced.sh") },
        ExampleEntry { name: "grep_context.sh", content: include_str!("../examples/todo/grep_context.sh") },
        ExampleEntry { name: "grep_params.sh", content: include_str!("../examples/todo/grep_params.sh") },
        ExampleEntry { name: "grep_regex.sh", content: include_str!("../examples/todo/grep_regex.sh") },
    ]
}

#[wasm_bindgen]
pub fn examples_json() -> String {
    serde_json::to_string(&all_examples()).unwrap_or_else(|_| "[]".to_string())
}

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}
