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
        let mut needs_command = false;
        let mut needs_env = false;
        let mut needs_fs = false;
        let mut needs_io = false;
        let mut needs_thread = false;
        let mut needs_duration = false;
        let mut has_early_return = false;

        // First pass - analyze what imports we need and check for early returns
        for command in commands {
            match command {
                Command::Simple(cmd) => {
                    if cmd.name == "false" {
                        has_early_return = true;
                    }
                    if cmd.name != "echo" && cmd.name != "true" && cmd.name != "false" {
                        needs_command = true;
                    }
                    if cmd.name == "cd" || !cmd.env_vars.is_empty() {
                        needs_env = true;
                    }
                    if cmd.name == "cat" || cmd.name == "ls" || cmd.name == "grep" || cmd.name == "wc" || cmd.name == "sort" || cmd.name == "uniq" || cmd.name == "find" || cmd.name == "xargs" {
                        needs_fs = true;
                    }
                    if cmd.name == "read" {
                        needs_io = true;
                    }
                    if cmd.name == "sleep" {
                        needs_thread = true;
                        needs_duration = true;
                    }
                }
                Command::If(_) => {
                    needs_fs = true; // If statements often use file tests
                }
                Command::While(_) => {
                    needs_env = true; // While loops often use variables
                }
                Command::For(_) => {
                    needs_env = true; // For loops often use variables
                }
                Command::Pipeline(_) => {
                    needs_command = true; // Pipelines use external commands
                    needs_fs = true; // Pipelines often involve file operations
                }
                Command::Background(_) => {
                    needs_thread = true;
                    needs_duration = true; // Background commands often use sleep
                }
                _ => {}
            }
        }

        // Add only needed imports
        if needs_command {
            output.push_str("use std::process::Command;\n");
        }
        if needs_env {
            output.push_str("use std::env;\n");
        }
        if needs_fs {
            output.push_str("use std::fs;\n");
        }
        if needs_io {
            output.push_str("use std::io::{self, Write};\n");
        }
        if needs_thread {
            output.push_str("use std::thread;\n");
        }
        if needs_duration {
            output.push_str("use std::time::Duration;\n");
        }
        if output.ends_with('\n') {
            output.push('\n');
        }

        output.push_str("fn main() -> std::process::ExitCode {\n");
        self.indent_level += 1;

        for command in commands {
            let chunk = self.generate_command(command);
            output.push_str(&self.indent_block(&chunk));
        }

        self.indent_level -= 1;
        // Only add success return if there's no early return
        if !has_early_return {
            output.push_str("    std::process::ExitCode::SUCCESS\n");
        }
        output.push_str("}\n");

        while output.ends_with('\n') { output.pop(); }
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
        
        // Handle variable assignments (e.g., i=5)
        if cmd.name.to_string().contains('=') {
            let name_str = cmd.name.to_string();
            let parts: Vec<&str> = name_str.splitn(2, '=').collect();
            if parts.len() == 2 {
                let var_name = &parts[0];
                let var_value = &parts[1];
                output.push_str(&format!("env::set_var(\"{}\", \"{}\");\n", var_name, var_value));
                return output;
            }
        }

        // Generate the command
        if cmd.name == "true" {
            // Builtin true: successful no-op
            output.push_str("/* true */\n");
        } else if cmd.name == "false" {
            // Builtin false: early return with error to reflect non-zero status
            output.push_str("return std::process::ExitCode::FAILURE;\n");
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
                    // Check if we have any variables to expand
                    let has_vars = cmd.args.iter().any(|arg| Self::extract_var_name(arg).is_some());
                    
                    if !has_vars && cmd.args.len() == 1 {
                        // Simple case: single literal string
                        let escaped = self.escape_rust_string(&cmd.args[0]);
                        output.push_str(&format!("println!(\"{}\");\n", escaped));
                    } else if !has_vars {
                        // Multiple literal strings - join them with space
                        let escaped = cmd.args.iter()
                            .map(|arg| self.escape_rust_string(arg))
                            .collect::<Vec<_>>()
                            .join(" ");
                        output.push_str(&format!("println!(\"{}\");\n", escaped));
                    } else {
                        // Complex case with variables - use Vec approach
                        output.push_str("let __echo_parts: Vec<String> = vec![\n");
                        for arg in &cmd.args {
                            if let Some(name) = Self::extract_var_name(arg) {
                                output.push_str(&format!("    std::env::var(\"{}\").unwrap_or_default(),\n", name));
                            } else {
                                let escaped = self.escape_rust_string(arg);
                                output.push_str(&format!("    \"{}\".to_string(),\n", escaped));
                            }
                        }
                        output.push_str("];\n");
                        output.push_str("println!(\"{}\", __echo_parts.join(\" \"));\n");
                    }
                    // Ensure success status
                    //output.push_str("/* success */\n");
                }
            }
        } else if cmd.name == "[[" {
            // Builtin [[ test: succeed (no-op)
            output.push_str("/* [[ test */\n");
        } else if cmd.name == "shopt" {
            // Builtin shopt: ignore
            output.push_str("/* builtin */\n");
        } else if cmd.name == "sleep" {
            // Use std::thread::sleep
            let dur = cmd.args.get(0).cloned().unwrap_or_else(|| Word::Literal("1".to_string()));
            output.push_str(&format!("thread::sleep(Duration::from_secs_f64({}f64));\n", dur));
        } else if cmd.name == "cd" {
            // Special handling for cd
            let dir = if cmd.args.is_empty() { ".".to_string() } else { cmd.args[0].to_string() };
            output.push_str(&format!("if let Err(_) = env::set_current_dir(\"{}\") {{\n", dir));
            output.push_str(&self.indent());
            output.push_str("    return std::process::ExitCode::FAILURE;\n");
            output.push_str("}\n");
        } else if cmd.name == "ls" {
            // Special handling for ls
            let args = if cmd.args.is_empty() { "." } else { &cmd.args[0].to_string() };
            output.push_str(&format!("match fs::read_dir(\"{}\") {{\n", args));
            output.push_str(&self.indent());
            output.push_str("    Ok(entries) => {\n");
            output.push_str(&self.indent());
            output.push_str("        for entry in entries {\n");
            output.push_str(&self.indent());
            output.push_str("            if let Ok(entry) = entry {\n");
            output.push_str(&self.indent());
            output.push_str("                let file_name = entry.file_name();\n");
            output.push_str(&self.indent());
            output.push_str("                if let Some(name) = file_name.to_str() {\n");
            output.push_str(&self.indent());
            output.push_str("                    if name != \".\" && name != \"..\" {\n");
            output.push_str(&self.indent());
            output.push_str("                        println!(\"{}\", name);\n");
            output.push_str(&self.indent());
            output.push_str("                    }\n");
            output.push_str(&self.indent());
            output.push_str("                }\n");
            output.push_str(&self.indent());
            output.push_str("            }\n");
            output.push_str(&self.indent());
            output.push_str("        }\n");
            output.push_str(&self.indent());
            output.push_str("    }\n");
            output.push_str(&self.indent());
            output.push_str("    Err(_) => return std::process::ExitCode::FAILURE,\n");
            output.push_str("}\n");
        } else if cmd.name == "grep" {
            // Special handling for grep
            if cmd.args.len() >= 2 {
                let pattern = cmd.args[0].to_string();
                let file = cmd.args[1].to_string();
                output.push_str(&format!("match fs::read_to_string(\"{}\") {{\n", file));
                output.push_str(&self.indent());
                output.push_str("    Ok(content) => {\n");
                output.push_str(&self.indent());
                output.push_str("        for line in content.lines() {\n");
                output.push_str(&self.indent());
                output.push_str(&format!("            if line.contains(\"{}\") {{\n", pattern));
                output.push_str(&self.indent());
                output.push_str("                println!(\"{}\", line);\n");
                output.push_str(&self.indent());
                output.push_str("            }\n");
                output.push_str(&self.indent());
                output.push_str("        }\n");
                output.push_str(&self.indent());
                output.push_str("    }\n");
                output.push_str(&self.indent());
                output.push_str("    Err(_) => return std::process::ExitCode::FAILURE,\n");
                output.push_str("}\n");
            }
        } else if cmd.name == "cat" {
            // Special handling for cat including heredocs
            let mut printed_any = false;
            for redir in &cmd.redirects {
                if matches!(redir.operator, RedirectOperator::Heredoc | RedirectOperator::HeredocTabs) {
                    if let Some(body) = &redir.heredoc_body {
                        // Normalize line endings to handle Windows vs Unix line endings
                        let normalized_body = body.replace("\r\n", "\n").replace("\r", "\n");
                        let esc = self.escape_rust_string(&normalized_body);
                        output.push_str(&format!("print!(\"{}\");\n", esc));
                        printed_any = true;
                    }
                }
            }
            if !printed_any {
                for arg in &cmd.args {
                    output.push_str(&format!("match fs::read_to_string(\"{}\") {{\n", arg.to_string()));
                    output.push_str(&self.indent());
                    output.push_str("    Ok(content) => print!(\"{}\", content),\n");
                    output.push_str(&self.indent());
                    output.push_str("    Err(_) => return std::process::ExitCode::FAILURE,\n");
                    output.push_str("}\n");
                }
            }
        } else if cmd.name == "mkdir" {
            // Special handling for mkdir
            for arg in &cmd.args {
                output.push_str(&format!("if let Err(_) = fs::create_dir_all(\"{}\") {{\n", arg.to_string()));
                output.push_str(&self.indent());
                output.push_str("    return std::process::ExitCode::FAILURE;\n");
                output.push_str("}\n");
            }
        } else if cmd.name == "rm" {
            // Special handling for rm
            for arg in &cmd.args {
                output.push_str(&format!("if let Err(_) = fs::remove_file(\"{}\") {{\n", arg.to_string()));
                output.push_str(&self.indent());
                output.push_str("    return std::process::ExitCode::FAILURE;\n");
                output.push_str("}\n");
            }
        } else if cmd.name == "mv" {
            // Special handling for mv
            if cmd.args.len() >= 2 {
                let src = cmd.args[0].to_string();
                let dst = cmd.args[1].to_string();
                output.push_str(&format!("if let Err(_) = fs::rename(\"{}\", \"{}\") {{\n", src, dst));
                output.push_str(&self.indent());
                output.push_str("    return std::process::ExitCode::FAILURE;\n");
                output.push_str("}\n");
            }
        } else if cmd.name == "cp" {
            // Special handling for cp
            if cmd.args.len() >= 2 {
                let src = cmd.args[0].to_string();
                let dst = cmd.args[1].to_string();
                output.push_str(&format!("if let Err(_) = fs::copy(\"{}\", \"{}\") {{\n", src, dst));
                output.push_str(&self.indent());
                output.push_str("    return std::process::ExitCode::FAILURE;\n");
                output.push_str("}\n");
            }
        } else if cmd.name == "read" {
            // Read a line from stdin into a variable
            if let Some(var) = cmd.args.get(0) {
                let var_name = var.to_string();
                output.push_str(&format!("let mut {} = String::new();\n", var_name));
                output.push_str(&format!("if let Err(_) = io::stdin().read_line(&mut {}) {{\n", var_name));
                output.push_str(&self.indent());
                output.push_str("    return std::process::ExitCode::FAILURE;\n");
                output.push_str("}\n");
                output.push_str(&format!("let {v} = {v}.trim().to_string();\n", v = var_name));
            }
        } else {
            // Generic command
            if cmd.args.is_empty() {
                output.push_str(&format!("if let Err(_) = Command::new(\"{}\")\n", cmd.name));
                output.push_str(&self.indent());
                output.push_str("    .status() {\n");
                output.push_str(&self.indent());
                output.push_str("    return std::process::ExitCode::FAILURE;\n");
                output.push_str("}\n");
            } else {
                let args_str = cmd.args.iter().map(|arg| format!("\"{}\"", arg.to_string())).collect::<Vec<_>>().join(", ");
                output.push_str(&format!("if let Err(_) = Command::new(\"{}\")\n", cmd.name));
                output.push_str(&self.indent());
                output.push_str(&format!("    .args(&[{}])\n", args_str));
                output.push_str(&self.indent());
                output.push_str("    .status() {\n");
                output.push_str(&self.indent());
                output.push_str("    return std::process::ExitCode::FAILURE;\n");
                output.push_str("}\n");
            }
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
        
        // Extract variable name from condition if it's a test command
        let mut _var_name = None;
        if let Command::Simple(cmd) = &*while_loop.condition {
            if cmd.name == "[" || cmd.name == "test" {
                if let Some(test_op) = cmd.args.get(0) {
                    if test_op == "-lt" || test_op == "-gt" || test_op == "-eq" || test_op == "-ne" {
                        if let Some(var) = cmd.args.get(1) {
                            if let Some(stripped) = var.strip_prefix_char('$') {
                                _var_name = Some(stripped);
                            }
                        }
                    }
                }
            }
        }
        
        output.push_str("while ");
        output.push_str(&self.generate_condition(&while_loop.condition));
        output.push_str(" {\n");
        
        self.indent_level += 1;
        // Generate the body
        let body_chunk = self.generate_block(&while_loop.body);
        output.push_str(&body_chunk);
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
            let body_chunk = self.generate_block(&for_loop.body);
            output.push_str(&self.indent_block(&body_chunk));
            self.indent_level -= 1;
            output.push_str("}\n");
        } else {
            // For loop with items
            if for_loop.items.len() == 1 && (for_loop.items[0].to_string() == "$@" || for_loop.items[0].to_string() == "${@}") {
                // Special case: iterate over command line arguments
                output.push_str("for arg in std::env::args().skip(1) {\n");
                self.indent_level += 1;
                output.push_str(&self.indent());
                output.push_str(&format!("let {} = arg;\n", for_loop.variable));
                output.push_str(&self.indent());
                output.push_str(&self.generate_block(&for_loop.body));
                self.indent_level -= 1;
                output.push_str("}\n");
            } else {
                // Regular for loop with items - handle brace expansion
                let mut expanded_items = Vec::new();
                for item in &for_loop.items {
                    if let Some(expanded) = self.expand_brace_expression(&item.to_string()) {
                        expanded_items.extend(expanded);
                    } else {
                        expanded_items.push(item.to_string());
                    }
                }
                
                let items_str = expanded_items.iter().map(|item| format!("\"{}\"", item)).collect::<Vec<_>>().join(", ");
                output.push_str(&format!("for {} in &[{}] {{\n", for_loop.variable, items_str));
                self.indent_level += 1;
                output.push_str(&self.indent());
                // Generate the body
                let body_chunk = self.generate_block(&for_loop.body);
                output.push_str(&body_chunk);
                self.indent_level -= 1;
                output.push_str("}\n");
            }
        }
        
        output
    }

    fn generate_function(&mut self, func: &Function) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("fn {}() {{\n", func.name));
        self.indent_level += 1;
        let body_chunk = self.generate_block(&func.body);
        output.push_str(&self.indent_block(&body_chunk));
        self.indent_level -= 1;
        output.push_str("}\n");
        
        output
    }

    fn generate_subshell(&mut self, command: &Command) -> String {
        let mut output = String::new();
        // Emulate subshell by snapshotting and restoring environment
        output.push_str("{\n");
        output.push_str("    let __backup_env: std::collections::HashMap<String,String> = std::env::vars().collect();\n");
        self.indent_level += 1;
        let inner_chunk = self.generate_command(command);
        output.push_str(&self.indent_block(&inner_chunk));
        self.indent_level -= 1;
        output.push_str("    {\n");
        output.push_str("        use std::collections::{HashMap, HashSet};\n");
        output.push_str("        let __current_keys: HashSet<String> = std::env::vars().map(|(k, _)| k).collect();\n");
        output.push_str("        let __backup_keys: HashSet<String> = __backup_env.keys().cloned().collect();\n");
        output.push_str("        for k in __current_keys.difference(&__backup_keys) { std::env::remove_var(k); }\n");
        output.push_str("        for (k,v) in __backup_env.into_iter() { std::env::set_var(k, v); }\n");
        output.push_str("    }\n");
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
                            "-lt" => {
                                if let Some(var) = cmd.args.get(1) {
                                    if let Some(num) = cmd.args.get(2) {
                                        if let Some(stripped) = var.strip_prefix("$") {
                                            return format!("env::var(\"{}\").unwrap_or_default().parse::<i32>().unwrap_or(0) < {}", stripped, num);
                                        }
                                    }
                                }
                            }
                            "-gt" => {
                                if let Some(var) = cmd.args.get(1) {
                                    if let Some(num) = cmd.args.get(2) {
                                        if let Some(stripped) = var.strip_prefix("$") {
                                            return format!("env::var(\"{}\").unwrap_or_default().parse::<i32>().unwrap_or(0) > {}", stripped, num);
                                        }
                                    }
                                }
                            }
                            "-eq" => {
                                if let Some(var) = cmd.args.get(1) {
                                    if let Some(num) = cmd.args.get(2) {
                                        if let Some(stripped) = var.strip_prefix("$") {
                                            return format!("env::var(\"{}\").unwrap_or_default().parse::<i32>().unwrap_or(0) == {}", stripped, num);
                                        }
                                    }
                                }
                            }
                            "-ne" => {
                                if let Some(var) = cmd.args.get(1) {
                                    if let Some(num) = cmd.args.get(2) {
                                        if let Some(stripped) = var.strip_prefix("$") {
                                            return format!("env::var(\"{}\").unwrap_or_default().parse::<i32>().unwrap_or(0) != {}", stripped, num);
                                        }
                                    }
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
        let mut last_was_blank = false;
        for line in s.lines() {
            let is_blank = line.trim().is_empty();
            // Skip consecutive blank lines
            if is_blank && last_was_blank {
                continue;
            }
            out.push_str(&prefix);
            out.push_str(line);
            out.push('\n');
            last_was_blank = is_blank;
        }
        out
    }
    
    fn escape_rust_string(&self, s: &str) -> String {
        // First, unescape any \" sequences to " to avoid double-escaping
        let unescaped = s.replace("\\\"", "\"");
        // Then escape quotes and other characters for Rust
        let escaped = unescaped
            .replace("\\", "\\\\")  // Must escape backslashes first
            .replace("\"", "\\\"")  // Then escape quotes
            .replace("\n", "\\n")
            .replace("\r", "\\r")
            .replace("\t", "\\t");
        // For single quotes, no escaping needed in Rust strings
        escaped
    }
    
    fn expand_brace_expression(&self, s: &str) -> Option<Vec<String>> {
        // Handle simple numeric ranges like {1..5}
        if let Some(range) = s.strip_prefix('{').and_then(|s| s.strip_suffix('}')) {
            if let Some((start, end)) = range.split_once("..") {
                if let (Ok(start_num), Ok(end_num)) = (start.parse::<i32>(), end.parse::<i32>()) {
                    let mut result = Vec::new();
                    for i in start_num..=end_num {
                        result.push(i.to_string());
                    }
                    return Some(result);
                }
            }
            // Handle alphabetic ranges like {a..c}
            if let Some((start, end)) = range.split_once("..") {
                if start.len() == 1 && end.len() == 1 {
                    if let (Some(start_char), Some(end_char)) = (start.chars().next(), end.chars().next()) {
                        if start_char.is_ascii_lowercase() && end_char.is_ascii_lowercase() {
                            let mut result = Vec::new();
                            for c in start_char..=end_char {
                                result.push(c.to_string());
                            }
                            return Some(result);
                        }
                    }
                }
            }
            // Handle step ranges like {00..04..2}
            if let Some((range_part, step_part)) = range.split_once("..") {
                if let Some((start, end)) = range_part.split_once("..") {
                    if let (Ok(start_num), Ok(end_num), Ok(step)) = (
                        start.parse::<i32>(), 
                        end.parse::<i32>(), 
                        step_part.parse::<i32>()
                    ) {
                        let mut result = Vec::new();
                        let mut i = start_num;
                        while i <= end_num {
                            result.push(format!("{:02}", i)); // Zero-pad to 2 digits
                            i += step;
                        }
                        return Some(result);
                    }
                }
            }
        }
        None
    }
}

impl RustGenerator {
    fn extract_var_name(arg: &str) -> Option<String> {
        if let Some(stripped) = arg.strip_prefix("$") {
            if stripped.starts_with('{') && stripped.ends_with('}') && stripped.len() >= 3 {
                return Some(stripped[1..stripped.len()-1].to_string());
            }
            if !stripped.is_empty() {
                return Some(stripped.to_string());
            }
        }
        None
    }
}




