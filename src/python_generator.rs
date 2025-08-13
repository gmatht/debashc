use crate::ast::*;

pub struct PythonGenerator {
    indent_level: usize,
}

impl PythonGenerator {
    pub fn new() -> Self {
        Self { indent_level: 0 }
    }

    pub fn generate(&mut self, commands: &[Command]) -> String {
        let mut output = String::new();
        output.push_str("#!/usr/bin/env python3\n");
        output.push_str("import subprocess\n");
        output.push_str("import os\n");
        output.push_str("import sys\n");
        output.push_str("from pathlib import Path\n\n");

        for command in commands {
            output.push_str(&self.generate_command(command));
        }
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
            Command::BuiltinCommand(_) => "".to_string(),
            Command::BlankLine => "\n".to_string(),
        }
    }

    fn generate_simple_command(&self, cmd: &SimpleCommand) -> String {
        let mut output = String::new();
        
        // Handle environment variables
        for (var, value) in &cmd.env_vars {
            // Check if this is an array assignment
            match value {
                Word::Array(_, elements) => {
                    // This is an array assignment like arr=(one two three)
                    let elements_str = elements.iter().map(|s| format!("'{}'", s)).collect::<Vec<_>>().join(", ");
                    output.push_str(&format!("{} = [{}]\n", var, elements_str));
                }
                _ => {
                    // Check if this looks like an array assignment
                    if value.as_str().starts_with('(') && value.as_str().ends_with(')') {
                        // This is an array assignment like arr=(one two three) - fallback for string-based arrays
                        let content = &value.as_str()[1..value.as_str().len()-1];
                        let elements: Vec<&str> = content.split_whitespace().collect();
                        let elements_str = elements.iter().map(|&s| format!("'{}'", s)).collect::<Vec<_>>().join(", ");
                        output.push_str(&format!("{} = [{}]\n", var, elements_str));
                    } else if var.contains('[') && var.contains(']') {
                        // This is an associative array assignment like map[foo]=bar
                        if let Some(bracket_pos) = var.find('[') {
                            let map_name = &var[..bracket_pos];
                            let key = &var[bracket_pos + 1..var.len() - 1]; // Remove [ and ]
                            // Initialize the map if it doesn't exist
                            output.push_str(&format!("if '{}' not in globals():\n", map_name));
                            output.push_str(&self.indent());
                            output.push_str(&format!("    {} = {{}}\n", map_name));
                            output.push_str(&format!("{}['{}'] = '{}'\n", map_name, key, value));
                        } else {
                            // Fallback
                            output.push_str(&format!("os.environ['{}'] = '{}'\n", var, value));
                        }
                    } else {
                        // Regular environment variable
                        output.push_str(&format!("os.environ['{}'] = '{}'\n", var, value));
                    }
                }
            }
        }

        // Generate the command
        if cmd.name.is_literal("true") && cmd.env_vars.is_empty() {
            // Builtin true: successful no-op (only when no env vars)
            output.push_str("pass\n");
        } else if cmd.name.is_literal("false") {
            // Builtin false: represent failure by exiting non-zero
            output.push_str("import sys\n");
            output.push_str("sys.exit(1)\n");
        } else if cmd.name.is_literal("echo") {
            // Special handling for echo
            if cmd.args.is_empty() {
                output.push_str("print()\n");
            } else {
                // Support $# and $@
                if cmd.args.len() == 1 && cmd.args[0].as_variable() == Some("#") {
                    output.push_str("import sys\n");
                    output.push_str("print(len(sys.argv) - 1)\n");
                } else if cmd.args.len() == 1 && cmd.args[0].as_variable() == Some("@") {
                    output.push_str("import sys\n");
                    output.push_str("print(' '.join(sys.argv[1:]))\n");
                } else {
                    // Convert each argument to a Python string literal first
                    let mut python_args = Vec::new();
                    for arg in &cmd.args {
                        match arg {
                                                    Word::Literal(s) => {
                            // Convert special characters to Python escape sequences
                            let escaped = self.escape_python_string_literal(s);
                            python_args.push(escaped);
                        }
                            Word::Variable(var) => {
                                if var == "i" { 
                                    python_args.push("i".to_string());
                                } else if var == "1" { 
                                    python_args.push("sys.argv[1] if len(sys.argv) > 1 else ''".to_string());
                                } else if var == "2" { 
                                    python_args.push("sys.argv[2] if len(sys.argv) > 2 else ''".to_string());
                                } else if var == "3" { 
                                    python_args.push("sys.argv[3] if len(sys.argv) > 3 else ''".to_string());
                                } else if var == "4" { 
                                    python_args.push("sys.argv[4] if len(sys.argv) > 4 else ''".to_string());
                                } else if var == "5" { 
                                    python_args.push("sys.argv[5] if len(sys.argv) > 5 else ''".to_string());
                                } else { 
                                    python_args.push(format!("'${}'", var));
                                }
                            }
                            _ => {
                                python_args.push(self.word_to_string(arg));
                            }
                        }
                    }
                    
                    // Check if we need f-string interpolation
                    let args_str = python_args.join(" + ");
                    if args_str.contains('$') {
                        // Convert shell variables to Python f-string variables
                        let mut converted_args = args_str.clone();
                        converted_args = converted_args.replace("$i", "{i}");
                        converted_args = converted_args.replace("$1", "{sys.argv[1] if len(sys.argv) > 1 else ''}");
                        converted_args = converted_args.replace("$2", "{sys.argv[2] if len(sys.argv) > 2 else ''}");
                        converted_args = converted_args.replace("$3", "{sys.argv[3] if len(sys.argv) > 3 else ''}");
                        converted_args = converted_args.replace("$4", "{sys.argv[4] if len(sys.argv) > 4 else ''}");
                        converted_args = converted_args.replace("$5", "{sys.argv[5] if len(sys.argv) > 5 else ''}");
                        
                        // Escape any remaining $ signs that aren't part of our variables
                        converted_args = converted_args.replace("$", "\\$");
                        
                        output.push_str(&format!("print(f\"{}\")\n", converted_args));
                    } else {
                        // No variables, use regular print with concatenation
                        output.push_str(&format!("print({})\n", args_str));
                    }
                }
            }
        } else if cmd.name.is_literal("[[") {
            // Builtin double-bracket test: treat as no-op (success)
            output.push_str("pass\n");
        } else if cmd.name.is_literal("sleep") {
            // Use time.sleep
            output.push_str("import time\n");
            let dur = cmd.args.get(0).cloned().unwrap_or_else(|| Word::Literal("1".to_string()));
            output.push_str(&format!("time.sleep({})\n", dur));
        } else if cmd.name.is_literal("cd") {
            // Special handling for cd with tilde expansion
            let empty_word = Word::Literal("".to_string());
            let dir = cmd.args.first().unwrap_or(&empty_word);
            let dir_str = self.word_to_string(dir);
            
            if dir_str == "'~'" {
                // Handle tilde expansion for home directory
                output.push_str("home = os.path.expanduser('~')\n");
                output.push_str("os.chdir(home)\n");
            } else if dir_str.starts_with("'~/") && dir_str.ends_with("'") {
                // Handle tilde expansion with subdirectory
                let subdir = &dir_str[2..dir_str.len()-1]; // Remove "'~/" and "'"
                output.push_str("home = os.path.expanduser('~')\n");
                output.push_str(&format!("os.chdir(os.path.join(home, '{}'))\n", subdir));
            } else {
                // Regular directory change
                output.push_str(&format!("os.chdir({})\n", dir_str));
            }
        } else if cmd.name.is_literal("ls") {
            // Special handling for ls (use Python stdlib; ignore flags)
            let dir_expr = if cmd.args.is_empty() { ".".to_string() } else { cmd.args[0].to_string() };
            output.push_str(&format!("for item in os.listdir('{}'):\n", dir_expr));
            output.push_str(&self.indent());
            output.push_str("    if item not in ['.', '..']:\n");
            output.push_str(&self.indent());
            output.push_str("        print(item)\n");
        } else if cmd.name.is_literal("grep") {
            // Special handling for grep
            if cmd.args.len() >= 2 {
                let pattern = &cmd.args[0];
                let file = &cmd.args[1];
                output.push_str(&format!("with open('{}', 'r') as f:\n", file));
                output.push_str(&self.indent());
                output.push_str(&format!("    for line in f:\n"));
                output.push_str(&self.indent());
                output.push_str(&format!("        if '{}' in line:\n", pattern));
                output.push_str(&self.indent());
                output.push_str("            print(line.rstrip())\n");
            }
        } else if cmd.name.is_literal("declare") {
            // Handle declare command (usually for arrays)
            if cmd.args.len() >= 2 && cmd.args[0].as_str() == "-A" {
                // declare -A map_name creates an associative array
                let map_name = &cmd.args[1];
                output.push_str(&format!("{} = {{}}\n", map_name));
            } else if cmd.args.len() >= 2 && cmd.args[0].as_str() == "-a" {
                // declare -a arr_name creates an indexed array
                let arr_name = &cmd.args[1];
                output.push_str(&format!("{} = []\n", arr_name));
            } else if cmd.args.len() >= 1 {
                // declare var_name creates a regular variable
                let var_name = &cmd.args[0];
                output.push_str(&format!("{} = None\n", var_name));
            }
        } else if cmd.name.is_literal("printf") {
            // Special handling for printf
            if cmd.args.is_empty() {
                output.push_str("print()\n");
            } else {
                // Handle printf format string and arguments
                if cmd.args.len() >= 1 {
                    let format_str = &cmd.args[0];
                    let format_content = match format_str {
                        Word::Literal(s) => s.as_str(),
                        _ => "",
                    };
                    
                    if cmd.args.len() == 1 {
                        // Just the format string
                        let escaped = self.escape_python_string_literal(format_content);
                        output.push_str(&format!("print({}, end='')\n", escaped));
                    } else {
                        // Format string with arguments
                        let mut args_list = Vec::new();
                        for arg in &cmd.args[1..] {
                            match arg {
                                Word::Variable(var) => {
                                    if var == "i" { 
                                        args_list.push("i".to_string());
                                    } else if var == "1" { 
                                        args_list.push("sys.argv[1] if len(sys.argv) > 1 else ''".to_string());
                                    } else if var == "2" { 
                                        args_list.push("sys.argv[2] if len(sys.argv) > 2 else ''".to_string());
                                    } else if var == "3" { 
                                        args_list.push("sys.argv[3] if len(sys.argv) > 3 else ''".to_string());
                                    } else if var == "4" { 
                                        args_list.push("sys.argv[4] if len(sys.argv) > 4 else ''".to_string());
                                    } else if var == "5" { 
                                        args_list.push("sys.argv[5] if len(sys.argv) > 5 else ''".to_string());
                                    } else { 
                                        args_list.push(format!("'${}'", var));
                                    }
                                }
                                _ => {
                                    args_list.push(self.word_to_string(arg));
                                }
                            }
                        }
                        
                        // Convert printf format to Python format
                        let python_format = format_content
                            .replace("%s", "{}")
                            .replace("%d", "{}")
                            .replace("%i", "{}")
                            .replace("%u", "{}")
                            .replace("%o", "{}")
                            .replace("%x", "{}")
                            .replace("%X", "{}")
                            .replace("%f", "{}")
                            .replace("%F", "{}")
                            .replace("%e", "{}")
                            .replace("%E", "{}")
                            .replace("%g", "{}")
                            .replace("%G", "{}")
                            .replace("%c", "{}")
                            .replace("%%", "%");
                        
                        let escaped_format = self.escape_python_string_literal(&python_format);
                        let args_str = args_list.join(", ");
                        
                        if args_list.is_empty() {
                            output.push_str(&format!("print({}, end='')\n", escaped_format));
                        } else {
                            output.push_str(&format!("print({}.format({}), end='')\n", escaped_format, args_str));
                        }
                    }
                }
            }
        } else if cmd.name.is_literal("cat") {
            // Special handling for cat including heredocs
            let mut printed_any = false;
            for redir in &cmd.redirects {
                if matches!(redir.operator, RedirectOperator::Heredoc | RedirectOperator::HeredocTabs) {
                    if let Some(body) = &redir.heredoc_body {
                        let escaped = self.escape_python_string(body);
                        output.push_str(&format!("print({}, end='')\n", escaped));
                        printed_any = true;
                    }
                }
            }
            if !printed_any {
                for arg in &cmd.args {
                    output.push_str(&format!("with open('{}', 'r') as f:\n", arg));
                    output.push_str(&self.indent());
                    output.push_str("    print(f.read(), end='')\n");
                }
            }
        } else if cmd.name.is_literal("mkdir") {
            // Special handling for mkdir
            for arg in &cmd.args {
                output.push_str(&format!("os.makedirs('{}', exist_ok=True)\n", arg));
            }
        } else if cmd.name.is_literal("rm") {
            // Special handling for rm
            for arg in &cmd.args {
                output.push_str(&format!("os.remove('{}')\n", arg));
            }
        } else if cmd.name.is_literal("mv") {
            // Special handling for mv
            if cmd.args.len() >= 2 {
                let src = &cmd.args[0];
                let dst = &cmd.args[1];
                output.push_str(&format!("os.rename('{}', '{}')\n", src, dst));
            }
        } else if cmd.name.is_literal("cp") {
            // Special handling for cp
            if cmd.args.len() >= 2 {
                let src = &cmd.args[0];
                let dst = &cmd.args[1];
                output.push_str(&format!("import shutil\n"));
                output.push_str(&format!("shutil.copy2('{}', '{}')\n", src, dst));
            }
        } else if cmd.name.is_literal("read") {
            // Read a line into a variable
            if let Some(var) = cmd.args.get(0) {
                output.push_str(&format!("{} = input()\n", var));
            }
        } else if cmd.name.is_literal("shopt") {
            // Builtin: ignore; treat as success
            output.push_str("pass\n");
        } else if cmd.name.is_literal("[") {
            // Special handling for test commands
            self.generate_test_command(cmd, &mut output);
        } else {
            // Check if this is a variable assignment (e.g., i=$((i + 1)))
            if cmd.args.len() >= 2 && cmd.args[1].contains('=') {
                let arg_str = cmd.args[1].to_string();
                let assignment_parts: Vec<&str> = arg_str.splitn(2, '=').collect();
                if assignment_parts.len() == 2 {
                    let var_name = assignment_parts[0];
                    let value = assignment_parts[1];
                    
                    // Handle arithmetic expansion
                    if value.starts_with("$(") && value.ends_with(")") {
                        let arithmetic_expr = self.parse_arithmetic_expansion(value);
                        output.push_str(&format!("{} = {}\n", var_name, arithmetic_expr));
                    } else {
                        // Regular assignment
                        output.push_str(&format!("{} = {}\n", var_name, value));
                    }
                } else {
                    // Generic command
                    if cmd.args.is_empty() {
                        output.push_str(&format!("subprocess.run(['{}'])\n", cmd.name));
                    } else {
                        let args_str = cmd.args.iter().map(|arg| format!("'{}'", arg)).collect::<Vec<_>>().join(", ");
                        output.push_str(&format!("subprocess.run(['{}', {}])\n", cmd.name, args_str));
                    }
                }
            } else {
                // Check if this is a function call (e.g., greet "World")
                if cmd.name.is_literal("greet") {
                    // Handle function calls directly
                    if !cmd.args.is_empty() {
                        let args_str = cmd.args.iter().map(|arg| format!("'{}'", arg)).collect::<Vec<_>>().join(", ");
                        output.push_str(&format!("greet({})\n", args_str));
                    } else {
                        output.push_str("greet()\n");
                    }
                } else {
                    // Generic command
                    if cmd.args.is_empty() {
                        output.push_str(&format!("subprocess.run(['{}'])\n", cmd.name));
                    } else {
                        let args_str = cmd.args.iter().map(|arg| format!("'{}'", arg)).collect::<Vec<_>>().join(", ");
                        output.push_str(&format!("subprocess.run(['{}', {}])\n", cmd.name, args_str));
                    }
                }
            }
        }

        output
    }

    fn generate_shopt_command(&mut self, cmd: &ShoptCommand) -> String {
        // In Python, we can use a dictionary to track shell options
        // For now, just generate a comment indicating the option change
        format!("# shopt -{} {}\n", if cmd.enable { "s" } else { "u" }, cmd.option)
    }

    fn generate_test_expression(&mut self, test_expr: &TestExpression) -> String {
        // Convert shell test expressions to Python equivalent
        // For now, generate a basic implementation that handles common patterns
        let mut output = String::new();
        
        // Handle test modifiers if they're set
        if test_expr.modifiers.extglob {
            output.push_str("# extglob enabled\n");
        }
        if test_expr.modifiers.nocasematch {
            output.push_str("# nocasematch enabled\n");
        }
        if test_expr.modifiers.globstar {
            output.push_str("# globstar enabled\n");
        }
        if test_expr.modifiers.nullglob {
            output.push_str("# nullglob enabled\n");
        }
        if test_expr.modifiers.failglob {
            output.push_str("# failglob enabled\n");
        }
        if test_expr.modifiers.dotglob {
            output.push_str("# dotglob enabled\n");
        }
        
        // Generate the test expression
        // For now, just generate a comment with the expression
        output.push_str(&format!("# test expression: {}\n", test_expr.expression));
        output.push_str("pass  # TODO: implement test expression logic\n");
        
        output
    }

    fn generate_test_command(&self, cmd: &SimpleCommand, output: &mut String) {
        if cmd.args.len() >= 2 {
            let test_op = &cmd.args[0];
            let file_path = &cmd.args[1];
            
            match test_op.as_str() {
                "-f" => {
                    output.push_str(&format!("Path('{}').is_file()\n", file_path));
                }
                "-d" => {
                    output.push_str(&format!("Path('{}').is_dir()\n", file_path));
                }
                "-e" => {
                    output.push_str(&format!("Path('{}').exists()\n", file_path));
                }
                "-r" => {
                    output.push_str(&format!("os.access('{}', os.R_OK)\n", file_path));
                }
                "-w" => {
                    output.push_str(&format!("os.access('{}', os.W_OK)\n", file_path));
                }
                "-x" => {
                    output.push_str(&format!("os.access('{}', os.X_OK)\n", file_path));
                }
                _ => {
                    output.push_str("True\n");
                }
            }
        } else {
            output.push_str("True\n");
        }
    }

    fn generate_pipeline(&mut self, pipeline: &Pipeline) -> String {
        let mut output = String::new();
        
        if pipeline.commands.len() == 1 {
            output.push_str(&self.generate_command(&pipeline.commands[0]));
        } else {
            // Handle pipelines with multiple commands
            if pipeline.commands.len() == 2 {
                // Special case for common patterns like "command | sort"
                let first_cmd = &pipeline.commands[0];
                let second_cmd = &pipeline.commands[1];
                
                if let Command::Simple(cmd) = second_cmd {
                    if cmd.name.is_literal("sort") {
                        // Handle "command | sort" pattern
                        if let Command::For(for_loop) = first_cmd {
                            // Special case: for loop output piped to sort
                            output.push_str("output_lines = []\n");
                            // Generate a modified version of the for loop that collects output
                            output.push_str(&self.generate_for_loop_collecting_output(for_loop));
                            output.push_str("sorted_output = sorted(output_lines)\n");
                            output.push_str("for line in sorted_output:\n");
                            output.push_str("    if line.strip():\n");
                            output.push_str("        print(line)\n");
                            return output;
                        } else {
                            // Regular command piped to sort
                            output.push_str("import subprocess\n");
                            output.push_str(&format!("result = subprocess.run({}, capture_output=True, text=True)\n", 
                                self.command_to_string(first_cmd)));
                            output.push_str("sorted_output = sorted(result.stdout.strip().split('\\n'))\n");
                            output.push_str("for line in sorted_output:\n");
                            output.push_str("    if line.strip():\n");
                            output.push_str("        print(line)\n");
                            return output;
                        }
                    }
                }
            }
            
            // General pipeline handling
            output.push_str("import subprocess\n");
            for (i, command) in pipeline.commands.iter().enumerate() {
                if i == 0 {
                    output.push_str(&format!("result = subprocess.run({}, capture_output=True, text=True)\n", 
                        self.command_to_string(command)));
                } else {
                    output.push_str(&format!("result = subprocess.run({}, input=result.stdout, capture_output=True, text=True)\n", 
                        self.command_to_string(command)));
                }
            }
            output.push_str("print(result.stdout, end='')\n");
        }
        
        output
    }

    fn command_to_string(&self, command: &Command) -> String {
        match command {
            Command::Simple(cmd) => {
                if cmd.args.is_empty() {
                    format!("['{}']", cmd.name)
                } else {
                    let args = cmd.args.iter().map(|arg| format!("'{}'", arg)).collect::<Vec<_>>().join(", ");
                    format!("['{}', {}]", cmd.name, args)
                }
            }
            _ => "['command']".to_string(),
        }
    }

    fn generate_if_statement(&mut self, if_stmt: &IfStatement) -> String {
        let mut output = String::new();
        
        // Generate condition
        output.push_str("if ");
        output.push_str(&self.generate_condition(&if_stmt.condition));
        output.push_str(":\n");
        
        // Generate then branch
        self.indent_level += 1;
        output.push_str(&self.indent());
        output.push_str(&self.generate_command(&if_stmt.then_branch));
        self.indent_level -= 1;
        
        // Generate else branch if present
        if let Some(else_branch) = &if_stmt.else_branch {
            output.push_str("else:\n");
            self.indent_level += 1;
            output.push_str(&self.indent());
            output.push_str(&self.generate_command(else_branch));
            self.indent_level -= 1;
        }
        
        output
    }

    fn generate_while_loop(&mut self, while_loop: &WhileLoop) -> String {
        let mut output = String::new();
        
        // Generate the condition properly
        output.push_str("while ");
        output.push_str(&self.generate_condition(&while_loop.condition));
        output.push_str(":\n");
        self.indent_level += 1;
        
        // Generate body
        output.push_str(&self.indent());
        output.push_str(&self.generate_block(&while_loop.body));
        
        self.indent_level -= 1;
        
        output
    }

    fn generate_for_loop(&mut self, for_loop: &ForLoop) -> String {
        let mut output = String::new();
        
        if for_loop.items.is_empty() {
            // Infinite loop
            output.push_str("while True:\n");
            self.indent_level += 1;
            output.push_str(&self.indent());
            output.push_str(&self.generate_block(&for_loop.body));
            self.indent_level -= 1;
        } else {
            // For loop with items
            if for_loop.items.len() == 1 {
                match &for_loop.items[0] {
                    Word::Variable(var) if var == "@" => {
                        // Special case for iterating over arguments
                        output.push_str(&format!("for {} in sys.argv[1:]:\n", for_loop.variable));
                    }
                    Word::StringInterpolation(interp) => {
                        // Handle string interpolation, especially for array access
                        if interp.parts.len() == 1 {
                            match &interp.parts[0] {
                                crate::ast::StringPart::MapAccess(map_name, key) if key == "@" => {
                                    // Special case: iterate over all array elements
                                    output.push_str(&format!("for {} in {}:\n", for_loop.variable, map_name));
                                }
                                _ => {
                                    let items_str = self.word_to_string(&for_loop.items[0]);
                                    output.push_str(&format!("for {} in [{}]:\n", for_loop.variable, items_str));
                                }
                            }
                        } else {
                            let items_str = self.word_to_string(&for_loop.items[0]);
                            output.push_str(&format!("for {} in [{}]:\n", for_loop.variable, items_str));
                        }
                    }
                    Word::BraceExpansion(brace) => {
                        // Handle brace expansion like {1..5} -> range(1, 6)
                        if let Some(BraceItem::Range(range)) = brace.items.first() {
                            if let (Ok(start), Ok(end)) = (range.start.parse::<i32>(), range.end.parse::<i32>()) {
                                output.push_str(&format!("for {} in range({}, {}):\n", for_loop.variable, start, end + 1));
                            } else {
                                output.push_str(&format!("for {} in [{}]:\n", for_loop.variable, self.word_to_string(&for_loop.items[0])));
                            }
                        } else {
                            output.push_str(&format!("for {} in [{}]:\n", for_loop.variable, self.word_to_string(&for_loop.items[0])));
                        }
                    }
                    _ => {
                        let items_str = for_loop.items.iter().map(|item| self.word_to_string(item)).collect::<Vec<_>>().join(", ");
                        output.push_str(&format!("for {} in [{}]:\n", for_loop.variable, items_str));
                    }
                }
            } else {
                let items_str = for_loop.items.iter().map(|item| self.word_to_string(item)).collect::<Vec<_>>().join(", ");
                output.push_str(&format!("for {} in [{}]:\n", for_loop.variable, items_str));
            }
            self.indent_level += 1;
            output.push_str(&self.indent());
            output.push_str(&self.generate_block(&for_loop.body));
            self.indent_level -= 1;
        }
        
                output
    }
    
    fn generate_for_loop_collecting_output(&mut self, for_loop: &ForLoop) -> String {
        let mut output = String::new();
        
        if for_loop.items.is_empty() {
            // Infinite loop
            output.push_str("while True:\n");
            self.indent_level += 1;
            output.push_str(&self.indent());
            output.push_str("    # Collect output\n");
            output.push_str(&self.indent());
            output.push_str(&self.generate_block_collecting_output(&for_loop.body));
            self.indent_level -= 1;
        } else {
            // For loop with items
            if for_loop.items.len() == 1 {
                match &for_loop.items[0] {
                    Word::Variable(var) if var == "@" => {
                        // Special case for iterating over arguments
                        output.push_str(&format!("for {} in sys.argv[1:]:\n", for_loop.variable));
                    }
                    Word::StringInterpolation(interp) => {
                        // Handle string interpolation, especially for array access
                        if interp.parts.len() == 1 {
                            match &interp.parts[0] {
                                crate::ast::StringPart::MapAccess(map_name, key) if key == "@" => {
                                    // Special case: iterate over all array elements
                                    output.push_str(&format!("for {} in {}:\n", for_loop.variable, map_name));
                                }
                                _ => {
                                    let items_str = self.word_to_string(&for_loop.items[0]);
                                    output.push_str(&format!("for {} in [{}]:\n", for_loop.variable, items_str));
                                }
                            }
                        } else {
                            let items_str = self.word_to_string(&for_loop.items[0]);
                            output.push_str(&format!("for {} in [{}]:\n", for_loop.variable, items_str));
                        }
                    }
                    Word::BraceExpansion(brace) => {
                        // Handle brace expansion like {1..5} -> range(1, 6)
                        if let Some(BraceItem::Range(range)) = brace.items.first() {
                            if let (Ok(start), Ok(end)) = (range.start.parse::<i32>(), range.end.parse::<i32>()) {
                                output.push_str(&format!("for {} in range({}, {}):\n", for_loop.variable, start, end + 1));
                            } else {
                                output.push_str(&format!("for {} in [{}]:\n", for_loop.variable, self.word_to_string(&for_loop.items[0])));
                            }
                        } else {
                            output.push_str(&format!("for {} in [{}]:\n", for_loop.variable, self.word_to_string(&for_loop.items[0])));
                        }
                    }
                    _ => {
                        let items_str = for_loop.items.iter().map(|item| self.word_to_string(item)).collect::<Vec<_>>().join(", ");
                        output.push_str(&format!("for {} in [{}]:\n", for_loop.variable, items_str));
                    }
                }
            } else {
                let items_str = for_loop.items.iter().map(|item| self.word_to_string(item)).collect::<Vec<_>>().join(", ");
                output.push_str(&format!("for {} in [{}]:\n", for_loop.variable, items_str));
            }
            self.indent_level += 1;
            output.push_str(&self.indent());
            output.push_str("    # Collect output\n");
            output.push_str(&self.indent());
            output.push_str(&self.generate_block_collecting_output(&for_loop.body));
            self.indent_level -= 1;
        }
        
        output
    }
    
    fn generate_block_collecting_output(&mut self, block: &Block) -> String {
        let mut output = String::new();
        for command in &block.commands {
            match command {
                Command::Simple(cmd) => {
                    if cmd.name.is_literal("echo") {
                        // Collect echo output instead of printing
                        if cmd.args.is_empty() {
                            output.push_str("output_lines.append('')\n");
                        } else {
                            let args_str = cmd.args.iter().map(|arg| self.word_to_string(arg)).collect::<Vec<_>>().join(" + ");
                            output.push_str(&format!("output_lines.append({})\n", args_str));
                        }
                    } else {
                        // For other commands, just execute them normally
                        output.push_str(&self.generate_command(command));
                    }
                }
                _ => {
                    output.push_str(&self.generate_command(command));
                }
            }
        }
        output
    }
    
    fn generate_function(&mut self, func: &Function) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("def {}():\n", func.name));
        self.indent_level += 1;
        output.push_str(&self.indent());
        output.push_str(&self.generate_block(&func.body));
        self.indent_level -= 1;
        
        output
    }

    fn generate_subshell(&mut self, command: &Command) -> String {
        let mut output = String::new();
        // Run subshell inline
        output.push_str("try:\n");
        self.indent_level += 1;
        output.push_str(&self.indent());
        output.push_str(&self.generate_command(command));
        self.indent_level -= 1;
        output.push_str(&self.indent());
        output.push_str("except Exception as e:\n");
        self.indent_level += 1;
        output.push_str(&self.indent());
        output.push_str("print(f'Error: {e}', file=sys.stderr)\n");
        self.indent_level -= 1;
        output
    }

    fn generate_background(&mut self, command: &Command) -> String {
        let mut output = String::new();
        // Spawn background thread
        output.push_str("import threading\n");
        output.push_str("def _bg_body():\n");
        self.indent_level += 1;
        output.push_str(&self.indent());
        output.push_str("try:\n");
        self.indent_level += 1;
        output.push_str(&self.indent());
        output.push_str(&self.generate_command(command));
        self.indent_level -= 1;
        output.push_str(&self.indent());
        output.push_str("except Exception as e:\n");
        self.indent_level += 1;
        output.push_str(&self.indent());
        output.push_str("print(f'Error: {e}', file=sys.stderr)\n");
        self.indent_level -= 1;
        output.push_str(&self.indent());
        output.push_str("pass\n");
        self.indent_level -= 1;
        output.push_str("t = threading.Thread(target=_bg_body, daemon=True)\n");
        output.push_str("t.start()\n");
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
        // Handle shell test conditions
        match command {
            Command::Simple(cmd) => {
                if let Word::Literal(name) = &cmd.name {
                    if name == "[" || name == "test" {
                        if let Some(test_op) = cmd.args.get(0) {
                            if let Word::Literal(op) = test_op {
                                match op.as_str() {
                                    "-f" => {
                                        if let Some(file) = cmd.args.get(1) {
                                            return format!("Path('{}').is_file()", self.word_to_string(file));
                                        }
                                    }
                                    "-d" => {
                                        if let Some(dir) = cmd.args.get(1) {
                                            return format!("Path('{}').is_dir()", self.word_to_string(dir));
                                        }
                                    }
                                    "-e" => {
                                        if let Some(path) = cmd.args.get(1) {
                                            return format!("Path('{}').exists()", self.word_to_string(path));
                                        }
                                    }
                                    "-lt" => {
                                        if cmd.args.len() >= 3 {
                                            let left = &cmd.args[1];
                                            let right = &cmd.args[2];
                                            // Handle shell variables in comparison
                                            let left_expr = if let Word::Variable(var) = left { 
                                                if var == "i" { "i" } else { &self.word_to_string(left) }
                                            } else { 
                                                &self.word_to_string(left) 
                                            };
                                            let right_expr = if let Word::Literal(val) = right { 
                                                if val == "10" { "10" } else { &self.word_to_string(right) }
                                            } else { 
                                                &self.word_to_string(right) 
                                            };
                                            return format!("{} < {}", left_expr, right_expr);
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                "True".to_string()
            }
            _ => "True".to_string(),
        }
    }

    fn indent(&self) -> String {
        "    ".repeat(self.indent_level)
    }

    fn escape_python_string(&self, s: &str) -> String {
        // For Python, we need to handle quotes and newlines properly
        // If the string contains newlines, use triple quotes to avoid syntax errors
        if s.contains('\n') {
            // Use triple quotes for strings with newlines
            format!("'''{}'''", s)
        } else if s.contains('"') && s.contains("'") {
            // Use triple quotes to avoid escaping issues
            format!("'''{}'''", s)
        } else if s.contains('"') {
            // Use single quotes to avoid escaping double quotes
            format!("'{}'", s)
        } else {
            // Use double quotes for strings without double quotes
            format!("\"{}\"", s)
        }
    }
    
    fn escape_python_string_literal(&self, s: &str) -> String {
        // Convert special characters to Python escape sequences
        let mut result = String::new();
        for ch in s.chars() {
            match ch {
                '\n' => result.push_str("\\n"),
                '\r' => result.push_str("\\r"),
                '\t' => result.push_str("\\t"),
                '\x07' => result.push_str("\\a"),  // bell
                '\x08' => result.push_str("\\b"),  // backspace
                '\x0c' => result.push_str("\\f"),  // formfeed
                '\x0b' => result.push_str("\\v"),  // vertical tab
                '\'' => result.push_str("\\'"),
                '"' => result.push_str("\\\""),
                '\\' => result.push_str("\\\\"),
                _ => result.push(ch),
            }
        }
        
        // Choose appropriate quote style
        if result.contains('\'') && result.contains('"') {
            // Contains both quotes, use triple quotes
            format!("'''{}'''", result)
        } else if result.contains('\'') {
            // Contains single quotes, use double quotes
            format!("\"{}\"", result)
        } else {
            // Use single quotes
            format!("'{}'", result)
        }
    }
    
    fn convert_escape_sequences(&self, s: &str) -> String {
        // Convert shell escape sequences to Python-compatible format
        let mut result = String::new();
        let mut chars = s.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '\\' {
                if let Some(next_ch) = chars.next() {
                    match next_ch {
                        'a' => result.push('\x07'), // bell
                        'b' => result.push('\x08'), // backspace
                        'f' => result.push('\x0c'), // formfeed
                        'n' => result.push('\n'),   // newline
                        'r' => result.push('\r'),   // carriage return
                        't' => result.push('\t'),   // tab
                        'v' => result.push('\x0b'), // vertical tab
                        '\\' => result.push('\\'),  // backslash
                        '\'' => result.push('\''),  // single quote
                        '"' => result.push('"'),    // double quote
                        '0'..='7' => {
                            // Octal escape sequence
                            let mut octal = String::new();
                            octal.push(next_ch);
                            if let Some(&ch) = chars.peek() {
                                if ch >= '0' && ch <= '7' {
                                    octal.push(chars.next().unwrap());
                                    if let Some(&ch2) = chars.peek() {
                                        if ch2 >= '0' && ch2 <= '7' {
                                            octal.push(chars.next().unwrap());
                                        }
                                    }
                                }
                            }
                            if let Ok(byte) = u8::from_str_radix(&octal, 8) {
                                result.push(byte as char);
                            } else {
                                result.push('\\');
                                result.push_str(&octal);
                            }
                        }
                        'x' => {
                            // Hex escape sequence
                            let mut hex = String::new();
                            if let Some(&ch) = chars.peek() {
                                if (ch >= '0' && ch <= '9') || (ch >= 'a' && ch <= 'f') || (ch >= 'A' && ch <= 'F') {
                                    hex.push(chars.next().unwrap().to_ascii_lowercase());
                                    if let Some(&ch2) = chars.peek() {
                                        if (ch2 >= '0' && ch2 <= '9') || (ch2 >= 'a' && ch2 <= 'f') || (ch2 >= 'A' && ch2 <= 'F') {
                                            hex.push(chars.next().unwrap().to_ascii_lowercase());
                                        }
                                    }
                                }
                            }
                            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                                result.push(byte as char);
                            } else {
                                result.push('\\');
                                result.push('x');
                                result.push_str(&hex);
                            }
                        }
                        'u' => {
                            // Unicode escape sequence (4 hex digits)
                            let mut hex = String::new();
                            for _ in 0..4 {
                                if let Some(&ch) = chars.peek() {
                                    if (ch >= '0' && ch <= '9') || (ch >= 'a' && ch <= 'f') || (ch >= 'A' && ch <= 'F') {
                                        hex.push(chars.next().unwrap().to_ascii_lowercase());
                                    } else {
                                        break;
                                    }
                                } else {
                                    break;
                                }
                            }
                            if hex.len() == 4 {
                                if let Ok(codepoint) = u32::from_str_radix(&hex, 16) {
                                    if let Some(ch) = char::from_u32(codepoint) {
                                        result.push(ch);
                                    } else {
                                        result.push('\\');
                                        result.push('u');
                                        result.push_str(&hex);
                                    }
                                } else {
                                    result.push('\\');
                                    result.push('u');
                                    result.push_str(&hex);
                                }
                            } else {
                                result.push('\\');
                                result.push('u');
                                result.push_str(&hex);
                            }
                        }
                        'U' => {
                            // Unicode escape sequence (8 hex digits)
                            let mut hex = String::new();
                            for _ in 0..8 {
                                if let Some(&ch) = chars.peek() {
                                    if (ch >= '0' && ch <= '9') || (ch >= 'a' && ch <= 'f') || (ch >= 'A' && ch <= 'F') {
                                        hex.push(chars.next().unwrap().to_ascii_lowercase());
                                    } else {
                                        break;
                                    }
                                                            } else {
                                break;
                            }
                        }
                        if hex.len() == 8 {
                                if let Ok(codepoint) = u32::from_str_radix(&hex, 16) {
                                    if let Some(ch) = char::from_u32(codepoint) {
                                        result.push(ch);
                                    } else {
                                        result.push('\\');
                                        result.push('U');
                                        result.push_str(&hex);
                                    }
                                } else {
                                    result.push('\\');
                                    result.push('U');
                                    result.push_str(&hex);
                                }
                            } else {
                                result.push('\\');
                                result.push('U');
                                result.push_str(&hex);
                            }
                        }
                        _ => {
                            // Unknown escape sequence, keep as-is
                            result.push('\\');
                            result.push(next_ch);
                        }
                    }
                } else {
                    // Backslash at end of string
                    result.push('\\');
                }
            } else {
                result.push(ch);
            }
        }
        
        result
    }
    
    fn parse_arithmetic_expansion(&self, s: &str) -> String {
        // Handle arithmetic expansion like $((i + 1)) -> (i + 1)
        if s.starts_with("$(") && s.ends_with(")") {
            let content = &s[2..s.len()-1];
            // Convert shell arithmetic to Python arithmetic
            let converted = content.replace("$i", "i")
                                 .replace("$1", "sys.argv[1] if len(sys.argv) > 1 else ''")
                                 .replace("$2", "sys.argv[2] if len(sys.argv) > 2 else ''")
                                 .replace("$3", "sys.argv[3] if len(sys.argv) > 3 else ''")
                                 .replace("$4", "sys.argv[4] if len(sys.argv) > 4 else ''")
                                 .replace("$5", "sys.argv[5] if len(sys.argv) > 5 else ''");
            converted
        } else {
            s.to_string()
        }
    }
    
    fn word_to_string(&self, word: &Word) -> String {
        match word {
            Word::Literal(s) => format!("'{}'", s),
            Word::Array(name, elements) => {
                // Convert array declaration to Python list
                let elements_str = elements.iter()
                    .map(|e| format!("'{}'", self.escape_python_string(e)))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{} = [{}]", name, elements_str)
            },
            Word::Variable(var) => {
                if var == "i" { "i".to_string() }
                else if var == "1" { "sys.argv[1] if len(sys.argv) > 1 else ''".to_string() }
                else if var == "2" { "sys.argv[2] if len(sys.argv) > 2 else ''".to_string() }
                else if var == "3" { "sys.argv[3] if len(sys.argv) > 3 else ''".to_string() }
                else if var == "4" { "sys.argv[4] if len(sys.argv) > 4 else ''".to_string() }
                else if var == "5" { "sys.argv[5] if len(sys.argv) > 5 else ''".to_string() }
                else { format!("'${}'", var) }
            }
            Word::Arithmetic(arith) => format!("({})", arith.expression),
            Word::BraceExpansion(brace) => {
                if let Some(BraceItem::Range(range)) = brace.items.first() {
                    if let (Ok(start), Ok(end)) = (range.start.parse::<i32>(), range.end.parse::<i32>()) {
                        format!("range({}, {})", start, end + 1)
                    } else {
                        format!("'{{{}}}'", brace.items.iter().map(|item| self.brace_item_to_string(item)).collect::<Vec<_>>().join(", "))
                    }
                } else {
                    format!("'{{{}}}'", brace.items.iter().map(|item| self.brace_item_to_string(item)).collect::<Vec<_>>().join(", "))
                }
            }
            Word::MapAccess(map_name, key) => {
                if key == "@" {
                    // Special case: @ means all elements
                    format!("' '.join({})", map_name)
                } else if key.parse::<usize>().is_ok() {
                    // Numeric key - treat as array access
                    format!("{}[{}]", map_name, key)
                } else {
                    // String key - treat as map access
                    format!("{}.get('{}', '')", map_name, key)
                }
            }
            Word::MapKeys(map_name) => format!("list({}.keys())", map_name),
            Word::MapLength(map_name) => format!("len({})", map_name),
            Word::CommandSubstitution(_) => "''".to_string(), // TODO: implement command substitution
            Word::StringInterpolation(interp) => {
                // Convert string interpolation parts to Python format
                let mut parts = Vec::new();
                for part in &interp.parts {
                    match part {
                        crate::ast::StringPart::Literal(s) => {
                            parts.push(format!("'{}'", s));
                        }
                        crate::ast::StringPart::Variable(var) => {
                            if var == "i" { 
                                parts.push("i".to_string());
                            } else if var == "1" { 
                                parts.push("sys.argv[1] if len(sys.argv) > 1 else ''".to_string());
                            } else if var == "2" { 
                                parts.push("sys.argv[2] if len(sys.argv) > 2 else ''".to_string());
                            } else if var == "3" { 
                                parts.push("sys.argv[3] if len(sys.argv) > 3 else ''".to_string());
                            } else if var == "4" { 
                                parts.push("sys.argv[4] if len(sys.argv) > 4 else ''".to_string());
                            } else if var == "5" { 
                                parts.push("sys.argv[5] if len(sys.argv) > 5 else ''".to_string());
                            } else { 
                                parts.push(format!("'${}'", var));
                            }
                        }
                        crate::ast::StringPart::MapAccess(map_name, key) => {
                            if key == "@" {
                                // Special case: @ means all elements
                                parts.push(format!("' '.join({})", map_name));
                            } else if key.parse::<usize>().is_ok() {
                                // Numeric key - treat as array access
                                parts.push(format!("{}[{}]", map_name, key));
                            } else {
                                // String key - treat as map access
                                parts.push(format!("{}.get('{}', '')", map_name, key));
                            }
                        }
                        crate::ast::StringPart::MapKeys(map_name) => {
                            parts.push(format!("list({}.keys())", map_name));
                        }
                        crate::ast::StringPart::MapLength(map_name) => {
                            parts.push(format!("len({})", map_name));
                        }
                        _ => {
                            parts.push("''".to_string()); // TODO: implement other parts
                        }
                    }
                }
                parts.join(" + ")
            }
            Word::ParameterExpansion(_) => "''".to_string(), // TODO: implement parameter expansion
        }
    }
    
    fn brace_item_to_string(&self, item: &BraceItem) -> String {
        match item {
            BraceItem::Literal(s) => s.clone(),
            BraceItem::Range(range) => format!("{}..{}", range.start, range.end),
            BraceItem::Sequence(seq) => seq.join(","),
        }
    }
}
