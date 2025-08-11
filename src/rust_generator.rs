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
        let mut needs_collections = false;
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
                    
                    // Check for associative array assignments
                    for (var, _) in &cmd.env_vars {
                        if var.contains('[') {
                            needs_collections = true;
                            break;
                        }
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
        if needs_collections {
            output.push_str("use std::collections;\n");
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
            Command::ShoptCommand(cmd) => self.generate_shopt_command(cmd),
            Command::TestExpression(test_expr) => self.generate_test_expression(test_expr),
            Command::Pipeline(pipeline) => self.generate_pipeline(pipeline),
            Command::If(if_stmt) => self.generate_if_statement(if_stmt),
            Command::While(while_loop) => self.generate_while_loop(while_loop),
            Command::For(for_loop) => self.generate_for_loop(for_loop),
            Command::Function(func) => self.generate_function(func),
            Command::Subshell(cmd) => self.generate_subshell(cmd),
            Command::Background(cmd) => self.generate_background(cmd),
            Command::Block(block) => self.generate_block(block),
            Command::BuiltinCommand(cmd) => self.generate_builtin_command(cmd),
            Command::BlankLine => "\n".to_string(),
        }
    }

    fn generate_simple_command(&self, cmd: &SimpleCommand) -> String {
        let mut output = String::new();
        
        // Handle environment variables and array assignments
        let mut has_associative_array = false;
        let mut associative_array_name = String::new();
        
        for (var, value) in &cmd.env_vars {
            if var.contains('[') {
                // This is an associative array assignment like map[foo]=bar
                if !has_associative_array {
                    // Extract the array name (everything before the first [)
                    if let Some(bracket_pos) = var.find('[') {
                        associative_array_name = var[..bracket_pos].to_string();
                        output.push_str(&format!("let mut {}: std::collections::HashMap<String, String> = std::collections::HashMap::new();\n", associative_array_name));
                        has_associative_array = true;
                    }
                }
                
                // Extract the key and value
                if let Some(bracket_pos) = var.find('[') {
                    if let Some(end_bracket_pos) = var.rfind(']') {
                        let key = &var[bracket_pos + 1..end_bracket_pos];
                        let value_str = self.word_to_string(value);
                        // Remove quotes if present
                        let clean_value = if value_str.starts_with('"') && value_str.ends_with('"') {
                            &value_str[1..value_str.len()-1]
                        } else {
                            &value_str
                        };
                        output.push_str(&format!("{}.insert(\"{}\".to_string(), \"{}\".to_string());\n", associative_array_name, key, clean_value));
                    }
                }
            } else {
                match value {
                    Word::Array(name, elements) => {
                        // Handle array declaration
                        let elements_str = elements.iter()
                            .map(|e| format!("\"{}\"", self.escape_rust_string(e)))
                            .collect::<Vec<_>>()
                            .join(", ");
                        output.push_str(&format!("let {}: Vec<&str> = vec![{}];\n", name, elements_str));
                    }
                    _ => {
                        // Handle regular environment variable
                        output.push_str(&format!("env::set_var(\"{}\", \"{}\");\n", var, value));
                    }
                }
            }
        }
        
        // Handle variable assignments (e.g., i=5)
        if self.word_to_string(&cmd.name).contains('=') {
            let name_str = self.word_to_string(&cmd.name);
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
                if cmd.args.len() == 1 && self.word_to_string(&cmd.args[0]) == "$#" {
                    output.push_str("let argc = std::env::args().count().saturating_sub(1);\n");
                    output.push_str("println!(\"{}\", argc);\n");
                } else if cmd.args.len() == 1 && (self.word_to_string(&cmd.args[0]) == "$@" || self.word_to_string(&cmd.args[0]) == "${@}") {
                    output.push_str("let joined = std::env::args().skip(1).collect::<Vec<_>>().join(\" \" );\n");
                    output.push_str("println!(\"{}\", joined);\n");
                } else {
                    // Check if we have any complex words that need special handling
                    let has_complex_words = cmd.args.iter().any(|arg| {
                        matches!(arg, Word::Variable(_) | Word::MapAccess(_, _) | Word::MapLength(_) | 
                                       Word::MapKeys(_) | Word::ParameterExpansion(_) | Word::Arithmetic(_) |
                                       Word::StringInterpolation(_))
                    });
                    
                    if !has_complex_words && cmd.args.len() == 1 {
                        // Simple case: single literal string
                        match &cmd.args[0] {
                            Word::Variable(var) => {
                                output.push_str(&format!("println!(\"{{}}\", {});\n", var));
                            }
                            _ => {
                                let arg_str = &cmd.args[0];
                                let clean_str = if arg_str.starts_with('"') && arg_str.ends_with('"') {
                                    &arg_str[1..arg_str.len()-1]
                                } else {
                                    &arg_str
                                };
                                let escaped = self.escape_rust_string(clean_str);
                                output.push_str(&format!("println!(\"{}\");\n", escaped));
                            }
                        }
                    } else if !has_complex_words {
                        // Multiple literal strings - join them with space
                        let escaped = cmd.args.iter()
                            .map(|arg| {
                                match arg {
                                    Word::Variable(var) => format!("{{{}}}", var),
                                    _ => {
                                        let arg_str = arg;
                                        let clean_str = if arg_str.to_string().starts_with('"') && arg_str.to_string().ends_with('"') {
                                            &arg_str.to_string()[1..arg_str.to_string().len()-1]
                                        } else {
                                            &arg_str.to_string()
                                        };
                                        self.escape_rust_string(clean_str)
                                    }
                                }
                            })
                            .collect::<Vec<_>>()
                            .join(" ");
                        output.push_str(&format!("println!(\"{}\");\n", escaped));
                    } else {
                        // Check if we can inline the complex case without __echo_parts
                        let can_inline = cmd.args.len() <= 3 && cmd.args.iter().all(|arg| {
                            matches!(arg, 
                                Word::Variable(_) | 
                                Word::Literal(_) |
                                Word::Arithmetic(_) |
                                Word::ParameterExpansion(_)
                            )
                        });
                        
                        if can_inline {
                            // Inline simple complex cases directly into println!
                            let mut format_parts = Vec::new();
                            let mut args = Vec::new();
                            
                            for arg in &cmd.args {
                                match arg {
                                    Word::Variable(var) => {
                                        format_parts.push("{}");
                                        args.push(format!("{}", var));
                                    }
                                    Word::Literal(lit) => {
                                        let clean_str = if lit.starts_with('"') && lit.ends_with('"') {
                                            &lit[1..lit.len()-1]
                                        } else {
                                            lit
                                        };
                                        let escaped = self.escape_rust_string(clean_str);
                                        format_parts.push(&escaped);
                                    }
                                    Word::Arithmetic(expr) => {
                                        format_parts.push("{}");
                                        args.push(format!("({})", expr.expression));
                                    }
                                    Word::ParameterExpansion(pe) => {
                                        format_parts.push("{}");
                                        let pe_str = match &pe.operator {
                                            ParameterExpansionOperator::UppercaseAll => format!("{}.to_uppercase()", pe.variable),
                                            ParameterExpansionOperator::LowercaseAll => format!("{}.to_lowercase()", pe.variable),
                                            ParameterExpansionOperator::UppercaseFirst => format!("{}.chars().next().unwrap_or(' ').to_uppercase().collect::<String>() + &{}[1..]", pe.variable, pe.variable),
                                            _ => format!("${{{}}}", pe.variable),
                                        };
                                        args.push(pe_str);
                                    }
                                    _ => {
                                        // Fallback to __echo_parts for unexpected cases
                                        can_inline = false;
                                        break;
                                    }
                                }
                            }
                            
                            if can_inline && args.is_empty() {
                                // All literals - no format args needed
                                output.push_str(&format!("println!(\"{}\");\n", format_parts.join(" ")));
                            } else if can_inline {
                                // Mixed literals and variables - use format args
                                let format_str = format_parts.join(" ");
                                output.push_str(&format!("println!(\"{}\", {});\n", format_str, args.join(", ")));
                            } else {
                                // Fallback to __echo_parts for complex cases
                                self.generate_echo_with_parts(output, &cmd.args);
                            }
                        } else {
                            // Complex case that needs __echo_parts
                            self.generate_echo_with_parts(output, &cmd.args);
                        }
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
                            let dir = if cmd.args.is_empty() { "." } else { &cmd.args[0] };
            output.push_str(&format!("if let Err(_) = env::set_current_dir(\"{}\") {{\n", dir));
            output.push_str(&self.indent());
            output.push_str("    return std::process::ExitCode::FAILURE;\n");
            output.push_str("}\n");
        } else if cmd.name == "ls" {
            // Special handling for ls
                            let args = if cmd.args.is_empty() { "." } else { &cmd.args[0] };
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
                let pattern = &cmd.args[0];
                let file = &cmd.args[1];
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
                let src = &cmd.args[0];
                let dst = &cmd.args[1];
                output.push_str(&format!("if let Err(_) = fs::rename(\"{}\", \"{}\") {{\n", src, dst));
                output.push_str(&self.indent());
                output.push_str("    return std::process::ExitCode::FAILURE;\n");
                output.push_str("}\n");
            }
        } else if cmd.name == "cp" {
            // Special handling for cp
            if cmd.args.len() >= 2 {
                let src = &cmd.args[0];
                let dst = &cmd.args[1];
                output.push_str(&format!("if let Err(_) = fs::copy(\"{}\", \"{}\") {{\n", src, dst));
                output.push_str(&self.indent());
                output.push_str("    return std::process::ExitCode::FAILURE;\n");
                output.push_str("}\n");
            }
        } else if cmd.name == "read" {
            // Read a line from stdin into a variable
            if let Some(var) = cmd.args.get(0) {
                let var_name = &var;
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
                let args_str = cmd.args.iter().map(|arg| {
                    match arg {
                        Word::Variable(var) => format!("{{{}}}", var),
                        Word::StringInterpolation(interp) => {
                            // Handle string interpolation in command arguments
                            if interp.parts.len() == 1 {
                                match &interp.parts[0] {
                                    StringPart::Variable(var) => {
                                        // For variables, just use the variable name
                                        format!("{{{}}}", var)
                                    }
                                    StringPart::Literal(s) => {
                                        format!("\"{}\"", self.escape_rust_string(s))
                                    }
                                    _ => {
                                        let arg_str = self.word_to_string(arg);
                                        format!("\"{}\"", self.escape_rust_string(&arg_str))
                                    }
                                }
                            } else {
                                let arg_str = self.word_to_string(arg);
                                format!("\"{}\"", self.escape_rust_string(&arg_str))
                            }
                        }
                        _ => {
                            let arg_str = self.word_to_string(arg);
                            format!("\"{}\"", self.escape_rust_string(&arg_str))
                        }
                    }
                }).collect::<Vec<_>>().join(", ");
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

    fn generate_shopt_command(&mut self, cmd: &ShoptCommand) -> String {
        let mut output = String::new();
        
        // Handle shopt command for shell options
        if cmd.enable {
            match cmd.option.as_str() {
                "extglob" => {
                    output.push_str("// extglob option enabled\n");
                }
                "nocasematch" => {
                    output.push_str("// nocasematch option enabled\n");
                }
                _ => {
                    output.push_str(&format!("// shopt -s {} not implemented\n", cmd.option));
                }
            }
        } else {
            match cmd.option.as_str() {
                "extglob" => {
                    output.push_str("// extglob option disabled\n");
                }
                "nocasematch" => {
                    output.push_str("// nocasematch option disabled\n");
                }
                _ => {
                    output.push_str(&format!("// shopt -u {} not implemented\n", cmd.option));
                }
            }
        }
        
        output
    }
    
    fn generate_builtin_command(&mut self, cmd: &BuiltinCommand) -> String {
        let mut output = String::new();
        
        // Handle environment variables if any
        for (var, value) in &cmd.env_vars {
            output.push_str(&format!("env::set_var(\"{}\", \"{}\");\n", var, value));
        }
        
        // Generate the builtin command
        match cmd.name.as_str() {
            "set" => {
                // Convert shell set options to Rust equivalents
                for arg in &cmd.args {
                    if let Word::Literal(opt) = arg {
                        match opt.as_str() {
                            "-e" => output.push_str("// set -e: exit on error\n"),
                            "-u" => output.push_str("// set -u: error on undefined variables\n"),
                            "-o" => {
                                // Handle pipefail and other options
                                if let Some(next_arg) = cmd.args.iter().skip(1).find(|a| {
                                    if let Word::Literal(s) = a { s == "pipefail" } else { false }
                                }) {
                                    output.push_str("// set -o pipefail\n");
                                }
                            }
                            _ => output.push_str(&format!("// set {}\n", opt)),
                        }
                    }
                }
            }
            "export" => {
                // Convert export to Rust environment variable assignment
                for arg in &cmd.args {
                    if let Word::Literal(var) = arg {
                        if var.contains('=') {
                            let parts: Vec<&str> = var.splitn(2, '=').collect();
                            if parts.len() == 2 {
                                let var_name = parts[0];
                                let var_value = parts[1];
                                output.push_str(&format!("env::set_var(\"{}\", \"{}\");\n", var_name, var_value));
                            }
                        } else {
                            output.push_str(&format!("// export {}\n", var));
                        }
                    }
                }
            }
            "local" => {
                // Convert local to Rust let declaration
                for arg in &cmd.args {
                    if let Word::Literal(var) = arg {
                        if var.contains('=') {
                            let parts: Vec<&str> = var.splitn(2, '=').collect();
                            if parts.len() == 2 {
                                let var_name = parts[0];
                                let var_value = parts[1];
                                output.push_str(&format!("let {} = \"{}\";\n", var_name, var_value));
                            }
                        } else {
                            output.push_str(&format!("let {};\n", var));
                        }
                    }
                }
            }
            "unset" => {
                // Convert unset to Rust environment variable removal
                for arg in &cmd.args {
                    if let Word::Literal(var) = arg {
                        output.push_str(&format!("env::remove_var(\"{}\");\n", var));
                    }
                }
            }
            "declare" => {
                // Handle declare command for arrays
                for arg in &cmd.args {
                    if let Word::Literal(opt) = arg {
                        if opt == "-A" {
                            // Declare associative array - for now just comment it
                            output.push_str("// declare -A: associative array declaration\n");
                        } else {
                            output.push_str(&format!("// declare {}\n", opt));
                        }
                    }
                }
            }
            _ => {
                // For other builtins, generate a comment
                output.push_str(&format!("// {} {}\n", cmd.name, 
                    cmd.args.iter().map(|arg| arg.to_string()).collect::<Vec<_>>().join(" ")));
            }
        }
        
        output
    }

    fn generate_test_expression(&mut self, test_expr: &TestExpression) -> String {
        let mut output = String::new();
        
        // Parse the test expression to extract components
        let expr = &test_expr.expression;
        let modifiers = &test_expr.modifiers;
        
        // Add comments about enabled options
        if modifiers.extglob {
            output.push_str("// extglob enabled\n");
        }
        if modifiers.nocasematch {
            output.push_str("// nocasematch enabled\n");
        }
        
        // Parse the expression to determine the type of test
        if expr.contains(" =~ ") {
            // Regex matching: [[ $var =~ pattern ]]
            let parts: Vec<&str> = expr.split(" =~ ").collect();
            if parts.len() == 2 {
                let var = parts[0].trim();
                let pattern = parts[1].trim();
                
                // Convert to Rust regex matching
                output.push_str(&format!("// Regex test: {} =~ {}\n", var, pattern));
                output.push_str("true // TODO: implement regex matching\n");
            } else {
                output.push_str(&format!("// Invalid regex test: {}\n", expr));
                output.push_str("false");
            }
        } else if expr.contains(" == ") {
            // Pattern matching: [[ $var == pattern ]]
            let parts: Vec<&str> = expr.split(" == ").collect();
            if parts.len() == 2 {
                let var = parts[0].trim();
                let pattern = parts[1].trim();
                
                if modifiers.nocasematch {
                    // Case-insensitive matching
                    output.push_str(&format!("// Case-insensitive pattern test: {} == {}\n", var, pattern));
                    output.push_str("true // TODO: implement case-insensitive pattern matching\n");
                } else {
                    // Case-sensitive matching
                    output.push_str(&format!("// Pattern test: {} == {}\n", var, pattern));
                    output.push_str("true // TODO: implement pattern matching\n");
                }
            } else {
                output.push_str(&format!("// Invalid pattern test: {}\n", expr));
                output.push_str("false");
            }
        } else {
            // Generic test expression
            output.push_str(&format!("// Test expression: {}\n", expr));
            output.push_str("true");
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
            if for_loop.items.len() == 1 && (self.word_to_string(&for_loop.items[0]) == "$@" || self.word_to_string(&for_loop.items[0]) == "${@}") {
                // Special case: iterate over command line arguments
                output.push_str("for arg in std::env::args().skip(1) {\n");
                self.indent_level += 1;
                output.push_str(&self.indent());
                output.push_str(&format!("let {} = arg;\n", for_loop.variable));
                output.push_str(&self.indent());
                output.push_str(&self.generate_block(&for_loop.body));
                self.indent_level -= 1;
                output.push_str("}\n");
            } else if for_loop.items.len() == 1 {
                // Check for special array iteration case: "${arr[@]}"
                match &for_loop.items[0] {
                    Word::StringInterpolation(interp) => {
                        if interp.parts.len() == 1 {
                            match &interp.parts[0] {
                                StringPart::MapAccess(map_name, key) if key == "@" => {
                                    // Special case: iterate over all array elements
                                    output.push_str(&format!("for {} in &{} {{\n", for_loop.variable, map_name));
                                    self.indent_level += 1;
                                    output.push_str(&self.indent());
                                    output.push_str(&format!("let {} = {};\n", for_loop.variable, for_loop.variable));
                                    output.push_str(&self.indent());
                                    output.push_str(&self.generate_block(&for_loop.body));
                                    self.indent_level -= 1;
                                    output.push_str("}\n");
                                    return output;
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
                
                // Regular for loop with items - handle brace expansion
                let mut expanded_items = Vec::new();
                for item in &for_loop.items {
                    if let Some(expanded) = self.expand_brace_expression(&self.word_to_string(item)) {
                        expanded_items.extend(expanded);
                    } else {
                        // Handle StringInterpolation specially for array iteration
                        match item {
                            Word::StringInterpolation(interp) => {
                                if interp.parts.len() == 1 {
                                    match &interp.parts[0] {
                                        StringPart::MapAccess(map_name, key) if key == "@" => {
                                            // This should have been caught above, but handle it here too
                                            expanded_items.push(map_name.clone());
                                        }
                                        _ => {
                                            expanded_items.push(self.word_to_string(item));
                                        }
                                    }
                                } else {
                                    expanded_items.push(self.word_to_string(item));
                                }
                            }
                            _ => {
                                expanded_items.push(self.word_to_string(item));
                            }
                        }
                    }
                }
                
                // Check if we have a single array reference
                if expanded_items.len() == 1 && !expanded_items[0].contains('"') && !expanded_items[0].contains(' ') {
                    // Single array reference - iterate over the array
                    output.push_str(&format!("for {} in &{} {{\n", for_loop.variable, expanded_items[0]));
                } else {
                    // Multiple items or complex items - use array literal
                    let items_str = expanded_items.iter().map(|item| format!("\"{}\"", item)).collect::<Vec<_>>().join(", ");
                    output.push_str(&format!("for {} in &[{}] {{\n", for_loop.variable, items_str));
                }
                self.indent_level += 1;
                output.push_str(&self.indent());
                // Generate the body
                let body_chunk = self.generate_block(&for_loop.body);
                output.push_str(&body_chunk);
                self.indent_level -= 1;
                output.push_str("}\n");
            } else {
                // Regular for loop with items - handle brace expansion
                let mut expanded_items = Vec::new();
                for item in &for_loop.items {
                    if let Some(expanded) = self.expand_brace_expression(&self.word_to_string(item)) {
                        expanded_items.extend(expanded);
                    } else {
                        // Handle StringInterpolation specially for array iteration
                        match item {
                            Word::StringInterpolation(interp) => {
                                if interp.parts.len() == 1 {
                                    match &interp.parts[0] {
                                        StringPart::MapAccess(map_name, key) if key == "@" => {
                                            // This should have been caught above, but handle it here too
                                            expanded_items.push(map_name.clone());
                                        }
                                        _ => {
                                            expanded_items.push(self.word_to_string(item));
                                        }
                                    }
                                } else {
                                    expanded_items.push(self.word_to_string(item));
                                }
                            }
                            _ => {
                                expanded_items.push(self.word_to_string(item));
                            }
                        }
                    }
                }
                
                // Check if we have a single array reference
                if expanded_items.len() == 1 && !expanded_items[0].contains('"') && !expanded_items[0].contains(' ') {
                    // Single array reference - iterate over the array
                    output.push_str(&format!("for {} in &{} {{\n", for_loop.variable, expanded_items[0]));
                } else {
                    // Multiple items or complex items - use array literal
                    let items_str = expanded_items.iter().map(|item| format!("\"{}\"", item)).collect::<Vec<_>>().join(", ");
                    output.push_str(&format!("for {} in &[{}] {{\n", for_loop.variable, items_str));
                }
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
                                    let file_str = &file;
                                    let clean_file = if file_str.starts_with('"') && file_str.ends_with('"') {
                                        &file_str[1..file_str.len()-1]
                                    } else {
                                        &file_str
                                    };
                                    return format!("fs::metadata(\"{}\").is_ok()", clean_file);
                                }
                            }
                            "-d" => {
                                if let Some(dir) = cmd.args.get(1) {
                                    let dir_str = &dir;
                                    let clean_dir = if dir_str.starts_with('"') && dir_str.ends_with('"') {
                                        &dir_str[1..dir_str.len()-1]
                                    } else {
                                        &dir_str
                                    };
                                    return format!("fs::metadata(\"{}\").map(|m| m.is_dir()).unwrap_or(false)", clean_dir);
                                }
                            }
                            "-e" => {
                                if let Some(path) = cmd.args.get(1) {
                                    let path_str = &path;
                                    let clean_path = if path_str.starts_with('"') && path_str.ends_with('"') {
                                        &path_str[1..path_str.len()-1]
                                    } else {
                                        &path_str
                                    };
                                    return format!("fs::metadata(\"{}\").is_ok()", clean_path);
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
    
    fn word_to_string(&self, word: &Word) -> String {
        match word {
            Word::Literal(s) => s.clone(),
            Word::Variable(var) => {
                // Special case for $@ - convert to Rust equivalent
                if var == "@" {
                    "std::env::args().skip(1).collect::<Vec<_>>()".to_string()
                } else {
                    format!("${}", var)
                }
            },
            Word::Array(name, elements) => {
                // Convert array declaration to Rust Vec
                let elements_str = elements.iter()
                    .map(|e| format!("\"{}\"", self.escape_rust_string(e)))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("let {}: Vec<&str> = vec![{}];", name, elements_str)
            },
            Word::ParameterExpansion(pe) => {
                match &pe.operator {
                    ParameterExpansionOperator::UppercaseAll => format!("${{{}}}", pe.variable),
                    ParameterExpansionOperator::LowercaseAll => format!("${{{}}}", pe.variable),
                    ParameterExpansionOperator::UppercaseFirst => format!("${{{}}}", pe.variable),
                    ParameterExpansionOperator::RemoveLongestPrefix(pattern) => format!("${{{}}}##{}", pe.variable, pattern),
                    ParameterExpansionOperator::RemoveShortestPrefix(pattern) => format!("${{{}}}#{}", pe.variable, pattern),
                    ParameterExpansionOperator::RemoveLongestSuffix(pattern) => format!("${{{}}}%%{}", pe.variable, pattern),
                    ParameterExpansionOperator::RemoveShortestSuffix(pattern) => format!("${{{}}}%{}", pe.variable, pattern),
                    ParameterExpansionOperator::SubstituteAll(pattern, replacement) => format!("${{{}}}//{}/{}", pe.variable, pattern, replacement),
                    ParameterExpansionOperator::DefaultValue(default) => format!("${{{}}}:-{}", pe.variable, default),
                    ParameterExpansionOperator::AssignDefault(default) => format!("${{{}}}:={}", pe.variable, default),
                    ParameterExpansionOperator::ErrorIfUnset(error) => format!("${{{}}}:?{}", pe.variable, error),
                    ParameterExpansionOperator::Basename => format!("${{{}}}##*/", pe.variable),
                    ParameterExpansionOperator::Dirname => format!("${{{}}}%/*", pe.variable),
                }
            },
            Word::MapAccess(map_name, key) => {
                // Convert array access to Rust Vec access
                if key == "@" {
                    // Special case: array iteration
                    format!("{}", map_name)
                } else {
                    // Regular array access - convert to Rust Vec access
                    format!("{}.get({}).unwrap_or(&String::new())", map_name, key)
                }
            },
            Word::MapKeys(map_name) => {
                // Convert array keys to Rust Vec iteration
                format!("{}", map_name)
            },
            Word::MapLength(map_name) => {
                // Convert array length to Rust Vec length
                format!("{}.len()", map_name)
            },
            Word::Arithmetic(expr) => expr.expression.clone(),
            Word::BraceExpansion(expansion) => {
                let mut result = String::new();
                if let Some(ref prefix) = expansion.prefix {
                    result.push_str(prefix);
                }
                for (i, item) in expansion.items.iter().enumerate() {
                    if i > 0 {
                        result.push(',');
                    }
                    match item {
                        BraceItem::Literal(s) => result.push_str(s),
                        BraceItem::Range(range) => {
                            result.push_str(&range.start);
                            result.push_str("..");
                            result.push_str(&range.end);
                            if let Some(ref step) = range.step {
                                result.push_str("..");
                                result.push_str(step);
                            }
                        }
                        BraceItem::Sequence(seq) => {
                            result.push_str(&seq.join(","));
                        }
                    }
                }
                if let Some(ref suffix) = expansion.suffix {
                    result.push_str(suffix);
                }
                format!("{{{}}}", result)
            }
            Word::CommandSubstitution(_) => "$(...)".to_string(),
            Word::StringInterpolation(interp) => {
                let mut result = String::new();
                for part in &interp.parts {
                    match part {
                        StringPart::Literal(s) => result.push_str(s),
                        StringPart::Variable(var) => {
                            // Special case for $@ - convert to Rust equivalent
                            if var == "@" {
                                result.push_str("std::env::args().skip(1).collect::<Vec<_>>()");
                            } else {
                                result.push_str(&format!("${}", var));
                            }
                        },
                        StringPart::ParameterExpansion(pe) => {
                            match &pe.operator {
                                ParameterExpansionOperator::UppercaseAll => result.push_str(&format!("${{{}}}", pe.variable)),
                                ParameterExpansionOperator::LowercaseAll => result.push_str(&format!("${{{}}}", pe.variable)),
                                ParameterExpansionOperator::UppercaseFirst => result.push_str(&format!("${{{}}}", pe.variable)),
                                ParameterExpansionOperator::RemoveLongestPrefix(pattern) => result.push_str(&format!("${{{}}}##{}", pe.variable, pattern)),
                                ParameterExpansionOperator::RemoveShortestPrefix(pattern) => result.push_str(&format!("${{{}}}#{}", pe.variable, pattern)),
                                ParameterExpansionOperator::RemoveLongestSuffix(pattern) => result.push_str(&format!("${{{}}}%%{}", pe.variable, pattern)),
                                ParameterExpansionOperator::RemoveShortestSuffix(pattern) => result.push_str(&format!("${{{}}}%{}", pe.variable, pattern)),
                                ParameterExpansionOperator::SubstituteAll(pattern, replacement) => result.push_str(&format!("${{{}}}//{}/{}", pe.variable, pattern, replacement)),
                                ParameterExpansionOperator::DefaultValue(default) => result.push_str(&format!("${{{}}}:-{}", pe.variable, default)),
                                ParameterExpansionOperator::AssignDefault(default) => result.push_str(&format!("${{{}}}:={}", pe.variable, default)),
                                ParameterExpansionOperator::ErrorIfUnset(error) => result.push_str(&format!("${{{}}}:?{}", pe.variable, error)),
                                ParameterExpansionOperator::Basename => result.push_str(&format!("${{{}}}##*/", pe.variable)),
                                ParameterExpansionOperator::Dirname => result.push_str(&format!("${{{}}}%/*", pe.variable)),
                            }
                        },
                        StringPart::MapAccess(map_name, key) => {
                            // Convert Bash array access to Rust equivalent
                            if key == "@" {
                                // Special case: array iteration
                                result.push_str(&format!("{}", map_name));
                            } else {
                                // Regular array access - convert to Rust Vec access
                                result.push_str(&format!("{}.get({}).unwrap_or(&String::new())", map_name, key));
                            }
                        },
                        StringPart::MapKeys(map_name) => {
                            // Convert Bash array keys to Rust equivalent
                            result.push_str(&format!("{}", map_name));
                        },
                        StringPart::MapLength(map_name) => {
                            // Convert Bash array length to Rust equivalent
                            result.push_str(&format!("{}.len()", map_name));
                        },
                        StringPart::Arithmetic(expr) => result.push_str(&expr.expression),
                        StringPart::CommandSubstitution(_) => result.push_str("$(...)"),
                    }
                }
                result
            }
        }
    }

    fn escape_rust_string(&self, s: &str) -> String {
        // Handle ANSI-C escape sequences and other special characters
        let mut result = String::new();
        let mut chars = s.chars().peekable();
        
        while let Some(ch) = chars.next() {
            match ch {
                '\\' => {
                    if let Some(next_ch) = chars.next() {
                        match next_ch {
                            'n' => result.push_str("\\n"),
                            'r' => result.push_str("\\r"),
                            't' => result.push_str("\\t"),
                            'a' => result.push_str("\\x07"), // bell
                            'b' => result.push_str("\\x08"), // backspace
                            'f' => result.push_str("\\x0c"), // formfeed
                            'v' => result.push_str("\\x0b"), // vertical tab
                            '\\' => result.push_str("\\\\"),
                            '"' => result.push_str("\\\""),
                            '\'' => result.push_str("\\'"),
                            'x' => {
                                // Hex escape: \xHH
                                let mut hex = String::new();
                                for _ in 0..2 {
                                    if let Some(hex_ch) = chars.next() {
                                        if hex_ch.is_ascii_hexdigit() {
                                            hex.push(hex_ch);
                                        } else {
                                            break;
                                        }
                                    } else {
                                        break;
                                    }
                                }
                                if hex.len() == 2 {
                                    result.push_str(&format!("\\x{}", hex));
                                } else {
                                    result.push_str("\\\\x");
                                    result.push_str(&hex);
                                }
                            }
                            'u' => {
                                // Unicode escape: \uHHHH
                                let mut hex = String::new();
                                for _ in 0..4 {
                                    if let Some(hex_ch) = chars.next() {
                                        if hex_ch.is_ascii_hexdigit() {
                                            hex.push(hex_ch);
                                        } else {
                                            break;
                                        }
                                    } else {
                                        break;
                                    }
                                }
                                if hex.len() == 4 {
                                    result.push_str(&format!("\\u{}", hex));
                                } else {
                                    result.push_str("\\\\u");
                                    result.push_str(&hex);
                                }
                            }
                            _ => {
                                // Unknown escape sequence, treat as literal
                                result.push_str("\\\\");
                                result.push(next_ch);
                            }
                        }
                    } else {
                        // Trailing backslash
                        result.push_str("\\\\");
                    }
                }
                '"' => result.push_str("\\\""),
                '\n' => result.push_str("\\n"),
                '\r' => result.push_str("\\r"),
                '\t' => result.push_str("\\t"),
                _ => result.push(ch),
            }
        }
        
        result
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

    fn generate_echo_with_parts(&mut self, output: &mut String, args: &[Word]) {
        output.push_str("let __echo_parts: Vec<String> = vec![\n");
        for arg in args {
            match arg {
                Word::Variable(var) => {
                    output.push_str(&format!("    {}.to_string(),\n", var));
                }
                Word::MapAccess(map_name, key) => {
                    if key == "@" {
                        output.push_str(&format!("    {}.join(\" \"),\n", map_name));
                    } else {
                        output.push_str(&format!("    {}.get({}).unwrap_or(&String::new()),\n", map_name, key));
                    }
                }
                Word::MapLength(map_name) => {
                    output.push_str(&format!("    {}.len(),\n", map_name));
                }
                Word::MapKeys(map_name) => {
                    output.push_str(&format!("    {}.join(\" \"),\n", map_name));
                }
                Word::ParameterExpansion(pe) => {
                    let pe_str = match &pe.operator {
                        ParameterExpansionOperator::UppercaseAll => format!("{}.to_uppercase()", pe.variable),
                        ParameterExpansionOperator::LowercaseAll => format!("{}.to_lowercase()", pe.variable),
                        ParameterExpansionOperator::UppercaseFirst => format!("{}.chars().next().unwrap_or(' ').to_uppercase().collect::<String>() + &{}[1..]", pe.variable, pe.variable),
                        _ => format!("${{{}}}", pe.variable),
                    };
                    output.push_str(&format!("    {},\n", pe_str));
                }
                Word::Arithmetic(expr) => {
                    output.push_str(&format!("    ({}),\n", expr.expression));
                }
                Word::StringInterpolation(interp) => {
                    let mut interp_parts = Vec::new();
                    for part in &interp.parts {
                        match part {
                            StringPart::Literal(s) => {
                                interp_parts.push(format!("\"{}\"", self.escape_rust_string(s)));
                            }
                            StringPart::Variable(var) => {
                                if var == "@" {
                                    interp_parts.push("std::env::args().skip(1).collect::<Vec<_>>()".to_string());
                                } else {
                                    interp_parts.push(format!("${}", var));
                                }
                            }
                            StringPart::MapAccess(map_name, key) => {
                                if key == "@" {
                                    interp_parts.push(format!("{}", map_name));
                                } else {
                                    interp_parts.push(format!("{}.get({}).unwrap_or(&String::new())", map_name, key));
                                }
                            }
                            StringPart::MapKeys(map_name) => {
                                interp_parts.push(format!("{}", map_name));
                            }
                            StringPart::MapLength(map_name) => {
                                interp_parts.push(format!("{}.len()", map_name));
                            }
                            StringPart::Arithmetic(expr) => {
                                interp_parts.push(format!("({})", expr.expression));
                            }
                            StringPart::ParameterExpansion(pe) => {
                                let pe_str = match &pe.operator {
                                    ParameterExpansionOperator::UppercaseAll => format!("{}.to_uppercase()", pe.variable),
                                    ParameterExpansionOperator::LowercaseAll => format!("{}.to_lowercase()", pe.variable),
                                    ParameterExpansionOperator::UppercaseFirst => format!("{}.chars().next().unwrap_or(' ').to_uppercase().collect::<String>() + &{}[1..]", pe.variable, pe.variable),
                                    _ => format!("${{{}}}", pe.variable),
                                };
                                interp_parts.push(pe_str);
                            }
                            StringPart::CommandSubstitution(_) => {
                                interp_parts.push("$(...)".to_string());
                            }
                            _ => {
                                // For any other unhandled cases, convert to string representation
                                interp_parts.push(format!("\"{:?}\"", part));
                            }
                        }
                    }
                    // Join the parts and convert to string
                    if interp_parts.len() == 1 {
                        output.push_str(&format!("    {}.to_string(),\n", interp_parts[0]));
                    } else {
                        output.push_str(&format!("    ({}).to_string(),\n", interp_parts.join(" + ")));
                    }
                }
                _ => {
                    let arg_str = arg;
                    let clean_str = if arg_str.starts_with('"') && arg_str.ends_with('"') {
                        &arg_str[1..arg_str.len()-1]
                    } else {
                        &arg_str
                    };
                    let escaped = self.escape_rust_string(clean_str);
                    output.push_str(&format!("    \"{}\",\n", escaped));
                }
            }
        }
        output.push_str("];\n");
        output.push_str("println!(\"{}\", __echo_parts.join(\" \"));\n");
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




