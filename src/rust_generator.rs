use crate::ast::*;
use crate::shared_utils::SharedUtils;

pub struct RustGenerator {
    indent_level: usize,
    pipeline_counter: usize,
}

impl RustGenerator {
    pub fn new() -> Self {
        Self { 
            indent_level: 0,
            pipeline_counter: 0,
        }
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
                    needs_io = true; // Pipelines use stdin/stdout for piping
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
        // Add regex support for pattern matching
        if needs_fs || needs_command {
            // Note: regex crate not available in basic setup, using string operations instead
            // output.push_str("use regex;\n");
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

    fn generate_simple_command(&mut self, cmd: &SimpleCommand) -> String {
        let mut output = String::new();
        
        // Handle environment variables and array assignments
        let mut has_associative_array = false;
        let mut associative_array_name = String::new();
        
        // Pre-process process substitution and here-string redirects to create temporary files
        let mut process_sub_files = Vec::new();
        let mut has_here_string = false;
        let mut temp_file_counter = 0;
        
        for redir in &cmd.redirects {
            match &redir.operator {
                RedirectOperator::ProcessSubstitutionInput(cmd) => {
                    // Process substitution input: <(command)
                    temp_file_counter += 1;
                    let temp_file = format!("/tmp/process_sub_{}_{}.tmp", std::process::id(), temp_file_counter);
                    let temp_var = format!("temp_file_ps_{}", temp_file_counter);
                    output.push_str(&format!("let {} = \"{}\";\n", temp_var, temp_file));
                    
                    // Generate the command for system call
                    let cmd_str = match &**cmd {
                        Command::Simple(simple_cmd) => {
                            let args = simple_cmd.args.iter().map(|arg| self.word_to_string(arg)).collect::<Vec<_>>().join(" ");
                            format!("{} {}", simple_cmd.name, args)
                        }
                        Command::Subshell(subshell_cmd) => {
                            // For subshells in process substitution, we need to execute the inner command
                            match &**subshell_cmd {
                                Command::Simple(simple_cmd) => {
                                    let args = simple_cmd.args.iter().map(|arg| self.word_to_string(arg)).collect::<Vec<_>>().join(" ");
                                    format!("{} {}", simple_cmd.name, args)
                                }
                                Command::Pipeline(pipeline) => {
                                    // Handle pipeline in subshell
                                    let mut cmd_parts = Vec::new();
                                    for cmd in pipeline.commands.iter() {
                                        if let Command::Simple(simple_cmd) = cmd {
                                            let args = simple_cmd.args.iter().map(|arg| self.word_to_string(arg)).collect::<Vec<_>>().join(" ");
                                            cmd_parts.push(format!("{} {}", simple_cmd.name, args));
                                        }
                                    }
                                    cmd_parts.join(" | ")
                                }
                                _ => {
                                    // For other command types, generate the command without the subshell wrapper
                                    match &**subshell_cmd {
                                        Command::Simple(simple_cmd) => {
                                            let args = simple_cmd.args.iter().map(|arg| self.word_to_string(arg)).collect::<Vec<_>>().join(" ");
                                            format!("{} {}", simple_cmd.name, args)
                                        }
                                        _ => self.command_to_string(&**subshell_cmd),
                                    }
                                }
                            }
                        }
                        _ => self.command_to_string(&**cmd),
                    };
                    
                    // Clean up the command string for system call and properly escape it
                    let clean_cmd = cmd_str.replace('\n', " ").replace("  ", " ");
                    // Use proper Rust system call syntax
                    output.push_str(&format!("let _ = Command::new(\"sh\")\n"));
                    output.push_str(&self.indent());
                    output.push_str(&format!("    .arg(\"-c\")\n"));
                    output.push_str(&self.indent());
                    output.push_str(&format!("    .arg(\"{} > {}\")\n", clean_cmd, temp_var));
                    output.push_str(&self.indent());
                    output.push_str("    .status();\n");
                    process_sub_files.push((temp_var, temp_file));
                }
                RedirectOperator::ProcessSubstitutionOutput(_cmd) => {
                    // Process substitution output: >(command)
                    temp_file_counter += 1;
                    let temp_file = format!("/tmp/process_sub_out_{}_{}.tmp", std::process::id(), temp_file_counter);
                    let temp_var = format!("temp_file_out_{}", temp_file_counter);
                    output.push_str(&format!("let {} = \"{}\";\n", temp_var, temp_file));
                    process_sub_files.push((temp_var, temp_file));
                }
                RedirectOperator::HereString => {
                    // Here-string: command <<< "string"
                    has_here_string = true;
                    if let Some(body) = &redir.heredoc_body {
                        // Use a variable to store the here-string content
                        output.push_str(&format!("let here_string_content = \"{}\";\n", self.escape_rust_string(body)));
                    }
                }
                RedirectOperator::Input => {
                    // Check if this input redirect looks like a process substitution
                    // The parser might not have converted this to ProcessSubstitutionInput
                    if redir.target.starts_with("(") && redir.target.ends_with(")") {
                        // This looks like a process substitution, create a temp file
                        temp_file_counter += 1;
                        let temp_file = format!("/tmp/process_sub_input_{}_{}.tmp", std::process::id(), temp_file_counter);
                        let temp_var = format!("temp_file_input_{}", temp_file_counter);
                        output.push_str(&format!("let {} = \"{}\";\n", temp_var, temp_file));
                        
                        // Extract the command from the target (remove parentheses)
                        let cmd_str = redir.target.trim_start_matches('(').trim_end_matches(')');
                        
                        // For simple commands like printf 'x\ny\n', create the temp file directly
                        if cmd_str.starts_with("printf '") && cmd_str.ends_with("'") {
                            // Extract the content between the quotes
                            let content = &cmd_str[8..cmd_str.len()-1]; // Remove "printf '" and "'"
                            // Create temp file with the content
                            output.push_str(&format!("let _ = fs::write({}, \"{}\");\n", temp_var, content.replace("\\n", "\n")));
                        } else {
                            // For other commands, use system() with proper escaping
                            let clean_cmd = cmd_str.replace('\n', " ").replace("  ", " ");
                            output.push_str(&format!("let _ = Command::new(\"sh\")\n"));
                            output.push_str(&self.indent());
                            output.push_str(&format!("    .arg(\"-c\")\n"));
                            output.push_str(&self.indent());
                            output.push_str(&format!("    .arg(\"{} > {}\")\n", clean_cmd, temp_var));
                            output.push_str(&self.indent());
                            output.push_str("    .status();\n");
                        }
                        process_sub_files.push((temp_var, temp_file));
                    }
                }
                _ => {}
            }
        }
        
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
        } else if cmd.name == "test" || cmd.name == "[" {
            // Special handling for test command
            self.generate_test_command(cmd, &mut output);
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
            // Special handling for cd with tilde expansion
            let dir = if cmd.args.is_empty() { Word::Literal(".".to_string()) } else { cmd.args[0].clone() };
            let dir_str = self.word_to_string(&dir);
            
            if dir_str == "~" {
                // Handle tilde expansion for home directory
                output.push_str("let home = env::var(\"HOME\").or_else(|_| env::var(\"USERPROFILE\"));\n");
                output.push_str("if let Ok(home_path) = home {\n");
                output.push_str(&self.indent());
                output.push_str("    if let Err(_) = env::set_current_dir(home_path) {\n");
                output.push_str(&self.indent());
                output.push_str(&self.indent());
                output.push_str("        return std::process::ExitCode::FAILURE;\n");
                output.push_str(&self.indent());
                output.push_str("    }\n");
                output.push_str("} else {\n");
                output.push_str(&self.indent());
                output.push_str("    return std::process::ExitCode::FAILURE;\n");
                output.push_str("}\n");
            } else if dir_str.starts_with("~/") {
                // Handle tilde expansion with subdirectory
                let subdir = &dir_str[2..]; // Remove "~/"
                output.push_str("let home = env::var(\"HOME\").or_else(|_| env::var(\"USERPROFILE\"));\n");
                output.push_str("if let Ok(home_path) = home {\n");
                output.push_str(&self.indent());
                output.push_str(&format!("    let full_path = format!(\"{{}}/{}\", home_path);\n", subdir));
                output.push_str(&self.indent());
                output.push_str("    if let Err(_) = env::set_current_dir(full_path) {\n");
                output.push_str(&self.indent());
                output.push_str(&self.indent());
                output.push_str("        return std::process::ExitCode::FAILURE;\n");
                output.push_str(&self.indent());
                output.push_str("    }\n");
                output.push_str("} else {\n");
                output.push_str(&self.indent());
                output.push_str("    return std::process::ExitCode::FAILURE;\n");
                output.push_str("}\n");
            } else {
                // Regular directory change
                output.push_str(&format!("if let Err(_) = env::set_current_dir(\"{}\") {{\n", dir_str));
                output.push_str(&self.indent());
                output.push_str("    return std::process::ExitCode::FAILURE;\n");
                output.push_str("}\n");
            }
        } else if cmd.name == "ls" {
            // Special handling for ls with brace expansion support
            if cmd.args.is_empty() {
                // Default to current directory
                output.push_str("if let Ok(entries) = fs::read_dir(\".\") {\n");
                output.push_str(&self.indent());
                output.push_str("    for entry in entries {\n");
                output.push_str(&self.indent());
                output.push_str("        if let Ok(entry) = entry {\n");
                output.push_str(&self.indent());
                output.push_str("            if let Some(name) = entry.file_name().to_str() {\n");
                output.push_str(&self.indent());
                output.push_str("                if name != \".\" && name != \"..\" {\n");
                output.push_str(&self.indent());
                output.push_str("                    println!(\"{}\", name);\n");
                output.push_str(&self.indent());
                output.push_str("                }\n");
                output.push_str(&self.indent());
                output.push_str("            }\n");
                output.push_str(&self.indent());
                output.push_str("        }\n");
                output.push_str(&self.indent());
                output.push_str("    }\n");
                output.push_str("}\n");
            } else {
                // Handle arguments with potential brace expansion
                let mut expanded_args = Vec::new();
                for arg in &cmd.args {
                    if self.word_to_string(arg).starts_with('-') {
                        // Skip flags
                        continue;
                    }
                    // Expand brace expansion in the argument
                    let expanded = self.expand_brace_expansions_in_args(&[arg.clone()]);
                    expanded_args.extend(expanded);
                }
                
                if expanded_args.is_empty() {
                    // No non-flag arguments, default to current directory
                    output.push_str("if let Ok(entries) = fs::read_dir(\".\") {\n");
                    output.push_str(&self.indent());
                    output.push_str("    for entry in entries {\n");
                    output.push_str(&self.indent());
                    output.push_str("        if let Ok(entry) = entry {\n");
                    output.push_str(&self.indent());
                    output.push_str("            if let Some(name) = entry.file_name().to_str() {\n");
                    output.push_str(&self.indent());
                    output.push_str("                if name != \".\" && name != \"..\" {\n");
                    output.push_str(&self.indent());
                    output.push_str("                    println!(\"{}\", name);\n");
                    output.push_str(&self.indent());
                    output.push_str("                }\n");
                    output.push_str(&self.indent());
                    output.push_str("            }\n");
                    output.push_str(&self.indent());
                    output.push_str("        }\n");
                    output.push_str(&self.indent());
                    output.push_str("    }\n");
                    output.push_str("}\n");
                } else {
                    // Handle each expanded argument
                    for arg in expanded_args {
                        if arg.contains('*') {
                            // Handle glob patterns like file_*.txt
                            let pattern = arg.replace('*', ".*");
                            output.push_str(&format!("// List files matching pattern: {}\n", arg));
                            output.push_str("if let Ok(entries) = fs::read_dir(\".\") {\n");
                            output.push_str(&self.indent());
                            output.push_str("    for entry in entries {\n");
                            output.push_str(&self.indent());
                            output.push_str("        if let Ok(entry) = entry {\n");
                            output.push_str(&self.indent());
                            output.push_str("            if let Some(name) = entry.file_name().to_str() {\n");
                            output.push_str(&self.indent());
                            output.push_str(&format!("                if name.contains(\"{}\") {{\n", pattern));
                            output.push_str(&self.indent());
                            output.push_str("                    println!(\"{}\", name);\n");
                            output.push_str(&self.indent());
                            output.push_str("                }\n");
                            output.push_str(&self.indent());
                            output.push_str("            }\n");
                            output.push_str(&self.indent());
                            output.push_str("        }\n");
                            output.push_str(&self.indent());
                            output.push_str("    }\n");
                            output.push_str("}\n");
                        } else {
                            // Regular directory/file
                            output.push_str(&format!("if let Ok(entries) = fs::read_dir(\"{}\") {{\n", arg));
                            output.push_str(&self.indent());
                            output.push_str("    for entry in entries {\n");
                            output.push_str(&self.indent());
                            output.push_str("        if let Ok(entry) = entry {\n");
                            output.push_str(&self.indent());
                            output.push_str("            if let Some(name) = entry.file_name().to_str() {\n");
                            output.push_str(&self.indent());
                            output.push_str("                if name != \".\" && name != \"..\" {\n");
                            output.push_str(&self.indent());
                            output.push_str("                    println!(\"{}\", name);\n");
                            output.push_str(&self.indent());
                            output.push_str("                }\n");
                            output.push_str(&self.indent());
                            output.push_str("            }\n");
                            output.push_str(&self.indent());
                            output.push_str("        }\n");
                            output.push_str(&self.indent());
                            output.push_str("    }\n");
                            output.push_str("}\n");
                        }
                    }
                }
            }
        } else if cmd.name == "grep" {
            // Special handling for grep with enhanced options
            if cmd.args.len() >= 1 {
                // Find the pattern (first non-flag argument)
                let mut pattern = None;
                let mut file = None;
                let mut flags = Vec::new();
                
                for arg in &cmd.args {
                    let arg_str = self.word_to_string(arg);
                    if arg_str.starts_with('-') {
                        flags.push(arg_str);
                    } else if pattern.is_none() {
                        pattern = Some(arg_str);
                    } else if file.is_none() {
                        file = Some(arg_str);
                    }
                }
                
                if let Some(pattern) = pattern {
                    let file: String = file.map_or("STDIN".to_string(), |w| w.as_str().to_string());
                    
                    // Check for -o flag (only matching part)
                    let only_matching = flags.iter().any(|flag| flag == "-o");
                    
                    if only_matching {
                        if file == "STDIN" {
                            if has_here_string {
                                // Use string splitting to process here-string content directly
                                output.push_str("let here_lines: Vec<&str> = here_string_content.split('\\n').collect();\n");
                                output.push_str("for line in here_lines {\n");
                                output.push_str(&self.indent());
                                output.push_str(&format!("    if let Ok(re) = regex::Regex::new(\"{}\") {{\n", pattern));
                                output.push_str(&self.indent());
                                output.push_str("        if let Some(captures) = re.captures(line) {\n");
                                output.push_str(&self.indent());
                                output.push_str("            if let Some(m) = captures.get(1) {\n");
                                output.push_str(&self.indent());
                                output.push_str("                println!(\"{}\", m.as_str());\n");
                                output.push_str(&self.indent());
                                output.push_str("            }\n");
                                output.push_str(&self.indent());
                                output.push_str("        }\n");
                                output.push_str(&self.indent());
                                output.push_str("    }\n");
                                output.push_str("}\n");
                            } else {
                                output.push_str("let stdin = io::stdin();\n");
                                output.push_str("let mut buffer = String::new();\n");
                                output.push_str("while stdin.read_line(&mut buffer).unwrap() > 0 {\n");
                                output.push_str(&self.indent());
                                output.push_str(&format!("    if let Ok(re) = regex::Regex::new(\"{}\") {{\n", pattern));
                                output.push_str(&self.indent());
                                output.push_str("        if let Some(captures) = re.captures(&buffer) {\n");
                                output.push_str(&self.indent());
                                output.push_str("            if let Some(m) = captures.get(1) {\n");
                                output.push_str(&self.indent());
                                output.push_str("                println!(\"{}\", m.as_str());\n");
                                output.push_str(&self.indent());
                                output.push_str("            }\n");
                                output.push_str(&self.indent());
                                output.push_str("        }\n");
                                output.push_str(&self.indent());
                                output.push_str("    }\n");
                                output.push_str(&self.indent());
                                output.push_str("    buffer.clear();\n");
                                output.push_str("}\n");
                            }
                        } else {
                            output.push_str(&format!("if let Ok(content) = fs::read_to_string(\"{}\") {{\n", file));
                            output.push_str(&self.indent());
                            output.push_str(&format!("    if let Ok(re) = regex::Regex::new(\"{}\") {{\n", pattern));
                            output.push_str(&self.indent());
                            output.push_str("        for line in content.lines() {\n");
                            output.push_str(&self.indent());
                            output.push_str("            if let Some(captures) = re.captures(line) {\n");
                            output.push_str(&self.indent());
                            output.push_str("                if let Some(m) = captures.get(1) {\n");
                            output.push_str(&self.indent());
                            output.push_str("                    println!(\"{}\", m.as_str());\n");
                            output.push_str(&self.indent());
                            output.push_str("                }\n");
                            output.push_str(&self.indent());
                            output.push_str("            }\n");
                            output.push_str(&self.indent());
                            output.push_str("        }\n");
                            output.push_str(&self.indent());
                            output.push_str("    }\n");
                            output.push_str("}\n");
                        }
                    } else {
                        if file == "STDIN" {
                            if has_here_string {
                                // Use string splitting to process here-string content directly
                                output.push_str("let here_lines: Vec<&str> = here_string_content.split('\\n').collect();\n");
                                output.push_str("for line in here_lines {\n");
                                output.push_str(&self.indent());
                                output.push_str(&format!("    if let Ok(re) = regex::Regex::new(\"{}\") {{\n", pattern));
                                output.push_str(&self.indent());
                                output.push_str("        if re.is_match(line) {\n");
                                output.push_str(&self.indent());
                                output.push_str("            println!(\"{}\", line);\n");
                                output.push_str(&self.indent());
                                output.push_str("        }\n");
                                output.push_str(&self.indent());
                                output.push_str("    }\n");
                                output.push_str("}\n");
                            } else {
                                output.push_str("let stdin = io::stdin();\n");
                                output.push_str("let mut buffer = String::new();\n");
                                output.push_str("while stdin.read_line(&mut buffer).unwrap() > 0 {\n");
                                output.push_str(&self.indent());
                                output.push_str(&format!("    if let Ok(re) = regex::Regex::new(\"{}\") {{\n", pattern));
                                output.push_str(&self.indent());
                                output.push_str("        if re.is_match(&buffer) {\n");
                                output.push_str(&self.indent());
                                output.push_str("            print!(\"{}\", buffer);\n");
                                output.push_str(&self.indent());
                                output.push_str("        }\n");
                                output.push_str(&self.indent());
                                output.push_str("    }\n");
                                output.push_str(&self.indent());
                                output.push_str("    buffer.clear();\n");
                                output.push_str("}\n");
                            }
                        } else {
                            output.push_str(&format!("if let Ok(content) = fs::read_to_string(\"{}\") {{\n", file));
                            output.push_str(&self.indent());
                            output.push_str(&format!("    if let Ok(re) = regex::Regex::new(\"{}\") {{\n", pattern));
                            output.push_str(&self.indent());
                            output.push_str("        for line in content.lines() {\n");
                            output.push_str(&self.indent());
                            output.push_str("            if re.is_match(line) {\n");
                            output.push_str(&self.indent());
                            output.push_str("                println!(\"{}\", line);\n");
                            output.push_str(&self.indent());
                            output.push_str("            }\n");
                            output.push_str(&self.indent());
                            output.push_str("        }\n");
                            output.push_str(&self.indent());
                            output.push_str("    }\n");
                            output.push_str("}\n");
                        }
                    }
                }
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
            // Special handling for rm with brace expansion support
            if !cmd.args.is_empty() {
                // Expand brace expansion in arguments
                let expanded_args = self.expand_brace_expansions_in_args(&cmd.args);
                
                // Remove each file
                for arg in expanded_args {
                    if arg.contains('*') {
                        // Handle glob patterns like file_*.txt
                        output.push_str(&format!("// Remove files matching pattern: {}\n", arg));
                        output.push_str("if let Ok(entries) = fs::read_dir(\".\") {\n");
                        output.push_str(&self.indent());
                        output.push_str("    for entry in entries {\n");
                        output.push_str(&self.indent());
                        output.push_str("        if let Ok(entry) = entry {\n");
                        output.push_str(&self.indent());
                        output.push_str("            if let Some(name) = entry.file_name().to_str() {\n");
                        output.push_str(&self.indent());
                        output.push_str(&format!("                if name.contains(\"{}\") {{\n", arg.replace('*', ".*")));
                        output.push_str(&self.indent());
                        output.push_str("                    let _ = fs::remove_file(entry.path());\n");
                        output.push_str(&self.indent());
                        output.push_str("                }\n");
                        output.push_str(&self.indent());
                        output.push_str("            }\n");
                        output.push_str(&self.indent());
                        output.push_str("        }\n");
                        output.push_str(&self.indent());
                        output.push_str("    }\n");
                        output.push_str("}\n");
                    } else {
                        // Regular file
                        output.push_str(&format!("if let Err(_) = fs::remove_file(\"{}\") {{\n", arg));
                        output.push_str(&self.indent());
                        output.push_str("    return std::process::ExitCode::FAILURE;\n");
                        output.push_str(&self.indent());
                        output.push_str("}\n");
                    }
                }
            }
        } else if cmd.name == "mv" {
            // Special handling for mv
            if cmd.args.len() >= 2 {
                let src = &cmd.args[0];
                let dst = &cmd.args[1];
                output.push_str(&format!("if let Err(_) = fs::rename(\"{}\", \"{}\") {{\n", src, dst));
                output.push_str(&self.indent());
                output.push_str("    return std::process::ExitCode::FAILURE;\n");
                output.push_str(&self.indent());
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
                output.push_str(&self.indent());
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
                output.push_str(&self.indent());
                output.push_str("}\n");
                output.push_str(&format!("let {v} = {v}.trim().to_string();\n", v = var_name));
            }
        } else if cmd.name == "printf" {
            // Handle printf command with format strings and arguments
            if cmd.args.is_empty() {
                output.push_str("println!();\n");
            } else {
                let format_str = &cmd.args[0];
                let args = &cmd.args[1..];
                
                if args.is_empty() {
                    // Single format string argument
                    let format_str_rust = self.convert_shell_printf_to_rust_format(&self.word_to_string(format_str));
                    output.push_str(&format!("print!(\"{}\");\n", format_str_rust));
                } else {
                    // Format string with arguments
                    let format_str_rust = self.convert_shell_printf_to_rust_format(&self.word_to_string(format_str));
                    let mut format_args = Vec::new();
                    for arg in args {
                        match arg {
                            Word::Literal(s) => {
                                // Handle literal strings like "Name", "Age", "City"
                                format_args.push(format!("\"{}\"", self.escape_rust_string(s)));
                            }
                            Word::Variable(var) => {
                                format_args.push(format!("{}", var));
                            }
                            Word::StringInterpolation(interp) => {
                                // Handle string interpolation in printf arguments
                                let content = self.convert_string_interpolation_to_rust(interp);
                                format_args.push(content);
                            }
                            Word::Arithmetic(expr) => {
                                format_args.push(format!("({})", expr.expression));
                            }
                            Word::ParameterExpansion(pe) => {
                                let pe_str = self.generate_parameter_expansion_rust(pe);
                                format_args.push(pe_str);
                            }
                            _ => {
                                let arg_str = self.word_to_string(arg);
                                format_args.push(format!("\"{}\"", self.escape_rust_string(&arg_str)));
                            }
                        }
                    }
                    
                    if format_args.is_empty() {
                        output.push_str(&format!("print!(\"{}\");\n", format_str_rust));
                    } else {
                        output.push_str(&format!("print!(\"{}\", {});\n", 
                            format_str_rust, format_args.join(", ")));
                    }
                }
            }
        } else if cmd.name == "touch" {
            // Special handling for touch with brace expansion support
            if !cmd.args.is_empty() {
                // Expand brace expansion in arguments
                let expanded_files = self.expand_brace_expansions_in_args(&cmd.args);
                
                // Now create all the files
                for file in expanded_files {
                    output.push_str(&format!("if let Err(_) = fs::write(\"{}\", \"\") {{\n", file));
                    output.push_str(&self.indent());
                    output.push_str("    return std::process::ExitCode::FAILURE;\n");
                    output.push_str(&self.indent());
                    output.push_str("}\n");
                }
            }
        } else if cmd.name == "mapfile" {
            // Handle mapfile command for reading lines into an array
            if cmd.args.len() >= 2 && self.word_to_string(&cmd.args[0]) == "-t" {
                let array_name = &cmd.args[1];
                output.push_str(&format!("let mut {}: Vec<String> = Vec::new();\n", array_name));
                
                // Check if we have process substitution files available
                let mut input_source = "STDIN".to_string();
                let mut file_handle = None;
                
                // Check if we have process substitution files available
                if !process_sub_files.is_empty() {
                    input_source = process_sub_files[0].1.clone();
                } else {
                    // Check if we have a process substitution redirect
                    for redir in &cmd.redirects {
                        match &redir.operator {
                            RedirectOperator::Input => {
                                // Check if this is a process substitution target
                                // The parser might not have converted this to ProcessSubstitutionInput,
                                // so we check if the target looks like a process substitution
                                if (redir.target.starts_with("<(") && redir.target.ends_with(")")) ||
                                   (redir.target.starts_with("(") && redir.target.ends_with(")")) {
                                    // This is a process substitution, we should have a temp file
                                    if let Some(temp_file) = process_sub_files.first() {
                                        input_source = temp_file.1.clone();
                                    } else {
                                        // Fallback to the target as-is (this shouldn't happen)
                                        input_source = redir.target.to_string();
                                    }
                                } else {
                                    input_source = redir.target.to_string();
                                }
                            }
                            _ => {}
                        }
                    }
                }
                
                if input_source == "STDIN" {
                    output.push_str("let stdin = io::stdin();\n");
                    output.push_str("let mut buffer = String::new();\n");
                    output.push_str("while stdin.read_line(&mut buffer).unwrap() > 0 {\n");
                } else {
                    let fh = format!("fh_{}", array_name);
                    file_handle = Some(fh.clone());
                    output.push_str(&format!("let mut {} = fs::File::open(\"{}\").unwrap();\n", fh, input_source));
                    output.push_str(&format!("let mut buffer = String::new();\n"));
                    output.push_str(&format!("while {}.read_line(&mut buffer).unwrap() > 0 {{\n", fh));
                }
                output.push_str(&self.indent());
                output.push_str(&format!("{}.push(buffer.trim().to_string());\n", array_name));
                output.push_str(&self.indent());
                output.push_str("buffer.clear();\n");
                output.push_str("}\n");
                
                if let Some(fh) = file_handle {
                    // No need to close file handle in Rust, it's automatically closed when dropped
                }
            }
        } else if cmd.name == "comm" {
            // Handle comm command for comparing sorted files
            if cmd.args.len() >= 3 {
                let flag = &cmd.args[0];
                let file1 = &cmd.args[1];
                let file2 = &cmd.args[2];
                
                // Check if we have process substitution files
                let mut file1_path = file1.to_string();
                let mut file2_path = file2.to_string();
                
                // Use process substitution files if available
                if !process_sub_files.is_empty() {
                    if process_sub_files.len() >= 1 {
                        file1_path = process_sub_files[0].1.clone();
                    }
                    if process_sub_files.len() >= 2 {
                        file2_path = process_sub_files[1].1.clone();
                    }
                }
                
                output.push_str(&format!("// comm {} {} {}\n", flag, file1, file2));
                output.push_str("let _ = Command::new(\"comm\")\n");
                output.push_str(&self.indent());
                output.push_str(&format!("    .arg(\"{}\")\n", flag));
                output.push_str(&self.indent());
                output.push_str(&format!("    .arg(\"{}\")\n", file1_path));
                output.push_str(&self.indent());
                output.push_str(&format!("    .arg(\"{}\")\n", file2_path));
                output.push_str(&self.indent());
                output.push_str("    .status();\n");
            }
        } else if cmd.name == "diff" {
            // Handle diff command with process substitution
            if cmd.args.is_empty() && !process_sub_files.is_empty() {
                // This is a diff with process substitution redirects
                if process_sub_files.len() >= 2 {
                    let file1 = &process_sub_files[0].1;
                    let file2 = &process_sub_files[1].1;
                    output.push_str(&format!("let _ = Command::new(\"diff\")\n"));
                    output.push_str(&self.indent());
                    output.push_str(&format!("    .arg(\"{}\")\n", file1));
                    output.push_str(&self.indent());
                    output.push_str(&format!("    .arg(\"{}\")\n", file2));
                    output.push_str(&self.indent());
                    output.push_str("    .status();\n");
                }
            } else {
                // Regular diff command
                let args = cmd.args.iter().map(|arg| self.word_to_string(arg)).collect::<Vec<_>>().join(" ");
                output.push_str(&format!("let _ = Command::new(\"diff\")\n"));
                output.push_str(&self.indent());
                output.push_str(&format!("    .arg(\"{}\")\n", args));
                output.push_str(&self.indent());
                output.push_str("    .status();\n");
            }
        } else if cmd.name == "paste" {
            // Handle paste command with process substitution
            if cmd.args.is_empty() && !process_sub_files.is_empty() {
                // This is a paste with process substitution redirects
                if process_sub_files.len() >= 2 {
                    let file1 = &process_sub_files[0].1;
                    let file2 = &process_sub_files[1].1;
                    output.push_str(&format!("let _ = Command::new(\"paste\")\n"));
                    output.push_str(&self.indent());
                    output.push_str(&format!("    .arg(\"{}\")\n", file1));
                    output.push_str(&self.indent());
                    output.push_str(&format!("    .arg(\"{}\")\n", file2));
                    output.push_str(&self.indent());
                    output.push_str("    .status();\n");
                }
            } else {
                // Regular paste command
                let args = cmd.args.iter().map(|arg| self.word_to_string(arg)).collect::<Vec<_>>().join(" ");
                output.push_str(&format!("let _ = Command::new(\"paste\")\n"));
                output.push_str(&self.indent());
                output.push_str(&format!("    .arg(\"{}\")\n", args));
                output.push_str(&self.indent());
                output.push_str("    .status();\n");
            }
        } else if cmd.name == "echo" {
            // Simple: echo is just a print function call
            let args = self.convert_echo_args_to_print_args(&cmd.args);
            if args.starts_with('"') && args.ends_with('"') {
                // Single literal string
                output.push_str(&format!("println!({});\n", args));
            } else if args.starts_with("format!") {
                // Multiple parts that need formatting - use println! with format!
                output.push_str(&format!("println!(\"{{}}\", {});\n", args));
            } else if args.contains(" + ") {
                // String concatenation - convert to format!
                let parts: Vec<&str> = args.split(" + ").collect();
                let format_string = "{}".repeat(parts.len());
                output.push_str(&format!("println!(\"{}\", {});\n", format_string, parts.join(", ")));
            } else {
                // Variable or other expression
                output.push_str(&format!("println!(\"{{}}\", {});\n", args));
            }
        } else {
            // Generic command
            if cmd.args.is_empty() {
                output.push_str(&format!("if let Err(_) = Command::new(\"{}\")\n", cmd.name));
                output.push_str(&self.indent());
                output.push_str("    .status() {\n");
                output.push_str(&self.indent());
                output.push_str("    return std::process::ExitCode::FAILURE;\n");
                output.push_str(&self.indent());
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
                output.push_str(&self.indent());
                output.push_str("}\n");
            }
        }

        // Handle redirects
        for redir in &cmd.redirects {
            match redir.operator {
                RedirectOperator::Input => {
                    // Input redirection: command < file
                    // Check if we have a here-string - if so, skip this since the command will read from $here_string_file
                    if !has_here_string {
                        // Check if this is a process substitution target
                        if redir.target.starts_with("<(") && redir.target.ends_with(")") {
                            // This is a process substitution, we should have a temp file
                            // Don't redirect STDIN here - let the command handle it directly
                            // The command (like mapfile) will use the temp file directly
                        } else if redir.target.starts_with("(") && redir.target.ends_with(")") {
                            // This looks like a process substitution, don't redirect STDIN
                            // The command (like mapfile) will use the temp file directly
                        } else {
                            output.push_str(&format!("// Input redirection to: {}\n", redir.target));
                            output.push_str(&format!("// Note: Input redirection not fully implemented in Rust generator\n"));
                        }
                    }
                }
                RedirectOperator::Output => {
                    // Output redirection: command > file
                    output.push_str(&format!("// Output redirection to: {}\n", redir.target));
                    output.push_str(&format!("// Note: Output redirection not fully implemented in Rust generator\n"));
                }
                RedirectOperator::Append => {
                    // Append redirection: command >> file
                    output.push_str(&format!("// Append redirection to: {}\n", redir.target));
                    output.push_str(&format!("// Note: Append redirection not fully implemented in Rust generator\n"));
                }
                RedirectOperator::Heredoc | RedirectOperator::HeredocTabs => {
                    // Heredoc: command << delimiter
                    // Skip heredoc handling for 'cat' command since it's handled specially in the cat command handler
                    if cmd.name != "cat" {
                        if let Some(body) = &redir.heredoc_body {
                            // Create a temporary file with the heredoc content
                            output.push_str(&format!("let temp_content = \"{}\";\n", self.escape_rust_string(body)));
                            output.push_str("let _ = fs::write(\"/tmp/heredoc_temp\", temp_content);\n");
                            output.push_str("// Note: Heredoc redirection not fully implemented in Rust generator\n");
                        }
                    }
                }
                _ => {
                    // Other redirects not yet implemented
                    output.push_str(&format!("// Redirect {:?} not yet implemented\n", redir.operator));
                }
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
                                if let Some(_next_arg) = cmd.args.iter().skip(1).find(|a| {
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
                            // Declare associative array
                            if let Some(array_name) = cmd.args.get(1) {
                                output.push_str(&format!("let mut {}: std::collections::HashMap<String, String> = std::collections::HashMap::new();\n", array_name));
                            }
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
                output.push_str(&format!("regex::Regex::new(\"{}\").unwrap().is_match({})\n", pattern, var));
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
                
                if modifiers.extglob {
                    // Handle extglob patterns
                    let regex_pattern = self.convert_extglob_to_rust_regex(pattern);
                    if modifiers.nocasematch {
                        output.push_str(&format!("// Case-insensitive extglob pattern test: {} == {}\n", var, pattern));
                        output.push_str(&format!("regex::Regex::new(\"{}\").unwrap().is_match(&{}.to_lowercase())\n", regex_pattern, var));
                    } else {
                        output.push_str(&format!("// Extglob pattern test: {} == {}\n", var, pattern));
                        output.push_str(&format!("regex::Regex::new(\"{}\").unwrap().is_match({})\n", regex_pattern, var));
                    }
                } else {
                    // Regular glob pattern matching - convert glob to regex
                    let regex_pattern = self.convert_glob_to_regex(pattern);
                    if modifiers.nocasematch {
                        // Case-insensitive matching
                        output.push_str(&format!("// Case-insensitive pattern test: {} == {}\n", var, pattern));
                        output.push_str(&format!("regex::Regex::new(\"{}\").unwrap().is_match(&{}.to_lowercase())\n", regex_pattern, var));
                    } else {
                        // Case-sensitive matching
                        output.push_str(&format!("// Pattern test: {} == {}\n", var, pattern));
                        output.push_str(&format!("regex::Regex::new(\"{}\").unwrap().is_match({})\n", regex_pattern, var));
                    }
                }
            } else {
                output.push_str(&format!("// Invalid pattern test: {}\n", expr));
                output.push_str("false");
            }
        } else if expr.contains(" != ") {
            // Pattern matching: [[ $var != pattern ]]
            let parts: Vec<&str> = expr.split(" != ").collect();
            if parts.len() == 2 {
                let var = parts[0].trim();
                let pattern = parts[1].trim();
                
                if modifiers.extglob {
                    // Handle extglob patterns
                    let regex_pattern = self.convert_extglob_to_rust_regex(pattern);
                    if modifiers.nocasematch {
                        output.push_str(&format!("// Case-insensitive extglob pattern test: {} != {}\n", var, pattern));
                        output.push_str(&format!("!regex::Regex::new(\"{}\").unwrap().is_match(&{}.to_lowercase())\n", regex_pattern, var));
                    } else {
                        output.push_str(&format!("// Extglob pattern test: {} != {}\n", var, pattern));
                        output.push_str(&format!("!regex::Regex::new(\"{}\").unwrap().is_match({})\n", regex_pattern, var));
                    }
                } else {
                    // Regular pattern matching
                    let regex_pattern = self.convert_glob_to_regex(pattern);
                    if modifiers.nocasematch {
                        // Case-insensitive matching
                        output.push_str(&format!("// Case-insensitive pattern test: {} != {}\n", var, pattern));
                        output.push_str(&format!("!regex::Regex::new(\"{}\").unwrap().is_match(&{}.to_lowercase())\n", regex_pattern, var));
                    } else {
                        // Case-sensitive matching
                        output.push_str(&format!("// Pattern test: {} != {}\n", var, pattern));
                        output.push_str(&format!("!regex::Regex::new(\"{}\").unwrap().is_match({})\n", regex_pattern, var));
                    }
                }
            } else {
                output.push_str(&format!("// Invalid pattern test: {}\n", expr));
                output.push_str("false");
            }
        } else if expr.contains(" -eq ") {
            // Numeric equality: [[ $var -eq value ]]
            let parts: Vec<&str> = expr.split(" -eq ").collect();
            if parts.len() == 2 {
                let var = parts[0].trim();
                let value = parts[1].trim();
                
                output.push_str(&format!("{} == {}\n", var, value));
            } else {
                output.push_str("false");
            }
        } else if expr.contains(" -ne ") {
            // Numeric inequality: [[ $var -ne value ]]
            let parts: Vec<&str> = expr.split(" -ne ").collect();
            if parts.len() == 2 {
                let var = parts[0].trim();
                let value = parts[1].trim();
                
                output.push_str(&format!("{} != {}\n", var, value));
            } else {
                output.push_str("false");
            }
        } else if expr.contains(" -lt ") {
            // Less than: [[ $var -lt value ]]
            let parts: Vec<&str> = expr.split(" -lt ").collect();
            if parts.len() == 2 {
                let var = parts[0].trim();
                let value = parts[1].trim();
                
                output.push_str(&format!("{} < {}\n", var, value));
            } else {
                output.push_str("false");
            }
        } else if expr.contains(" -le ") {
            // Less than or equal: [[ $var -le value ]]
            let parts: Vec<&str> = expr.split(" -le ").collect();
            if parts.len() == 2 {
                let var = parts[0].trim();
                let value = parts[1].trim();
                
                output.push_str(&format!("{} <= {}\n", var, value));
            } else {
                output.push_str("false");
            }
        } else if expr.contains(" -gt ") {
            // Greater than: [[ $var -gt value ]]
            let parts: Vec<&str> = expr.split(" -gt ").collect();
            if parts.len() == 2 {
                let var = parts[0].trim();
                let value = parts[1].trim();
                
                output.push_str(&format!("{} > {}\n", var, value));
            } else {
                output.push_str("false");
            }
        } else if expr.contains(" -ge ") {
            // Greater than or equal: [[ $var -ge value ]]
            let parts: Vec<&str> = expr.split(" -ge ").collect();
            if parts.len() == 2 {
                let var = parts[0].trim();
                let value = parts[1].trim();
                
                output.push_str(&format!("{} >= {}\n", var, value));
            } else {
                output.push_str("false");
            }
        } else if expr.contains(" -z ") {
            // String is empty: [[ -z $var ]]
            let var_str = expr.replace("-z", "").trim().to_string();
            output.push_str(&format!("{}.is_empty()\n", var_str));
        } else if expr.contains(" -n ") {
            // String is not empty: [[ -n $var ]]
            let var_str = expr.replace("-n", "").trim().to_string();
            output.push_str(&format!("!{}.is_empty()\n", var_str));
        } else if expr.contains(" -f ") {
            // File exists and is regular: [[ -f $var ]]
            let var_str = expr.replace("-f", "").trim().to_string();
            output.push_str(&format!("fs::metadata(\"{}\").map(|m| m.is_file()).unwrap_or(false)\n", var_str));
        } else if expr.contains(" -d ") {
            // Directory exists: [[ -d $var ]]
            let var_str = expr.replace("-d", "").trim().to_string();
            output.push_str(&format!("fs::metadata(\"{}\").map(|m| m.is_dir()).unwrap_or(false)\n", var_str));
        } else if expr.contains(" -e ") {
            // File exists: [[ -e $var ]]
            let var_str = expr.replace("-e", "").trim().to_string();
            output.push_str(&format!("fs::metadata(\"{}\").is_ok()\n", var_str));
        } else if expr.contains(" -r ") {
            // File is readable: [[ -r $var ]]
            let var_str = expr.replace("-r", "").trim().to_string();
            output.push_str(&format!("fs::metadata(\"{}\").map(|m| m.permissions().readonly()).unwrap_or(false)\n", var_str));
        } else if expr.contains(" -w ") {
            // File is writable: [[ -w $var ]]
            let var_str = expr.replace("-w", "").trim().to_string();
            output.push_str(&format!("fs::metadata(\"{}\").map(|m| !m.permissions().readonly()).unwrap_or(false)\n", var_str));
        } else if expr.contains(" -x ") {
            // File is executable: [[ -x $var ]]
            let var_str = expr.replace("-x", "").trim().to_string();
            output.push_str(&format!("fs::metadata(\"{}\").map(|m| m.permissions().mode() & 0o111 != 0).unwrap_or(false)\n", var_str));
        } else {
            // Unknown test expression
            output.push_str(&format!("false // Unknown test: {}\n", expr));
        }
        
        output
    }

    fn generate_pipeline(&mut self, pipeline: &Pipeline) -> String {
        let mut output = String::new();
        
        // Check if we have complex commands that need special handling
        let has_complex_commands = pipeline.commands.iter().any(|cmd| {
            matches!(cmd, Command::For(_)) || 
            (matches!(cmd, Command::Simple(_)) && {
                if let Command::Simple(simple) = cmd {
                    self.word_to_string(&simple.name) == "find"
                } else {
                    false
                }
            })
        });
        
        if pipeline.commands.len() == 1 {
            output.push_str(&self.generate_command(&pipeline.commands[0]));
        } else if has_complex_commands {
            // Handle pipelines with for loops and other complex commands
            // Use unique variable names for each pipeline to avoid redeclaration warnings
            self.pipeline_counter += 1;
            let pipeline_id = self.pipeline_counter;
            
            output.push_str(&format!("let mut output_{}: String = String::new();\n", pipeline_id));
            
            // Check if the first command is a for loop
            if let Command::For(for_loop) = &pipeline.commands[0] {
                // Generate the for loop that builds the output string for the pipeline
                let variable = &for_loop.variable;
                let items = &for_loop.items;
                
                // Convert items to Rust array syntax
                let items_str = if items.len() == 1 {
                    match &items[0] {
                        Word::StringInterpolation(interp) => {
                            if interp.parts.len() == 1 {
                                if let StringPart::MapAccess(map_name, key) = &interp.parts[0] {
                                    if key == "@" {
                                        format!("&{}", map_name)
                                    } else {
                                        format!("&{}", map_name)
                                    }
                                } else if let StringPart::MapKeys(map_name) = &interp.parts[0] {
                                    // This is ${!map[@]} - convert to keys(%map)
                                    format!("{}.keys()", map_name)
                                } else if let StringPart::Variable(var) = &interp.parts[0] {
                                    if var.starts_with("!") && var.ends_with("[@]") {
                                        // This is !map[@] - convert to keys(%map)
                                        let map_name = &var[1..var.len()-3];
                                        format!("{}.keys()", map_name)
                                    } else if var.ends_with("[@]") {
                                        let array_name = &var[..var.len()-3];
                                        format!("&{}", array_name)
                                    } else {
                                        format!("&{}", var)
                                    }
                                } else {
                                    format!("&{}", items[0])
                                }
                            } else {
                                format!("&{}", items[0])
                            }
                        }
                        Word::MapAccess(map_name, key) => {
                            if key == "@" {
                                format!("&{}", map_name)
                            } else {
                                format!("&{}", map_name)
                            }
                        }
                        _ => format!("&{}", items[0])
                    }
                } else {
                    format!("&[{}]", items.iter().map(|s| format!("\"{}\"", self.word_to_string(s))).collect::<Vec<_>>().join(", "))
                };
                
                // Generate the for loop that builds the output string for the pipeline
                output.push_str(&format!("for {} in {} {{\n", variable, items_str));
                // Instead of printing directly, build the output string
                for cmd in &for_loop.body.commands {
                    if let Command::Simple(simple_cmd) = cmd {
                        if simple_cmd.name == "echo" {
                            // Convert echo to building output string
                            let mut echo_parts = Vec::new();
                            for arg in &simple_cmd.args {
                                match arg {
                                    Word::StringInterpolation(interp) => {
                                        // Handle string interpolation by building format string and arguments
                                        let mut format_string = String::new();
                                        let mut format_args = Vec::new();
                                        
                                        for part in &interp.parts {
                                            match part {
                                                StringPart::Literal(lit) => {
                                                    // String literal - add to format string
                                                    format_string.push_str(&self.escape_rust_string(lit));
                                                }
                                                StringPart::Variable(var) => {
                                                    // Variable - add placeholder and keep the part
                                                    format_string.push_str("{}");
                                                    format_args.push(var.clone());
                                                }
                                                StringPart::MapAccess(map_name, key) => {
                                                    // Map access - add placeholder and convert to Rust code
                                                    format_string.push_str("{}");
                                                    format_args.push(self.convert_map_access_to_rust(map_name, key));
                                                }
                                                StringPart::MapKeys(map_name) => {
                                                    // ${!map[@]} -> keys(%map)
                                                    format_string.push_str("{}");
                                                    format_args.push(format!("{}.keys()", map_name));
                                                }
                                                _ => {
                                                    // For any other unhandled cases, add placeholder and convert to string representation
                                                    format_string.push_str("{}");
                                                    format_args.push(format!("{:?}", part));
                                                }
                                            }
                                        }
                                        
                                        if format_args.is_empty() {
                                            // No variables, just output the literal string
                                            echo_parts.push(format!("\"{}\"", format_string));
                                        } else {
                                            // Has variables, use format! macro
                                            echo_parts.push(format!("format!(\"{}\", {})", format_string, format_args.join(", ")));
                                        }
                                    }
                                    _ => {
                                        // For non-interpolated words, just convert normally
                                        echo_parts.push(self.word_to_string(arg));
                                    }
                                }
                            }
                            // For multiple parts, use format! macro to avoid type issues
                            let echo_str = if echo_parts.len() == 1 {
                                echo_parts[0].clone()
                            } else {
                                // Build format string and arguments properly
                                let mut format_string = String::new();
                                let mut format_args = Vec::new();
                                
                                for part in &echo_parts {
                                    if part.starts_with('"') && part.ends_with('"') {
                                        // String literal - add to format string
                                        format_string.push_str(&part[1..part.len()-1]);
                                    } else {
                                        // Variable or expression - add placeholder and keep the part
                                        format_string.push_str("{}");
                                        format_args.push(part.clone());
                                    }
                                }
                                
                                if format_args.is_empty() {
                                    // No variables, just output the literal string
                                    format!("\"{}\\n\"", format_string)
                                } else {
                                    // Has variables, use format! macro
                                    format!("format!(\"{}\\n\", {})", format_string, format_args.join(", "))
                                }
                            };
                            output.push_str(&self.indent());
                            output.push_str(&format!("output_{} += &({});\n", pipeline_id, echo_str));
                        } else {
                            // For other commands, generate normally but capture output
                            output.push_str(&self.indent());
                            output.push_str(&format!("// Execute command: {}\n", simple_cmd.name));
                            output.push_str(&self.indent());
                            output.push_str(&format!("let cmd_output = Command::new(\"{}\")\n", simple_cmd.name));
                            output.push_str(&self.indent());
                            output.push_str("    .output();\n");
                            output.push_str(&self.indent());
                            output.push_str(&format!("if let Ok(output) = cmd_output {{\n"));
                            output.push_str(&self.indent());
                            output.push_str(&format!("    output_{} += &String::from_utf8_lossy(&output.stdout);\n", pipeline_id));
                            output.push_str(&self.indent());
                            output.push_str("}\n");
                        }
                    } else {
                        // For non-simple commands, generate normally but capture output
                        output.push_str(&self.indent());
                        output.push_str("// Execute complex command\n");
                        output.push_str(&self.indent());
                        output.push_str(&format!("let cmd_output = Command::new(\"sh\")\n"));
                        output.push_str(&self.indent());
                        output.push_str("    .arg(\"-c\")\n");
                        output.push_str(&self.indent());
                        output.push_str(&format!("    .arg(\"{}\")\n", self.command_to_string(cmd)));
                        output.push_str(&self.indent());
                        output.push_str("    .output();\n");
                        output.push_str(&self.indent());
                        output.push_str(&format!("if let Ok(output) = cmd_output {{\n"));
                        output.push_str(&self.indent());
                        output.push_str(&format!("    output_{} += &String::from_utf8_lossy(&output.stdout);\n", pipeline_id));
                        output.push_str(&self.indent());
                        output.push_str("}\n");
                    }
                }
                output.push_str("}\n");
            } else {
                // First command - check if it's a find command and handle specially
                if let Command::Simple(cmd) = &pipeline.commands[0] {
                    if cmd.name == "find" {
                        // Handle find command with Windows compatibility
                        let mut find_args = Vec::new();
                        for arg in &cmd.args {
                            match arg {
                                Word::Literal(s) => {
                                    if s == "." {
                                        find_args.push(".".to_string());
                                    } else if s == "-name" {
                                        find_args.push("-name".to_string());
                                    } else {
                                        // Convert glob pattern to regex for find
                                        let pattern = self.convert_glob_to_regex(s);
                                        find_args.push(pattern);
                                    }
                                }
                                Word::StringInterpolation(interp) => {
                                    if interp.parts.len() == 1 {
                                        if let StringPart::Literal(s) = &interp.parts[0] {
                                            // Convert glob pattern to regex for find
                                            let pattern = self.convert_glob_to_regex(s);
                                            find_args.push(pattern);
                                        } else {
                                            find_args.push(self.convert_string_interpolation_to_rust(interp));
                                        }
                                    } else {
                                        find_args.push(self.convert_string_interpolation_to_rust(interp));
                                    }
                                }
                                _ => find_args.push(self.word_to_string(arg))
                            }
                        }
                        
                        // Use Rust's walkdir instead of system find for cross-platform compatibility
                        if find_args.len() >= 3 && find_args[1] == "-name" {
                            let pattern = &find_args[2];
                            let dir = &find_args[0];
                            output.push_str(&format!("// Find files matching pattern: {} in directory: {}\n", pattern, dir));
                            output.push_str("if let Ok(entries) = fs::read_dir(dir) {\n");
                            output.push_str(&self.indent());
                            output.push_str("    for entry in entries {\n");
                            output.push_str(&self.indent());
                            output.push_str("        if let Ok(entry) = entry {\n");
                            output.push_str(&self.indent());
                            output.push_str("            if let Some(name) = entry.file_name().to_str() {\n");
                            output.push_str(&self.indent());
                            output.push_str(&format!("                if regex::Regex::new(\"{}\").unwrap().is_match(name) {{\n", pattern));
                            output.push_str(&self.indent());
                            output.push_str(&format!("                    output_{} += &format!(\"{{}}\\n\", entry.path().display());\n", pipeline_id));
                            output.push_str(&self.indent());
                            output.push_str("                }\n");
                            output.push_str(&self.indent());
                            output.push_str("            }\n");
                            output.push_str(&self.indent());
                            output.push_str("        }\n");
                            output.push_str(&self.indent());
                            output.push_str("    }\n");
                            output.push_str("}\n");
                        } else {
                            // Fallback to system find command
                            let cmd_str = self.command_to_string(&pipeline.commands[0]);
                            output.push_str(&format!("let cmd_output = Command::new(\"find\")\n"));
                            output.push_str(&self.indent());
                            output.push_str(&format!("    .arg(\"{}\")\n", cmd_str));
                            output.push_str(&self.indent());
                            output.push_str("    .output();\n");
                            output.push_str(&self.indent());
                            output.push_str(&format!("if let Ok(output) = cmd_output {{\n"));
                            output.push_str(&self.indent());
                            output.push_str(&format!("    output_{} += &String::from_utf8_lossy(&output.stdout);\n", pipeline_id));
                            output.push_str(&self.indent());
                            output.push_str("}\n");
                        }
                    }
                }
            }
            
            // Now process the output through the remaining pipeline commands
            for cmd in &pipeline.commands[1..] {
                output.push_str(&format!("// Process output through: {}\n", self.command_to_string(cmd)));
                output.push_str(&format!("let cmd_output = Command::new(\"sh\")\n"));
                output.push_str(&self.indent());
                output.push_str("    .arg(\"-c\")\n");
                output.push_str(&self.indent());
                output.push_str(&format!("    .arg(\"{}\")\n", self.command_to_string(cmd)));
                output.push_str(&self.indent());
                output.push_str("    .stdin(std::process::Stdio::piped())\n");
                output.push_str(&self.indent());
                output.push_str("    .output();\n");
                output.push_str(&self.indent());
                output.push_str(&format!("if let Ok(mut child) = Command::new(\"sh\")\n"));
                output.push_str(&self.indent());
                output.push_str("    .arg(\"-c\")\n");
                output.push_str(&self.indent());
                output.push_str(&format!("    .arg(\"{}\")\n", self.command_to_string(cmd)));
                output.push_str(&self.indent());
                output.push_str("    .stdin(std::process::Stdio::piped())\n");
                output.push_str(&self.indent());
                output.push_str("    .spawn() {\n");
                output.push_str(&self.indent());
                output.push_str("    if let Some(mut stdin) = child.stdin.take() {\n");
                output.push_str(&self.indent());
                output.push_str(&format!("        let _ = stdin.write_all(output_{}.as_bytes());\n", pipeline_id));
                output.push_str(&self.indent());
                output.push_str("    }\n");
                output.push_str(&self.indent());
                output.push_str(&format!("    if let Ok(output) = child.wait_with_output() {{\n"));
                output.push_str(&self.indent());
                output.push_str(&format!("        output_{} = String::from_utf8_lossy(&output.stdout).to_string();\n", pipeline_id));
                output.push_str(&self.indent());
                output.push_str("    }\n");
                output.push_str(&self.indent());
                output.push_str("}\n");
            }
            
            // Print the final output
            output.push_str(&format!("print!(\"{{}}\", output_{});\n", pipeline_id));
        } else {
            // Simplified: execute sequentially; no external piping
            for cmd in &pipeline.commands {
                output.push_str(&self.generate_command(cmd));
            }
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
            Word::Literal(s) => self.escape_rust_string(s),
            Word::Variable(var) => {
                // Special case for $@ - convert to Rust equivalent
                if var == "@" {
                    "std::env::args().skip(1).collect::<Vec<_>>()".to_string()
                } else if var == "#" {
                    "std::env::args().skip(1).count()".to_string()
                } else {
                    var.clone()
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
                    self.convert_map_access_to_rust(map_name, key)
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
                // Handle brace expansion by expanding it to actual values
                if expansion.items.len() == 1 {
                    match &expansion.items[0] {
                        BraceItem::Range(range) => {
                            // Expand range like {1..5} to "1 2 3 4 5"
                            // Check if this is a character range
                            if let (Some(start_char), Some(end_char)) = (range.start.chars().next(), range.end.chars().next()) {
                                if start_char.is_ascii_lowercase() && end_char.is_ascii_lowercase() {
                                    // This is a character range
                                    let start = start_char as u8;
                                    let end = end_char as u8;
                                    if start <= end {
                                        let step = range.step.as_ref().and_then(|s| s.parse::<usize>().ok()).unwrap_or(1);
                                        let values: Vec<String> = (start..=end)
                                            .step_by(step)
                                            .map(|c| char::from(c).to_string())
                                            .collect();
                                        values.join(" ")
                                    } else {
                                        // Reverse range
                                        let step = range.step.as_ref().and_then(|s| s.parse::<usize>().ok()).unwrap_or(1);
                                        let values: Vec<String> = (end..=start)
                                            .rev()
                                            .step_by(step)
                                            .map(|c| char::from(c).to_string())
                                            .collect();
                                        values.join(" ")
                                    }
                                } else {
                                    // This is a numeric range
                                    self.expand_brace_range(range)
                                }
                            } else {
                                // This is a numeric range
                                self.expand_brace_range(range)
                            }
                        }
                        BraceItem::Literal(s) => {
                            // Handle literal strings that might contain ranges like "a..c" or "00..04..2"
                            if s.contains("..") {
                                let parts: Vec<&str> = s.split("..").collect();
                                if parts.len() == 2 {
                                    // Simple range like "a..c"
                                    if let (Some(start_char), Some(end_char)) = (parts[0].chars().next(), parts[1].chars().next()) {
                                        if start_char.is_ascii_lowercase() && end_char.is_ascii_lowercase() {
                                            let start = start_char as u8;
                                            let end = end_char as u8;
                                            if start <= end {
                                                let values: Vec<String> = (start..=end)
                                                    .map(|c| char::from(c).to_string())
                                                    .collect();
                                                values.join(" ")
                                            } else {
                                                s.clone()
                                            }
                                        } else {
                                            s.clone()
                                        }
                                    } else {
                                        s.clone()
                                    }
                                } else if parts.len() == 3 && parts[1].contains("..") {
                                    // Character range with step like "a..z..3"
                                    let sub_parts: Vec<&str> = parts[1].split("..").collect();
                                    if sub_parts.len() == 2 {
                                        if let (Some(start_char), Some(end_char)) = (parts[0].chars().next(), sub_parts[1].chars().next()) {
                                            if start_char.is_ascii_lowercase() && end_char.is_ascii_lowercase() {
                                                if let Ok(step) = parts[2].parse::<usize>() {
                                                    let start = start_char as u8;
                                                    let end = end_char as u8;
                                                    if start <= end {
                                                        let values: Vec<String> = (start..=end)
                                                            .step_by(step)
                                                            .map(|c| char::from(c).to_string())
                                                            .collect();
                                                        values.join(" ")
                                                    } else {
                                                        s.clone()
                                                    }
                                                } else {
                                                    s.clone()
                                                }
                                            } else {
                                                s.clone()
                                            }
                                        } else {
                                            s.clone()
                                        }
                                    } else {
                                        s.clone()
                                    }
                                } else if parts.len() == 3 {
                                    // Range with step like "00..04..2"
                                    if let (Ok(start), Ok(end), Ok(step)) = (parts[0].parse::<i64>(), parts[1].parse::<i64>(), parts[2].parse::<i64>()) {
                                        let values: Vec<String> = (start..=end).step_by(step as usize).map(|i| {
                                            // Preserve leading zeros by formatting with the same width as the original
                                            if parts[0].starts_with('0') && parts[0].len() > 1 {
                                                format!("{:0width$}", i, width = parts[0].len())
                                            } else {
                                                i.to_string()
                                            }
                                        }).collect();
                                        values.join(" ")
                                    } else {
                                        s.clone()
                                    }
                                } else {
                                    s.clone()
                                }
                            } else {
                                s.clone()
                            }
                        }
                        BraceItem::Sequence(seq) => {
                            // Expand sequence like {a,b,c} to "a b c"
                            seq.join(" ")
                        }
                    }
                } else {
                    // Multiple items - expand each one and join
                    let expanded_items: Vec<Vec<String>> = expansion.items.iter().map(|item| {
                        match item {
                            BraceItem::Literal(s) => vec![s.clone()],
                            BraceItem::Range(range) => {
                                self.expand_brace_range(range).split_whitespace().map(|s| s.to_string()).collect()
                            }
                            BraceItem::Sequence(seq) => seq.clone()
                        }
                    }).collect();
                    
                    // Generate cartesian product for multiple brace expansions like {a,b,c}{1,2,3}
                    if expanded_items.len() == 2 {
                        let mut result = Vec::new();
                        for item1 in &expanded_items[0] {
                            for item2 in &expanded_items[1] {
                                result.push(format!("{}{}", item1, item2));
                            }
                        }
                        result.join(" ")
                    } else {
                        // For more than 2 items, just join them (this could be enhanced for full cartesian product)
                        expanded_items.iter().map(|items| items.join(" ")).collect::<Vec<_>>().join(" ")
                    }
                }
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
                            } else if var == "#" {
                                result.push_str("std::env::args().skip(1).count()");
                            } else {
                                result.push_str(var);
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
                                    result.push_str(&self.convert_map_access_to_rust(map_name, key));
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
    
    fn simple_pattern_match(&self, pattern: &str, text: &str) -> bool {
        // Simple pattern matching without regex - handle basic glob patterns
        if pattern.contains('*') {
            // Convert glob pattern to simple string matching
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 1 {
                // No wildcards
                text == parts[0]
            } else if parts.len() == 2 {
                // Single wildcard: "prefix*suffix"
                text.starts_with(parts[0]) && text.ends_with(parts[1])
            } else {
                // Multiple wildcards - simple implementation
                let mut pos = 0;
                for (i, part) in parts.iter().enumerate() {
                    if i == 0 {
                        // First part must match at start
                        if !text.starts_with(part) {
                            return false;
                        }
                        pos = part.len();
                    } else if i == parts.len() - 1 {
                        // Last part must match at end
                        if !text.ends_with(part) {
                            return false;
                        }
                    } else {
                        // Middle parts must be found in order
                        if let Some(found_pos) = text[pos..].find(part) {
                            pos += found_pos + part.len();
                        } else {
                            return false;
                        }
                    }
                }
                true
            }
        } else {
            // No wildcards, exact match
            text == pattern
        }
    }
    
    fn expand_brace_expression(&self, s: &str) -> Option<Vec<String>> {
        // Simple implementation for brace expansion
        if !(s.starts_with('{') && s.ends_with('}')) {
            return None;
        }
        let inner = &s[1..s.len() - 1];
        let parts: Vec<&str> = inner.split(',').collect();
        Some(parts.iter().map(|s| s.to_string()).collect())
    }

    fn expand_brace_expansions_in_args(&self, args: &[Word]) -> Vec<String> {
        let mut result = Vec::new();
        
        for arg in args {
            match arg {
                Word::BraceExpansion(expansion) => {
                    // Handle brace expansion
                    if expansion.items.len() == 1 {
                        match &expansion.items[0] {
                            BraceItem::Literal(s) => {
                                // Check if this is a range like "a..c" or "00..04..2"
                                if s.contains("..") {
                                    let parts: Vec<&str> = s.split("..").collect();
                                    if parts.len() == 2 {
                                        if let (Some(start_char), Some(end_char)) = (parts[0].chars().next(), parts[1].chars().next()) {
                                            if start_char.is_ascii_lowercase() && end_char.is_ascii_lowercase() {
                                                let start = start_char as u8;
                                                let end = end_char as u8;
                                                if start <= end {
                                                    let values: Vec<String> = (start..=end)
                                                        .map(|c| char::from(c).to_string())
                                                        .collect();
                                                    result.extend(values);
                                                    continue;
                                                }
                                            }
                                        }
                                    } else if parts.len() == 3 {
                                        // Range with step like "00..04..2"
                                        if let (Ok(start), Ok(end), Ok(step)) = (parts[0].parse::<i64>(), parts[1].parse::<i64>(), parts[2].parse::<i64>()) {
                                            let values: Vec<String> = (start..=end).step_by(step as usize).map(|i| i.to_string()).collect();
                                            result.extend(values);
                                            continue;
                                        }
                                    }
                                }
                                result.push(s.clone());
                            }
                            BraceItem::Range(range) => {
                                if let (Ok(start), Ok(end)) = (range.start.parse::<i64>(), range.end.parse::<i64>()) {
                                    let step = range.step.as_ref().and_then(|s| s.parse::<i64>().ok()).unwrap_or(1);
                                    let values: Vec<String> = if step > 0 {
                                        (start..=end).step_by(step as usize).map(|i| {
                                            // Preserve leading zeros by formatting with the same width as the original
                                            if range.start.starts_with('0') && range.start.len() > 1 {
                                                format!("{:0width$}", i, width = range.start.len())
                                            } else {
                                                i.to_string()
                                            }
                                        }).collect()
                                    } else {
                                        (end..=start).rev().step_by((-step) as usize).map(|i| {
                                            if range.start.starts_with('0') && range.start.len() > 1 {
                                                format!("{:0width$}", i, width = range.start.len())
                                            } else {
                                                i.to_string()
                                            }
                                        }).collect()
                                    };
                                    result.extend(values);
                                } else {
                                    result.push(format!("{{{}}}..{{{}}}", range.start, range.end));
                                }
                            }
                            BraceItem::Sequence(seq) => {
                                result.extend(seq.iter().cloned());
                            }
                        }
                    } else {
                        // Multiple items - expand each one
                        for item in &expansion.items {
                            match item {
                                BraceItem::Literal(s) => result.push(s.clone()),
                                BraceItem::Range(range) => {
                                    if let (Ok(start), Ok(end)) = (range.start.parse::<i64>(), range.end.parse::<i64>()) {
                                        let step = range.step.as_ref().and_then(|s| s.parse::<i64>().ok()).unwrap_or(1);
                                        let values: Vec<String> = if step > 0 {
                                            (start..=end).step_by(step as usize).map(|i| i.to_string()).collect()
                                        } else {
                                            (end..=start).rev().step_by((-step) as usize).map(|i| i.to_string()).collect()
                                        };
                                        result.extend(values);
                                    } else {
                                        result.push(format!("{{{}}}..{{{}}}", range.start, range.end));
                                    }
                                }
                                BraceItem::Sequence(seq) => {
                                    result.extend(seq.iter().cloned());
                                }
                            }
                        }
                    }
                }
                _ => {
                    // Regular argument
                    let arg_str = self.word_to_string(arg);
                    result.push(arg_str);
                }
            }
        }
        
        result
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
                        // Handle shell variable syntax in the key
                        let rust_key = if key.starts_with('$') {
                            // Extract variable name from shell syntax like $foo
                            &key[1..]
                        } else if self.is_valid_variable_name(key) {
                            // This looks like a variable name (e.g., 'foo' in map[foo])
                            // Treat it as a variable, not a string literal
                            key
                        } else {
                            // String literal key - quote it
                            &format!("\"{}\"", key)
                        };
                        output.push_str(&format!("    {}.get({}).unwrap_or(&String::new()),\n", map_name, rust_key));
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
                                } else if var == "#" {
                                    interp_parts.push("std::env::args().skip(1).count()".to_string());
                                } else {
                                    interp_parts.push(var.clone());
                                }
                            }
                            StringPart::MapAccess(map_name, key) => {
                                if key == "@" {
                                    interp_parts.push(format!("{}", map_name));
                                } else {
                                    interp_parts.push(self.convert_map_access_to_rust(map_name, key));
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
                        }
                    }
                    // Join the parts and convert to string
                    if interp_parts.len() == 1 {
                        output.push_str(&format!("    {}.to_string(),\n", interp_parts[0]));
                    } else {
                        // Use format! macro for proper string concatenation
                        let format_parts: Vec<String> = interp_parts.iter().map(|part| {
                            if part.starts_with('"') && part.ends_with('"') {
                                // It's a string literal, keep the quotes for the format string
                                part.clone()
                            } else {
                                // It's a variable or expression, use as-is
                                part.clone()
                            }
                        }).collect();
                        
                        // Build the format string by replacing quoted strings with {} placeholders
                        let mut format_string = String::new();
                        let mut format_args = Vec::new();
                        
                        for part in &format_parts {
                            if part.starts_with('"') && part.ends_with('"') {
                                // String literal - add to format string and remove quotes
                                format_string.push_str(&part[1..part.len()-1]);
                            } else {
                                // Variable or expression - add placeholder and keep the part
                                format_string.push_str("{}");
                                format_args.push(part.clone());
                            }
                        }
                        
                        if format_args.is_empty() {
                            // No variables, just output the literal string
                            output.push_str(&format!("    \"{}\",\n", format_string));
                        } else {
                            // Has variables, use format! macro
                            output.push_str(&format!("    format!(\"{}\", {}),\n", 
                                format_string, format_args.join(", ")));
                        }
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

    fn convert_shell_printf_to_rust_format(&self, format_str: &str) -> String {
        // Convert shell printf format strings like "%-10s %-10s %s" to Rust format strings like "{:<10} {:<10} {}"
        let mut result = String::new();
        let mut chars = format_str.chars().peekable();
        let mut arg_index = 0;
        
        while let Some(ch) = chars.next() {
            if ch == '%' {
                if let Some(next_ch) = chars.peek() {
                    match next_ch {
                        '-' => {
                            // Handle left-justified format like %-10s
                            chars.next(); // consume the '-'
                            
                            // Parse width
                            let mut width = String::new();
                            while let Some(width_ch) = chars.peek() {
                                if width_ch.is_ascii_digit() {
                                    width.push(chars.next().unwrap());
                                } else {
                                    break;
                                }
                            }
                            
                            // Parse format specifier
                            if let Some(spec) = chars.next() {
                                match spec {
                                    's' => result.push_str(&format!("{{:<{}}}", width)),
                                    'd' => result.push_str(&format!("{{:<{}}}", width)),
                                    'f' => result.push_str(&format!("{{:<{}.2}}", width)),
                                    'x' => result.push_str(&format!("{{:<{}}}", width)),
                                    'X' => result.push_str(&format!("{{:<{}}}", width)),
                                    'o' => result.push_str(&format!("{{:<{}}}", width)),
                                    _ => result.push_str(&format!("{{:<{}}}", width)), // default to string
                                }
                                arg_index += 1;
                            }
                        }
                        '0'..='9' => {
                            // Handle right-justified format like %10s
                            let mut width = String::new();
                            while let Some(width_ch) = chars.peek() {
                                if width_ch.is_ascii_digit() {
                                    width.push(chars.next().unwrap());
                                } else {
                                    break;
                                }
                            }
                            
                            // Parse format specifier
                            if let Some(spec) = chars.next() {
                                match spec {
                                    's' => result.push_str(&format!("{{:>{}}}", width)),
                                    'd' => result.push_str(&format!("{{:>{}}}", width)),
                                    'f' => result.push_str(&format!("{{:>{}.2}}", width)),
                                    'x' => result.push_str(&format!("{{:>{}}}", width)),
                                    'X' => result.push_str(&format!("{{:>{}}}", width)),
                                    'o' => result.push_str(&format!("{{:>{}}}", width)),
                                    _ => result.push_str(&format!("{{:>{}}}", width)), // default to string
                                }
                                arg_index += 1;
                            }
                        }
                        's' => {
                            result.push_str("{}");
                            chars.next(); // consume the 's'
                            arg_index += 1;
                        }
                        'd' => {
                            result.push_str("{}");
                            chars.next(); // consume the 'd'
                            arg_index += 1;
                        }
                        'f' => {
                            result.push_str("{:.2}");
                            chars.next(); // consume the 'f'
                            arg_index += 1;
                        }
                        'x' => {
                            result.push_str("{:x}");
                            chars.next(); // consume the 'x'
                            arg_index += 1;
                        }
                        'X' => {
                            result.push_str("{:X}");
                            chars.next(); // consume the 'X'
                            arg_index += 1;
                        }
                        'o' => {
                            result.push_str("{:o}");
                            chars.next(); // consume the 'o'
                            arg_index += 1;
                        }
                        'n' => {
                            result.push_str("\\n");
                            chars.next(); // consume the 'n'
                        }
                        't' => {
                            result.push_str("\\t");
                            chars.next(); // consume the 't'
                        }
                        'r' => {
                            result.push_str("\\r");
                            chars.next(); // consume the 'r'
                        }
                        _ => {
                            // Unknown format specifier, treat as literal
                            result.push(ch);
                        }
                    }
                } else {
                    // Trailing %, treat as literal
                    result.push(ch);
                }
            } else if ch == '{' {
                result.push_str("{{");
            } else if ch == '}' {
                result.push_str("}}");
            } else {
                result.push(ch);
            }
        }
        
        result
    }

    fn word_to_rust_format_string(&self, word: &Word) -> String {
        match word {
            Word::Literal(s) => {
                // Convert shell printf format strings to Rust format strings
                let mut result = String::new();
                let mut chars = s.chars().peekable();
                while let Some(ch) = chars.next() {
                    if ch == '{' {
                        if let Some(next_ch) = chars.peek() {
                            if next_ch == &'{' {
                                result.push('{');
                                chars.next(); // Consume the second '{'
                            } else {
                                result.push_str("{{");
                            }
                        } else {
                            result.push_str("{{");
                        }
                    } else if ch == '}' {
                        if let Some(next_ch) = chars.peek() {
                            if next_ch == &'}' {
                                result.push('}');
                                chars.next(); // Consume the second '}'
                            } else {
                                result.push_str("}}");
                            }
                        } else {
                            result.push_str("}}");
                        }
                    } else {
                        result.push(ch);
                    }
                }
                result
            }
            Word::Variable(var) => {
                // Handle string interpolation in variables
                if var == "@" {
                    "std::env::args().skip(1).collect::<Vec<_>>()".to_string()
                } else if var == "#" {
                    "std::env::args().skip(1).count()".to_string()
                } else {
                    var.clone()
                }
            }
            Word::StringInterpolation(interp) => {
                let mut result = String::new();
                for part in &interp.parts {
                    match part {
                        StringPart::Literal(s) => result.push_str(s),
                        StringPart::Variable(var) => {
                            // Handle string interpolation in variables
                            if var == "@" {
                                result.push_str("std::env::args().skip(1).collect::<Vec<_>>()");
                            } else if var == "#" {
                                result.push_str("std::env::args().skip(1).count()");
                            } else {
                                result.push_str(var);
                            }
                        },
                        StringPart::ParameterExpansion(pe) => {
                            let pe_str = self.generate_parameter_expansion_rust(pe);
                            result.push_str(&pe_str);
                        }
                        _ => {
                            // For any other unhandled cases, convert to string representation
                            result.push_str(&format!("\"{:?}\"", part));
                        }
                    }
                }
                result
            }
            Word::Arithmetic(expr) => {
                format!("({})", expr.expression)
            }
            Word::MapAccess(map_name, key) => {
                if key == "@" {
                    format!("{}", map_name)
                } else {
                    // Handle shell variable syntax in the key
                    if key.starts_with('$') {
                        let var_name = &key[1..];
                        format!("{}.get({}).unwrap_or(&String::new())", map_name, var_name)
                    } else if key.contains('$') {
                        // Handle case where key contains $ but doesn't start with it
                        // This happens when the parser treats ${map[$k]} as MapAccess("map", "$k")
                        let var_name = key.replace('$', "");
                        format!("{}.get({}).unwrap_or(&String::new())", map_name, var_name)
                    } else if self.is_valid_variable_name(key) {
                        // This looks like a variable name (e.g., 'foo' in map[foo])
                        // Treat it as a variable, not a string literal
                        format!("{}.get({}).unwrap_or(&String::new())", map_name, key)
                    } else {
                        // String literal key - quote it
                        format!("{}.get(\"{}\").unwrap_or(&String::new())", map_name, key)
                    }
                }
            }
            Word::MapKeys(map_name) => {
                format!("{}", map_name)
            }
            Word::MapLength(map_name) => {
                format!("{}.len()", map_name)
            }
            _ => word.to_string(), // Fallback for other Word types
        }
    }

    fn convert_string_interpolation_to_rust(&self, interp: &StringInterpolation) -> String {
        let mut result = String::new();
        for part in &interp.parts {
            match part {
                StringPart::Literal(s) => result.push_str(s),
                StringPart::Variable(var) => {
                    // Handle string interpolation in variables
                    if var == "@" {
                        result.push_str("std::env::args().skip(1).collect::<Vec<_>>()");
                    } else if var == "#" {
                        result.push_str("std::env::args().skip(1).count()");
                    } else {
                        result.push_str(var);
                    }
                }
                StringPart::ParameterExpansion(pe) => {
                    let pe_str = self.generate_parameter_expansion_rust(pe);
                    result.push_str(&pe_str);
                }
                StringPart::MapAccess(map_name, key) => {
                    result.push_str(&self.convert_map_access_to_rust(map_name, key));
                }
                StringPart::MapKeys(map_name) => {
                    result.push_str(&format!("{}.keys().collect::<Vec<_>>().join(\" \")", map_name));
                }
                StringPart::MapLength(map_name) => {
                    result.push_str(&format!("{}.len()", map_name));
                }
                _ => {
                    // For any other unhandled cases, convert to string representation
                    result.push_str(&format!("\"{:?}\"", part));
                }
            }
        }
        result
    }

    fn convert_map_access_to_rust(&self, map_name: &str, key: &str) -> String {
        // Helper function to convert map/array access to proper Rust code
        // This handles both indexed arrays (Vec) and associative arrays (HashMap)
        
        // Check if this looks like an indexed array (numeric key)
        if let Ok(index) = key.parse::<usize>() {
            // This is an indexed array access like arr[1]
            // For Vec<&str>, we need to handle the type mismatch
            format!("{}.get({}).unwrap_or(&\"\")", map_name, index)
        } else if key.starts_with('$') {
            // This is a variable key like $k
            let var_name = &key[1..];
            format!("{}.get({}).unwrap_or(&String::new())", map_name, var_name)
        } else if self.is_valid_variable_name(key) {
            // This looks like a variable name (e.g., 'foo' in map[foo])
            // Treat it as a variable, not a string literal
            format!("{}.get({}).unwrap_or(&String::new())", map_name, key)
        } else {
            // This is a string literal key like "foo" or contains special characters
            format!("{}.get(\"{}\").unwrap_or(&String::new())", map_name, key)
        }
    }

    fn is_valid_variable_name(&self, name: &str) -> bool {
        // Check if a string looks like a valid variable name
        // Valid variable names in shell: alphanumeric + underscore, starting with letter or underscore
        if name.is_empty() {
            return false;
        }
        
        let first_char = name.chars().next().unwrap();
        if !first_char.is_ascii_alphabetic() && first_char != '_' {
            return false;
        }
        
        // All characters must be alphanumeric or underscore
        name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    }

    fn generate_parameter_expansion_rust(&self, pe: &ParameterExpansion) -> String {
        match &pe.operator {
            ParameterExpansionOperator::UppercaseAll => format!("{}.to_uppercase()", pe.variable),
            ParameterExpansionOperator::LowercaseAll => format!("{}.to_lowercase()", pe.variable),
            ParameterExpansionOperator::UppercaseFirst => format!("{}.chars().next().unwrap_or(' ').to_uppercase().collect::<String>() + &{}[1..]", pe.variable, pe.variable),
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
    }

    fn convert_glob_to_regex(&self, pattern: &str) -> String {
        // Simple glob to regex conversion
        let mut result = String::new();
        for ch in pattern.chars() {
            match ch {
                '.' => result.push_str("\\."),
                '*' => result.push_str(".*"),
                '?' => result.push_str("."),
                _ => result.push(ch),
            }
        }
        result
    }

    fn convert_extglob_to_rust_regex(&self, pattern: &str) -> String {
        // Simple extended glob to regex conversion
        self.convert_glob_to_regex(pattern)
    }

    fn command_to_string(&mut self, command: &Command) -> String {
        match command {
            Command::Simple(cmd) => {
                let mut result = self.word_to_string(&cmd.name);
                if !cmd.args.is_empty() {
                    result.push_str(" ");
                    result.push_str(&cmd.args.iter().map(|arg| self.word_to_string(arg)).collect::<Vec<_>>().join(" "));
                }
                result
            }
            Command::Pipeline(pipeline) => {
                pipeline.commands.iter().map(|cmd| self.command_to_string(cmd)).collect::<Vec<_>>().join(" | ")
            }
            Command::If(if_stmt) => {
                format!("if {}; then {}; fi", 
                    self.command_to_string(&if_stmt.condition),
                    self.command_to_string(&if_stmt.then_branch))
            }
            Command::While(while_loop) => {
                format!("while {}; do {}; done", 
                    self.command_to_string(&while_loop.condition),
                    self.generate_block(&while_loop.body))
            }
            Command::For(for_loop) => {
                let items = if for_loop.items.is_empty() {
                    "\"$@\"".to_string()
                } else {
                    for_loop.items.iter().map(|item| self.word_to_string(item)).collect::<Vec<_>>().join(" ")
                };
                format!("for {} in {}; do {}; done", 
                    for_loop.variable, items, self.generate_block(&for_loop.body))
            }
            Command::Function(func) => {
                format!("function {}() {{ {} }}", func.name, self.generate_block(&func.body))
            }
            Command::Subshell(cmd) => {
                format!("({})", self.command_to_string(cmd))
            }
            Command::Background(cmd) => {
                format!("{} &", self.command_to_string(cmd))
            }
            Command::Block(block) => {
                format!("{{ {} }}", self.generate_block(block))
            }
            Command::BuiltinCommand(cmd) => {
                let mut result = cmd.name.clone();
                if !cmd.args.is_empty() {
                    result.push_str(" ");
                    result.push_str(&cmd.args.iter().map(|arg| self.word_to_string(arg)).collect::<Vec<_>>().join(" "));
                }
                result
            }
            Command::TestExpression(test_expr) => {
                format!("[[ {} ]]", test_expr.expression)
            }
            Command::ShoptCommand(cmd) => {
                if cmd.enable {
                    format!("shopt -s {}", cmd.option)
                } else {
                    format!("shopt -u {}", cmd.option)
                }
            }
            Command::BlankLine => "".to_string(),
        }
    }

    fn expand_brace_range(&self, range: &crate::ast::BraceRange) -> String {
        // First check if this is a character range
        if let (Some(start_char), Some(end_char)) = (range.start.chars().next(), range.end.chars().next()) {
            if start_char.is_ascii_lowercase() && end_char.is_ascii_lowercase() {
                // This is a character range
                let start = start_char as u8;
                let end = end_char as u8;
                if start <= end {
                    let step = range.step.as_ref().and_then(|s| s.parse::<usize>().ok()).unwrap_or(1);
                    let values: Vec<String> = (start..=end)
                        .step_by(step)
                        .map(|c| char::from(c).to_string())
                        .collect();
                    values.join(" ")
                } else {
                    // Reverse range
                    let step = range.step.as_ref().and_then(|s| s.parse::<usize>().ok()).unwrap_or(1);
                    let values: Vec<String> = (end..=start)
                        .rev()
                        .step_by(step)
                        .map(|c| char::from(c).to_string())
                        .collect();
                    values.join(" ")
                }
            } else {
                // Try numeric range
                if let (Ok(start), Ok(end)) = (range.start.parse::<i64>(), range.end.parse::<i64>()) {
                    let step = range.step.as_ref().and_then(|s| s.parse::<i64>().ok()).unwrap_or(1);
                    let values: Vec<String> = if step > 0 {
                        (start..=end).step_by(step as usize).map(|i| {
                            // Preserve leading zeros by formatting with the same width as the original
                            if range.start.starts_with('0') && range.start.len() > 1 {
                                format!("{:0width$}", i, width = range.start.len())
                            } else {
                                i.to_string()
                            }
                        }).collect()
                    } else {
                        (end..=start).rev().step_by((-step) as usize).map(|i| {
                            if range.start.starts_with('0') && range.start.len() > 1 {
                                format!("{:0width$}", i, width = range.start.len())
                            } else {
                                i.to_string()
                            }
                        }).collect()
                    };
                    values.join(" ")
                } else {
                    // If parsing fails, fall back to literal
                    format!("{{{}}}", range.start)
                }
            }
        } else {
            // Try numeric range
            if let (Ok(start), Ok(end)) = (range.start.parse::<i64>(), range.end.parse::<i64>()) {
                let step = range.step.as_ref().and_then(|s| s.parse::<i64>().ok()).unwrap_or(1);
                let values: Vec<String> = if step > 0 {
                    (start..=end).step_by(step as usize).map(|i| {
                        // Preserve leading zeros by formatting with the same width as the original
                        if range.start.starts_with('0') && range.start.len() > 1 {
                            format!("{:0width$}", i, width = range.start.len())
                        } else {
                            i.to_string()
                        }
                    }).collect()
                } else {
                    (end..=start).rev().step_by((-step) as usize).map(|i| {
                        if range.start.starts_with('0') && range.start.len() > 1 {
                            format!("{:0width$}", i, width = range.start.len())
                        } else {
                            i.to_string()
                        }
                    }).collect()
                };
                values.join(" ")
            } else {
                // If parsing fails, fall back to literal
                format!("{{{}}}", range.start)
            }
        }
    }

    fn extract_var_name(arg: &str) -> Option<String> {
        SharedUtils::extract_var_name(arg)
    }

    fn generate_test_command(&mut self, cmd: &SimpleCommand, output: &mut String) {
        // Convert test conditions to Rust
        if cmd.args.len() == 3 {
            // Format: [ operand1 operator operand2 ]
            let operand1 = &cmd.args[0];
            let operator = &cmd.args[1];
            let operand2 = &cmd.args[2];
            
            match operator.as_str() {
                "-lt" => {
                    output.push_str(&format!("{} < {}", operand1, operand2));
                }
                "-le" => {
                    output.push_str(&format!("{} <= {}", operand1, operand2));
                }
                "-eq" => {
                    output.push_str(&format!("{} == {}", operand1, operand2));
                }
                "-ne" => {
                    output.push_str(&format!("{} != {}", operand1, operand2));
                }
                "-gt" => {
                    output.push_str(&format!("{} > {}", operand1, operand2));
                }
                "-ge" => {
                    output.push_str(&format!("{} >= {}", operand1, operand2));
                }
                _ => {
                    output.push_str(&format!("{} {} {}", operand1, operator, operand2));
                }
            }
        } else if cmd.args.len() >= 2 {
            let operator = &cmd.args[0];
            let operand = &cmd.args[1];
            
            match operator.as_str() {
                "-f" => {
                    output.push_str(&format!("std::path::Path::new({}).is_file()", self.word_to_string(operand)));
                }
                "-d" => {
                    output.push_str(&format!("std::path::Path::new({}).is_dir()", self.word_to_string(operand)));
                }
                "-e" => {
                    output.push_str(&format!("std::path::Path::new({}).exists()", self.word_to_string(operand)));
                }
                "-r" => {
                    output.push_str(&format!("std::fs::metadata({}).map(|m| m.permissions().readonly()).unwrap_or(false)", self.word_to_string(operand)));
                }
                "-w" => {
                    output.push_str(&format!("std::fs::metadata({}).map(|m| !m.permissions().readonly()).unwrap_or(false)", self.word_to_string(operand)));
                }
                "-x" => {
                    output.push_str(&format!("std::fs::metadata({}).map(|m| m.permissions().mode() & 0o111 != 0).unwrap_or(false)", self.word_to_string(operand)));
                }
                "-z" => {
                    output.push_str(&format!("{}.is_empty()", self.word_to_string(operand)));
                }
                "-n" => {
                    output.push_str(&format!("!{}.is_empty()", self.word_to_string(operand)));
                }
                _ => {
                    output.push_str(&format!("{} {} {}", self.word_to_string(operand), operator, self.word_to_string(operand)));
                }
            }
        }
    }

    fn convert_echo_args_to_print_args(&self, args: &[Word]) -> String {
        if args.is_empty() {
            return "\"\\n\"".to_string();
        }
        
        let mut parts = Vec::new();
        for arg in args {
            match arg {
                Word::Literal(s) => {
                    parts.push(format!("\"{}\"", self.escape_rust_string(s)));
                }
                Word::Variable(var) => {
                    if var == "#" {
                        parts.push("env::args().len() - 1".to_string());
                    } else if var == "@" {
                        parts.push("env::args().skip(1).collect::<Vec<_>>().join(\" \")".to_string());
                    } else if var.starts_with('#') && var.ends_with("[@]") {
                        let array_name = &var[1..var.len()-3];
                        parts.push(format!("{}.len()", array_name));
                    } else if var.starts_with('#') && var.ends_with("[*]") {
                        let array_name = &var[1..var.len()-3];
                        parts.push(format!("{}.len()", array_name));
                    } else if var.starts_with('!') && var.ends_with("[@]") {
                        let array_name = &var[1..var.len()-3];
                        parts.push(format!("{}.keys().cloned().collect::<Vec<_>>().join(\" \")", array_name));
                    } else if var.starts_with('!') && var.ends_with("[*]") {
                        let array_name = &var[1..var.len()-3];
                        parts.push(format!("{}.keys().cloned().collect::<Vec<_>>().join(\" \")", array_name));
                    } else {
                        parts.push(format!("{}", var));
                    }
                }
                Word::StringInterpolation(interp) => {
                    if interp.parts.len() == 1 {
                        if let StringPart::Literal(s) = &interp.parts[0] {
                            parts.push(format!("\"{}\"", self.escape_rust_string(s)));
                        } else if let StringPart::Variable(var) = &interp.parts[0] {
                            if var == "#" {
                                parts.push("env::args().len() - 1".to_string());
                            } else if var == "@" {
                                parts.push("env::args().skip(1).collect::<Vec<_>>().join(\" \")".to_string());
                            } else {
                                parts.push(format!("{}", var));
                            }
                        } else if let StringPart::MapAccess(map_name, key) = &interp.parts[0] {
                            if map_name == "map" {
                                parts.push(format!("map.get({}).unwrap_or(&String::new())", key));
                            } else {
                                parts.push(format!("{}.get({}).unwrap_or(&String::new())", map_name, key));
                            }
                        } else {
                            parts.push(self.convert_string_interpolation_to_rust(interp));
                        }
                    } else {
                        // Multiple parts - handle each part separately
                        for part in &interp.parts {
                            match part {
                                StringPart::Literal(s) => {
                                    parts.push(format!("\"{}\"", self.escape_rust_string(s)));
                                }
                                StringPart::Variable(var) => {
                                    if var == "#" {
                                        parts.push("env::args().len() - 1".to_string());
                                    } else if var == "@" {
                                        parts.push("env::args().skip(1).collect::<Vec<_>>().join(\" \")".to_string());
                                    } else {
                                        parts.push(format!("{}", var));
                                    }
                                }
                                _ => {
                                    parts.push(self.convert_string_interpolation_to_rust(interp));
                                }
                            }
                        }
                    }
                }
                Word::BraceExpansion(expansion) => {
                    // Use the helper method to expand brace expansions
                    let expanded = self.expand_brace_expansions_in_args(&[arg.clone()]);
                    parts.push(format!("\"{}\"", expanded.join(" ")));
                }
                _ => {
                    parts.push(self.word_to_string(arg));
                }
            }
        }
        
        // For Rust, we need to use println! with format! for proper string handling
        if parts.len() == 1 {
            parts[0].clone()
        } else {
            // Use format! macro for multiple parts to avoid string concatenation issues
            format!("format!(\"{}\", {})", "{}".repeat(parts.len()), parts.join(", "))
        }
    }

}




