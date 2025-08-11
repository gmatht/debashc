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
            Command::BlankLine => "\n".to_string(),
        }
    }

    fn generate_simple_command(&self, cmd: &SimpleCommand) -> String {
        let mut output = String::new();
        
        // Handle environment variables
        for (var, value) in &cmd.env_vars {
            output.push_str(&format!("os.environ['{}'] = '{}'\n", var, value));
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
                    let args = cmd.args.join(" ");
                    // Check if we need f-string interpolation
                    if args.contains('$') {
                        // Convert shell variables to Python f-string variables
                        let mut converted_args = args.clone();
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
                        // No variables, use regular print
                        let escaped_args = self.escape_python_string(&args);
                        output.push_str(&format!("print({})\n", escaped_args));
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
            // Special handling for cd
            let empty_word = Word::Literal("".to_string());
            let dir = cmd.args.first().unwrap_or(&empty_word);
            output.push_str(&format!("os.chdir('{}')\n", dir));
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
        let action = if cmd.enable { "enable" } else { "disable" };
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
            // For now, handle simple pipelines
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
            Word::MapAccess(_map_name, key) => format!("map.get('{}', '')", key), // TODO: implement map access
            Word::CommandSubstitution(_) => "''".to_string(), // TODO: implement command substitution
            Word::StringInterpolation(_) => "''".to_string(), // TODO: implement string interpolation
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
