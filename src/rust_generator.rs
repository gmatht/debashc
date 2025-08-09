use crate::ast::*;

pub struct RustGenerator {
    indent_level: usize,
}

impl RustGenerator {
    pub fn new() -> Self {
        Self { indent_level: 0 }
    }

    pub fn generate(&mut self, commands: &[Command]) -> String {
        let mut output = String::new();
        output.push_str("use std::process::Command;\n");
        output.push_str("use std::env;\n");
        output.push_str("use std::fs;\n");
        output.push_str("use std::io::{self, Write};\n");
        output.push_str("use std::thread;\n");
        output.push_str("use std::time::Duration;\n\n");
        output.push_str("fn main() -> Result<(), Box<dyn std::error::Error>> {\n");
        self.indent_level += 1;

        for command in commands {
            let chunk = self.generate_command(command);
            output.push_str(&self.indent_block(&chunk));
        }

        self.indent_level -= 1;
        output.push_str("    Ok(())\n");
        output.push_str("}\n");

        output
    }

    fn generate_command(&mut self, command: &Command) -> String {
        match command {
            Command::Simple(cmd) => self.generate_simple_command(cmd),
            Command::Pipeline(pipeline) => self.generate_pipeline(pipeline),
            Command::If(if_stmt) => self.generate_if_statement(if_stmt),
            Command::While(while_loop) => self.generate_while_loop(while_loop),
            Command::For(for_loop) => self.generate_for_loop(for_loop),
            Command::Function(func) => self.generate_function(func),
            Command::Subshell(cmd) => self.generate_subshell(cmd),
            Command::Background(cmd) => self.generate_background(cmd),
            Command::Block(block) => self.generate_block(block),
            Command::BlankLine => "\n".to_string(),
        }
    }

    fn generate_simple_command(&self, cmd: &SimpleCommand) -> String {
        let mut output = String::new();
        
        // Handle environment variables
        for (var, value) in &cmd.env_vars {
            output.push_str(&format!("env::set_var(\"{}\", \"{}\");\n", var, value));
        }

        // Generate the command
        if cmd.name == "true" {
            // Builtin true: successful no-op
            output.push_str("/* true */\n");
        } else if cmd.name == "false" {
            // Builtin false: early return with error to reflect non-zero status
            output.push_str("return Err(\"false builtin\".into());\n");
        } else if cmd.name == "echo" {
            // Special handling for echo
            if cmd.args.is_empty() {
                output.push_str("println!();\n");
            } else {
                if cmd.args.len() == 1 && cmd.args[0] == "$#" {
                    output.push_str("let argc = std::env::args().count().saturating_sub(1);\n");
                    output.push_str("println!(\"{}\", argc);\n");
                } else if cmd.args.len() == 1 && (cmd.args[0] == "$@" || cmd.args[0] == "${@}") {
                    output.push_str("let joined = std::env::args().skip(1).collect::<Vec<_>>().join(\" \" );\n");
                    output.push_str("println!(\"{}\", joined);\n");
                } else {
                    let args = cmd.args.join(" ");
                    let escaped_args = self.escape_rust_string(&args);
                    output.push_str(&format!("println!(\"{}\");\n", escaped_args));
                }
            }
        } else if cmd.name == "sleep" {
            // Use std::thread::sleep
            let dur = cmd.args.get(0).cloned().unwrap_or_else(|| "1".to_string());
            output.push_str(&format!("thread::sleep(Duration::from_secs_f64({}f64));\n", dur));
        } else if cmd.name == "cd" {
            // Special handling for cd
            let empty_string = "".to_string();
            let dir = cmd.args.first().unwrap_or(&empty_string);
            output.push_str(&format!("env::set_current_dir(\"{}\")?;\n", dir));
        } else if cmd.name == "ls" {
            // Special handling for ls
            let args = if cmd.args.is_empty() { "." } else { &cmd.args[0] };
            output.push_str(&format!("for entry in fs::read_dir(\"{}\")? {{\n", args));
            output.push_str(&self.indent());
            output.push_str("    let entry = entry?;\n");
            output.push_str(&self.indent());
            output.push_str("    let file_name = entry.file_name();\n");
            output.push_str(&self.indent());
            output.push_str("    if let Some(name) = file_name.to_str() {\n");
            output.push_str(&self.indent());
            output.push_str("        if name != \".\" && name != \"..\" {\n");
            output.push_str(&self.indent());
            output.push_str("            println!(\"{}\", name);\n");
            output.push_str(&self.indent());
            output.push_str("        }\n");
            output.push_str(&self.indent());
            output.push_str("    }\n");
            output.push_str("}\n");
        } else if cmd.name == "grep" {
            // Special handling for grep
            if cmd.args.len() >= 2 {
                let pattern = &cmd.args[0];
                let file = &cmd.args[1];
                output.push_str(&format!("let content = fs::read_to_string(\"{}\")?;\n", file));
                output.push_str("for line in content.lines() {\n");
                output.push_str(&self.indent());
                output.push_str(&format!("    if line.contains(\"{}\") {{\n", pattern));
                output.push_str(&self.indent());
                output.push_str("        println!(\"{}\", line);\n");
                output.push_str(&self.indent());
                output.push_str("    }\n");
                output.push_str("}\n");
            }
        } else if cmd.name == "cat" {
            // Special handling for cat
            for arg in &cmd.args {
                output.push_str(&format!("let content = fs::read_to_string(\"{}\")?;\n", arg));
                output.push_str("print!(\"{}\", content);\n");
            }
        } else if cmd.name == "mkdir" {
            // Special handling for mkdir
            for arg in &cmd.args {
                output.push_str(&format!("fs::create_dir_all(\"{}\")?;\n", arg));
            }
        } else if cmd.name == "rm" {
            // Special handling for rm
            for arg in &cmd.args {
                output.push_str(&format!("fs::remove_file(\"{}\")?;\n", arg));
            }
        } else if cmd.name == "mv" {
            // Special handling for mv
            if cmd.args.len() >= 2 {
                let src = &cmd.args[0];
                let dst = &cmd.args[1];
                output.push_str(&format!("fs::rename(\"{}\", \"{}\")?;\n", src, dst));
            }
        } else if cmd.name == "cp" {
            // Special handling for cp
            if cmd.args.len() >= 2 {
                let src = &cmd.args[0];
                let dst = &cmd.args[1];
                output.push_str(&format!("fs::copy(\"{}\", \"{}\")?;\n", src, dst));
            }
        } else if cmd.name == "read" {
            // Read a line from stdin into a variable
            if let Some(var) = cmd.args.get(0) {
                output.push_str(&format!("let mut {} = String::new();\n", var));
                output.push_str(&format!("io::stdin().read_line(&mut {})?;\n", var));
                output.push_str(&format!("let {v} = {v}.trim().to_string();\n", v = var));
            }
        } else {
            // Generic command
            let args_str = cmd.args.iter().map(|arg| format!("\"{}\"", arg)).collect::<Vec<_>>().join(", ");
            output.push_str(&format!("Command::new(\"{}\")\n", cmd.name));
            output.push_str(&self.indent());
            output.push_str(&format!("    .args(&[{}])\n", args_str));
            output.push_str(&self.indent());
            output.push_str("    .status()?;\n");
        }

        output
    }

    fn generate_pipeline(&mut self, pipeline: &Pipeline) -> String {
        let mut output = String::new();
        
        // Simplified: execute sequentially; no external piping
        for cmd in &pipeline.commands {
            output.push_str(&self.generate_command(cmd));
        }

        output
    }

    fn generate_if_statement(&mut self, if_stmt: &IfStatement) -> String {
        let mut output = String::new();
        
        output.push_str("if ");
        output.push_str(&self.generate_condition(&if_stmt.condition));
        output.push_str(" {\n");
        
        self.indent_level += 1;
        let then_chunk = self.generate_command(&if_stmt.then_branch);
        output.push_str(&self.indent_block(&then_chunk));
        self.indent_level -= 1;
        
        if let Some(else_branch) = &if_stmt.else_branch {
            output.push_str("} else {\n");
            self.indent_level += 1;
            let else_chunk = self.generate_command(else_branch);
            output.push_str(&self.indent_block(&else_chunk));
            self.indent_level -= 1;
        }
        
        output.push_str("}\n");
        
        output
    }

    fn generate_while_loop(&mut self, while_loop: &WhileLoop) -> String {
        let mut output = String::new();
        
        output.push_str("while ");
        output.push_str(&self.generate_condition(&while_loop.condition));
        output.push_str(" {\n");
        
        self.indent_level += 1;
        let body_chunk = self.generate_command(&while_loop.body);
        output.push_str(&self.indent_block(&body_chunk));
        self.indent_level -= 1;
        
        output.push_str("}\n");
        
        output
    }

    fn generate_for_loop(&mut self, for_loop: &ForLoop) -> String {
        let mut output = String::new();
        
        if for_loop.items.is_empty() {
            // Infinite loop
            output.push_str("loop {\n");
            self.indent_level += 1;
            let body_chunk = self.generate_command(&for_loop.body);
            output.push_str(&self.indent_block(&body_chunk));
            self.indent_level -= 1;
            output.push_str("}\n");
        } else {
            // For loop with items
            let items_str = for_loop.items.iter().map(|item| format!("\"{}\"", item)).collect::<Vec<_>>().join(", ");
            output.push_str(&format!("for {} in &[{}] {{\n", for_loop.variable, items_str));
            self.indent_level += 1;
            output.push_str(&self.indent());
            output.push_str(&self.generate_command(&for_loop.body));
            self.indent_level -= 1;
            output.push_str("}\n");
        }
        
        output
    }

    fn generate_function(&mut self, func: &Function) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("fn {}() -> Result<(), Box<dyn std::error::Error>> {{\n", func.name));
        self.indent_level += 1;
        let body_chunk = self.generate_command(&func.body);
        output.push_str(&self.indent_block(&body_chunk));
        self.indent_level -= 1;
        output.push_str("    Ok(())\n");
        output.push_str("}\n");
        
        output
    }

    fn generate_subshell(&mut self, command: &Command) -> String {
        let mut output = String::new();
        // Run subshell inline (foreground)
        output.push_str("{\n");
        self.indent_level += 1;
        let inner_chunk = self.generate_command(command);
        output.push_str(&self.indent_block(&inner_chunk));
        self.indent_level -= 1;
        output.push_str("}\n");
        output
    }

    fn generate_background(&mut self, command: &Command) -> String {
        let mut output = String::new();
        // Spawn in a background thread
        output.push_str("let _ = thread::spawn(|| {\n");
        output.push_str("    let _ = (|| -> Result<(), Box<dyn std::error::Error>> {\n");
        self.indent_level += 1;
        let inner_chunk = self.generate_command(command);
        output.push_str(&self.indent_block(&inner_chunk));
        output.push_str(&self.indent());
        output.push_str("Ok(())\n");
        self.indent_level -= 1;
        output.push_str("    })();\n");
        output.push_str("});\n");
        output
    }

    fn generate_block(&mut self, block: &Block) -> String {
        let mut output = String::new();
        for cmd in &block.commands {
            output.push_str(&self.generate_command(cmd));
        }
        output
    }

    fn generate_condition(&self, command: &Command) -> String {
        // For now, implement a simple condition check
        match command {
            Command::Simple(cmd) => {
                if cmd.name == "[" || cmd.name == "test" {
                    if let Some(test_op) = cmd.args.get(0) {
                        match test_op.as_str() {
                            "-f" => {
                                if let Some(file) = cmd.args.get(1) {
                                    return format!("fs::metadata(\"{}\").is_ok()", file);
                                }
                            }
                            "-d" => {
                                if let Some(dir) = cmd.args.get(1) {
                                    return format!("fs::metadata(\"{}\").map(|m| m.is_dir()).unwrap_or(false)", dir);
                                }
                            }
                            "-e" => {
                                if let Some(path) = cmd.args.get(1) {
                                    return format!("fs::metadata(\"{}\").is_ok()", path);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                "true".to_string()
            }
            _ => "true".to_string(),
        }
    }

    fn indent(&self) -> String {
        "    ".repeat(self.indent_level)
    }
    
    fn indent_block(&self, s: &str) -> String {
        let prefix = self.indent();
        let mut out = String::new();
        for line in s.lines() {
            out.push_str(&prefix);
            out.push_str(line);
            out.push('\n');
        }
        out
    }
    
    fn escape_rust_string(&self, s: &str) -> String {
        // First, unescape any \" sequences to " to avoid double-escaping
        let unescaped = s.replace("\\\"", "\"");
        // Then escape quotes and other characters for Rust
        unescaped.replace("\\", "\\\\")
                 .replace("\"", "\\\"")
                 .replace("\n", "\\n")
                 .replace("\r", "\\r")
                 .replace("\t", "\\t")
    }
}




