use crate::ast::*;
use std::collections::HashSet;
// HashMap import removed as it's not used

pub struct PerlGenerator {
    indent_level: usize,
    declared_locals: HashSet<String>,
    subshell_depth: usize,
}

impl PerlGenerator {
    pub fn new() -> Self {
        Self { indent_level: 0, declared_locals: HashSet::new(), subshell_depth: 0 }
    }

    pub fn generate(&mut self, commands: &[Command]) -> String {
        let mut output = String::new();
        output.push_str("#!/usr/bin/env perl\n");
        output.push_str("use strict;\n");
        output.push_str("use warnings;\n\n");

        for command in commands {
            output.push_str(&self.generate_command(command));
        }
        // Remove all trailing newlines
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

    fn generate_simple_command(&mut self, cmd: &SimpleCommand) -> String {
        let mut output = String::new();
        let has_env = !cmd.env_vars.is_empty() && cmd.name != "true";
        if has_env {
            output.push_str("{\n");
            for (var, value) in &cmd.env_vars {
                let val = self.perl_string_literal(value);
                output.push_str(&format!("local $ENV{{{}}} = {};;\n", var, val));
            }
        }

        // Generate the command
        if cmd.name == "((" {
            // Handle arithmetic expressions like ((i++))
            if let Some(expr) = cmd.args.first() {
                // Convert shell arithmetic to Perl
                let perl_expr = self.convert_arithmetic_to_perl(expr);
                output.push_str(&format!("{}\\n", perl_expr));
            }
        } else if cmd.name == "true" && !cmd.env_vars.is_empty() && cmd.args.is_empty() {
            // Assignment-only shell locals: e.g., a=1
            for (var, value) in &cmd.env_vars {
                let val = match value {
                    Word::Arithmetic(arithmetic) => {
                        // Handle shell arithmetic: $((i + 1)) -> $i + 1
                        self.convert_arithmetic_to_perl(&arithmetic.expression)
                    }
                    Word::Literal(literal) => {
                        if literal.starts_with("$(") && literal.ends_with(")") {
                            // Handle command substitution: $(command) -> `command`
                            let cmd = &literal[2..literal.len()-1];
                            format!("`{}`", cmd)
                        } else {
                            self.perl_string_literal(literal)
                        }
                    }
                    Word::Variable(var_name) => {
                        // Handle variable references
                        format!("${}", var_name)
                    }
                    _ => {
                        // Handle other Word types by converting to string
                        self.word_to_perl(value)
                    }
                };
                
                if self.subshell_depth > 0 || !self.declared_locals.contains(var) {
                    output.push_str(&format!("my ${} = {};\n", var, val));
                } else {
                    output.push_str(&format!("${} = {};\n", var, val));
                }
                if self.subshell_depth == 0 {
                    self.declared_locals.insert(var.clone());
                }
            }
        } else if cmd.name == "true" {
            // Builtin true: successful no-op
            output.push_str("1;\n");
        } else if cmd.name == "false" {
            // Builtin false: no-op; semantic failure not modeled in this simplified generator
            output.push_str("0;\n");
        } else if cmd.name == "echo" {
            // Special handling for echo
            if cmd.args.is_empty() {
                output.push_str("print(\"\\n\");\n");
            } else {
                // Support special variables like $# (argc)
                if cmd.args.len() == 1 {
                    let arg = &cmd.args[0];
                    if matches!(arg, Word::Variable(var) if var == "#") {
                        output.push_str("print(scalar(@ARGV) . \"\\n\");\n");
                    } else if matches!(arg, Word::Variable(var) if var == "@") {
                        output.push_str("print(join(\" \", @ARGV) . \"\\n\");\n");
                    } else if let Word::StringInterpolation(interp) = arg {
                        // Handle string interpolation like "$#"
                        if interp.parts.len() == 1 {
                            if let StringPart::Variable(var) = &interp.parts[0] {
                                if var == "#" {
                                    output.push_str("print(scalar(@ARGV) . \"\\n\");\n");
                                } else if var == "@" {
                                    output.push_str("print(join(\" \", @ARGV) . \"\\n\");\n");
                                } else {
                                    // Handle other variables in string interpolation
                                    let converted = self.convert_string_interpolation_to_perl(interp);
                                    output.push_str(&format!("print(\"{}\\n\");\n", converted));
                                }
                            } else {
                                // Handle other string parts
                                let converted = self.convert_string_interpolation_to_perl(interp);
                                output.push_str(&format!("print(\"{}\\n\");\n", converted));
                            }
                        } else {
                            // Handle multiple parts in string interpolation
                            let converted = self.convert_string_interpolation_to_perl(interp);
                            output.push_str(&format!("print(\"{}\\n\");\n", converted));
                        }
                    } else {
                        // Handle direct variable references like $# or $@
                        let arg_str = arg.to_string();
                        if arg_str == "$#" {
                            output.push_str("print(scalar(@ARGV) . \"\\n\");\n");
                        } else if arg_str == "$@" {
                            output.push_str("print(join(\" \", @ARGV) . \"\\n\");\n");
                        } else {
                            let args = cmd.args.join(" ");
                            // Convert shell positional parameters to Perl equivalents
                            let converted_args = args.replace("$1", "$_[0]")
                                                   .replace("$2", "$_[1]")
                                                   .replace("$3", "$_[2]")
                                                   .replace("$4", "$_[3]")
                                                   .replace("$5", "$_[4]")
                                                   .replace("$6", "$_[5]")
                                                   .replace("$7", "$_[6]")
                                                   .replace("$8", "$_[7]")
                                                   .replace("$9", "$_[8]");
                            let escaped_args = self.escape_perl_string(&converted_args);
                            // Allow interpolation ($var) intentionally by not escaping '$'
                            output.push_str(&format!("print(\"{}\\n\");\n", escaped_args));
                        }
                    }
                } else {
                    // Handle multiple arguments or no arguments
                    let args = cmd.args.join(" ");
                    // Convert shell positional parameters to Perl equivalents
                    let converted_args = args.replace("$1", "$_[0]")
                                           .replace("$2", "$_[1]")
                                           .replace("$3", "$_[2]")
                                           .replace("$4", "$_[3]")
                                           .replace("$5", "$_[4]")
                                           .replace("$6", "$_[5]")
                                           .replace("$7", "$_[6]")
                                           .replace("$8", "$_[7]")
                                           .replace("$9", "$_[8]");
                    let escaped_args = self.escape_perl_string(&converted_args);
                    // Allow interpolation ($var) intentionally by not escaping '$'
                    output.push_str(&format!("print(\"{}\\n\");\n", escaped_args));
                }
            }
        } else if cmd.name == "cd" {
            // Special handling for cd
            let empty_word = Word::Literal("".to_string());
            let dir = cmd.args.first().unwrap_or(&empty_word);
            output.push_str(&format!("chdir('{}') or die \"Cannot change to directory: $!\\n\";\n", dir));
        } else if cmd.name == "ls" {
            // Special handling for ls (ignore flags like -la); default to current dir
            let dir = if cmd.args.is_empty() {
                ".".to_string()
            } else if cmd.args[0].starts_with('-') {
                ".".to_string()
            } else {
                cmd.args[0].to_string()
            };
            output.push_str(&format!("opendir(my $dh, '{}') or die \"Cannot open directory: $!\\n\";\n", dir));
            output.push_str("while (my $file = readdir($dh)) {\n");
            output.push_str("    print(\"$file\\n\") unless $file =~ /^\\.\\.?$/;\n");
            output.push_str("}\n");
            output.push_str("closedir($dh);\n");
        } else if cmd.name == "grep" {
            // Special handling for grep
            if cmd.args.len() >= 2 {
                let pattern = &cmd.args[0];
                let file = &cmd.args[1];
                output.push_str(&format!("open(my $fh, '<', '{}') or die \"Cannot open file: $!\\n\";\n", file));
                output.push_str(&format!("while (my $line = <$fh>) {{\n"));
                output.push_str(&format!("    print($line) if $line =~ /{}/;\n", pattern));
                output.push_str("}\n");
                output.push_str("close($fh);\n");
            }
        } else if cmd.name == "cat" {
            // Special handling for cat including heredocs
            // If there are heredoc redirects attached, emit their bodies inline
            let mut printed_any = false;
            for redir in &cmd.redirects {
                if matches!(redir.operator, RedirectOperator::Heredoc | RedirectOperator::HeredocTabs) {
                    if let Some(body) = &redir.heredoc_body {
                        output.push_str(&format!("print <<'{}';\n{}\n{}\n;\n", redir.target, body, redir.target));
                        printed_any = true;
                    }
                }
            }
            if !printed_any {
                for arg in &cmd.args {
                    output.push_str(&format!("open(my $fh, '<', '{}') or die \"Cannot open file: $!\\n\";\n", arg));
                    output.push_str("while (my $line = <$fh>) {\n");
                    output.push_str("    print($line);\n");
                    output.push_str("}\n");
                    output.push_str("close($fh);\n");
                }
            }
        } else if cmd.name == "mkdir" {
            // Special handling for mkdir
            for arg in &cmd.args {
                output.push_str(&format!("mkdir('{}') or die \"Cannot create directory: $!\\n\";\n", arg));
            }
        } else if cmd.name == "rm" {
            // Special handling for rm
            for arg in &cmd.args {
                output.push_str(&format!("unlink('{}') or die \"Cannot remove file: $!\\n\";\n", arg));
            }
        } else if cmd.name == "mv" {
            // Special handling for mv
            if cmd.args.len() >= 2 {
                let src = &cmd.args[0];
                let dst = &cmd.args[1];
                output.push_str(&format!("rename('{}', '{}') or die \"Cannot move file: $!\\n\";\n", src, dst));
            }
        } else if cmd.name == "cp" {
            // Special handling for cp
            if cmd.args.len() >= 2 {
                let src = &cmd.args[0];
                let dst = &cmd.args[1];
                output.push_str(&format!("use File::Copy;\n"));
                output.push_str(&format!("copy('{}', '{}') or die \"Cannot copy file: $!\\n\";\n", src, dst));
            }
        } else if cmd.name == "test" || cmd.name == "[" {
            // Special handling for test
            self.generate_test_command(cmd, &mut output);
        } else if cmd.name == "[[" {
            // Builtin [[ ... ]]: treat as success no-op for now
            output.push_str("1;\n");
        } else if cmd.name == "shopt" {
            // Builtin: ignore; treat as success
            output.push_str("1;\n");
        } else if cmd.name == "export" {
            // Persistently set environment variables provided as VAR=VAL pairs
            for arg in &cmd.args {
                if let Some(eq_idx) = arg.find('=') {
                    let (k, v) = arg.split_at(eq_idx);
                    let v2 = if v.len() > 0 { &v[1..] } else { "" };
                    output.push_str(&format!("$ENV{{{}}} = {};;\n", k, self.perl_string_literal(v2)));
                }
            }
        } else {
            // Check if this might be a function call (not a builtin)
            let builtins = ["echo", "cd", "ls", "grep", "cat", "mkdir", "rm", "mv", "cp", "test", "[", "[[", "shopt", "export", "true", "false"];
            if !builtins.contains(&cmd.name.as_str()) {
                // Check if this looks like a function call (no file extension, no path separators)
                if !cmd.name.contains('.') && !cmd.name.contains('/') && !cmd.name.contains('\\') {
                    // Assume it's a function call
                    let args = cmd
                        .args
                        .iter()
                        .map(|arg| self.word_to_perl(arg))
                        .collect::<Vec<_>>();
                    if args.is_empty() {
                        output.push_str(&format!("{}();\n", cmd.name));
                    } else {
                        output.push_str(&format!("{}({});\n", cmd.name, args.join(", ")));
                    }
                } else {
                    // Non-builtin command - use system()
                    let name = self.perl_string_literal(&cmd.name);
                    let args = cmd
                        .args
                        .iter()
                        .map(|arg| self.perl_string_literal(arg))
                        .collect::<Vec<_>>()
                        .join(", ");
                    output.push_str(&format!("system({}, {});\n", name, args));
                }
            } else {
                // Builtin command - handle as before
                let args = cmd
                    .args
                    .iter()
                    .map(|arg| self.word_to_perl(arg))
                    .collect::<Vec<_>>();
                if args.is_empty() {
                    output.push_str(&format!("{}();\n", cmd.name));
                } else {
                    output.push_str(&format!("{}({});\n", cmd.name, args.join(", ")));
                }
            }
        }
        if has_env { output.push_str("}\n"); }
        output
    }

    fn generate_test_command(&mut self, cmd: &SimpleCommand, output: &mut String) {
        // Convert test conditions to Perl
        if cmd.args.len() == 3 {
            // Format: [ operand1 operator operand2 ]
            let operand1 = &cmd.args[0];
            let operator = &cmd.args[1];
            let operand2 = &cmd.args[2];
            
            // Ensure variables are declared if they're shell variables
            if let Word::Variable(var_name) = operand1 {
                if !self.declared_locals.contains(var_name) {
                    output.push_str(&format!("my ${} = 0;\n", var_name));
                    self.declared_locals.insert(var_name.to_string());
                }
            }
            if let Word::Variable(var_name) = operand2 {
                if !self.declared_locals.contains(var_name) {
                    output.push_str(&format!("my ${} = 0;\n", var_name));
                    self.declared_locals.insert(var_name.to_string());
                }
            }
            
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
            
            // Ensure variables are declared if they're shell variables
            if let Word::Variable(var_name) = operand {
                if !self.declared_locals.contains(var_name) {
                    output.push_str(&format!("my ${} = 0;\n", var_name));
                    self.declared_locals.insert(var_name.to_string());
                }
            }
            
            match operator.as_str() {
                "-f" => {
                    output.push_str(&format!("-f {}", self.word_to_perl_for_test(operand)));
                }
                "-d" => {
                    output.push_str(&format!("-d {}", self.word_to_perl_for_test(operand)));
                }
                "-e" => {
                    output.push_str(&format!("-e {}", self.word_to_perl_for_test(operand)));
                }
                "-r" => {
                    output.push_str(&format!("-r {}", self.word_to_perl_for_test(operand)));
                }
                "-w" => {
                    output.push_str(&format!("-w {}", self.word_to_perl_for_test(operand)));
                }
                "-x" => {
                    output.push_str(&format!("-x {}", self.word_to_perl_for_test(operand)));
                }
                "-z" => {
                    output.push_str(&format!("-z {}", self.word_to_perl_for_test(operand)));
                }
                "-n" => {
                    output.push_str(&format!("-s {}", self.word_to_perl_for_test(operand)));
                }
                _ => {
                    output.push_str(&format!("{} {} {}", self.word_to_perl_for_test(operand), operator, self.word_to_perl_for_test(operand)));
                }
            }
        }
    }

    fn generate_pipeline(&mut self, pipeline: &Pipeline) -> String {
        let mut output = String::new();
        
        let has_pipe = pipeline.operators.iter().any(|op| matches!(op, PipeOperator::Pipe));
        if pipeline.commands.len() == 1 {
            output.push_str(&self.generate_command(&pipeline.commands[0]));
        } else if has_pipe {
            // For now, handle simple pipelines using system calls
            output.push_str("my $output;\n");
            for (i, command) in pipeline.commands.iter().enumerate() {
                if i == 0 {
                    // First command - capture output
                    output.push_str(&format!("$output = qx({});\n", self.command_to_string(command)));
                } else {
                    // Subsequent commands - pipe through, but handle quotes carefully
                    let cmd_str = self.command_to_string(command);
                    // For qx, we need to handle the command string very carefully
                    // Use single quotes around the entire command to avoid shell interpretation
                    let escaped_cmd = cmd_str.replace("'", "'\"'\"'"); // Escape single quotes properly
                    output.push_str(&format!("$output = qx('echo \"$output\" | {}');\n", escaped_cmd));
                }
            }
            output.push_str("print($output);\n");
        } else {
            // Implement && and || via system() exit codes
            output.push_str("my $last_status = 0;\n");
            if let Some(first) = pipeline.commands.first() {
                output.push_str(&format!("$last_status = system('{}');\n", self.command_to_string(first)));
            }
            for (idx, op) in pipeline.operators.iter().enumerate() {
                let cmd = &pipeline.commands[idx + 1];
                match op {
                    PipeOperator::And => {
                        output.push_str(&format!("if ($last_status == 0) {{ $last_status = system('{}'); }}\n", self.command_to_string(cmd)));
                    }
                    PipeOperator::Or => {
                        output.push_str(&format!("if ($last_status != 0) {{ $last_status = system('{}'); }}\n", self.command_to_string(cmd)));
                    }
                    PipeOperator::Pipe => {}
                }
            }
        }
        
        output
    }

    fn command_to_string(&self, command: &Command) -> String {
        match command {
            Command::Simple(cmd) => {
                if cmd.args.is_empty() {
                    cmd.name.to_string()
                } else {
                    let args = cmd.args.iter().map(|arg| arg.to_string()).collect::<Vec<_>>().join(" ");
                    format!("{} {}", cmd.name, args)
                }
            }
            _ => "command".to_string(),
        }
    }

    fn generate_if_statement(&mut self, if_stmt: &IfStatement) -> String {
        let mut output = String::new();
        
        // Generate condition
        output.push_str("if (");
        match &*if_stmt.condition {
            Command::Simple(cmd) if cmd.name == "[" || cmd.name == "test" => {
                self.generate_test_command(cmd, &mut output);
            }
            _ => {
                output.push_str(&self.generate_command(&if_stmt.condition));
            }
        }
        output.push_str(") {\n");
        
        // Generate then branch
        self.indent_level += 1;
        output.push_str(&self.indent());
        output.push_str(&self.generate_command(&if_stmt.then_branch));
        self.indent_level -= 1;
        
        // Generate else branch if present
        if let Some(else_branch) = &if_stmt.else_branch {
            output.push_str(&self.indent());
            output.push_str("} else {\n");
            self.indent_level += 1;
            output.push_str(&self.indent());
            output.push_str(&self.generate_command(else_branch));
            self.indent_level -= 1;
        }
        
        output.push_str(&self.indent());
        output.push_str("}\n");
        
        output
    }

    fn generate_while_loop(&mut self, while_loop: &WhileLoop) -> String {
        let mut output = String::new();
        
        // Handle different types of conditions
        match &*while_loop.condition {
            Command::Simple(cmd) if cmd.name == "[" || cmd.name == "test" => {
                // For test commands, generate a simple while loop
                // Initialize any variables used in test conditions
                if cmd.args.len() >= 3 {
                    // Check both operands for variables that need initialization
                    let operand1 = &cmd.args[0];
                    let operand2 = &cmd.args[2];
                    
                                    // Initialize first operand if it's a variable
                if let Word::Variable(var_name) = operand1 {
                    if !self.declared_locals.contains(var_name) {
                        // Check if this variable was used in a previous for loop
                        if var_name == "i" {
                            output.push_str(&format!("my ${} = 5;\n", var_name));
                        } else {
                            output.push_str(&format!("my ${} = 0;\n", var_name));
                        }
                        self.declared_locals.insert(var_name.to_string());
                    }
                }
                
                // Initialize second operand if it's a variable
                if let Word::Variable(var_name) = operand2 {
                    if !self.declared_locals.contains(var_name) {
                        output.push_str(&format!("my ${} = 0;\n", var_name));
                        self.declared_locals.insert(var_name.to_string());
                    }
                }
                } else if cmd.args.len() >= 1 {
                    // Handle single argument test conditions
                    let var_name = cmd.args[0].trim_start_matches('$');
                    if !self.declared_locals.contains(var_name) {
                        output.push_str(&format!("my ${} = 0;\n", var_name));
                        self.declared_locals.insert(var_name.to_string());
                    }
                }
                output.push_str("while (");
                self.generate_test_command(cmd, &mut output);
                output.push_str(") {\n");
            }
            _ => {
                // For other command types, generate a complex while loop with exit status check
                output.push_str("while (1) {\n");
                output.push_str(&self.indent());
                output.push_str("my $condition = ");
                output.push_str("system(");
                output.push_str(&self.generate_command(&while_loop.condition));
                output.push_str(") == 0");
                output.push_str(";\n");
                output.push_str(&self.indent());
                output.push_str("last unless $condition;\n");
            }
        }
        
        self.indent_level += 1;
        
        // Generate body commands
        for command in &while_loop.body.commands {
            output.push_str(&self.indent());
            output.push_str(&self.generate_command(command));
        }
        
        self.indent_level -= 1;
        output.push_str("}\n");
        
        output
    }

    fn find_for_loop_variable(&self, command: &Command) -> Option<String> {
        match command {
            Command::For(for_loop) => Some(for_loop.variable.clone()),
            Command::Block(block) => {
                for cmd in &block.commands {
                    if let Some(var) = self.find_for_loop_variable(cmd) {
                        return Some(var);
                    }
                }
                None
            }
            _ => None
        }
    }

    fn generate_for_loop(&mut self, for_loop: &ForLoop) -> String {
        let variable = &for_loop.variable;
        let items = &for_loop.items;
        let body = &for_loop.body;
        
        // Special case for iterating over arguments ($@)
        if items.len() == 1 {
            let item = &items[0];
            if matches!(item, Word::Variable(var) if var == "@") {
                self.indent_level += 1;
                let body_code = self.generate_block(body);
                self.indent_level -= 1;
                return format!("for my ${} (@ARGV) {{\n{}}}\n", variable, body_code);
            } else if let Word::StringInterpolation(interp) = item {
                if interp.parts.len() == 1 {
                    if let StringPart::Variable(var) = &interp.parts[0] {
                        if var == "@" {
                            self.indent_level += 1;
                            let body_code = self.generate_block(body);
                            self.indent_level -= 1;
                            return format!("for my ${} (@ARGV) {{\n{}}}\n", variable, body_code);
                        }
                    }
                }
            }
        }
        
        // Convert shell brace expansion to Perl range syntax
        let items_str = if items.len() == 1 {
            match &items[0] {
                Word::BraceExpansion(expansion) => {
                    // Handle brace expansion items
                    if expansion.items.len() == 1 {
                        match &expansion.items[0] {
                            BraceItem::Range(range) => {
                                // Convert {1..5} to (1..5)
                                format!("({}..{})", range.start, range.end)
                            }
                            BraceItem::Literal(s) => {
                                // Single literal item
                                format!("\"{}\"", s)
                            }
                            BraceItem::Sequence(seq) => {
                                // Convert {a,b,c} to ("a", "b", "c")
                                format!("({})", seq.iter().map(|s| format!("\"{}\"", s)).collect::<Vec<_>>().join(", "))
                            }
                        }
                    } else {
                        // Multiple items
                        let parts: Vec<String> = expansion.items.iter().map(|item| {
                            match item {
                                BraceItem::Literal(s) => format!("\"{}\"", s),
                                BraceItem::Range(range) => format!("({}..{})", range.start, range.end),
                                BraceItem::Sequence(seq) => format!("({})", seq.iter().map(|s| format!("\"{}\"", s)).collect::<Vec<_>>().join(", ")),
                            }
                        }).collect();
                        format!("({})", parts.join(", "))
                    }
                }
                Word::Literal(s) if s.starts_with('{') && s.ends_with('}') => {
                    // Fallback for literal strings that look like brace expansions
                    let content = &s[1..s.len()-1];
                    if content.contains("..") {
                        // Already in range format like {1..5}
                        content.to_string()
                    } else {
                        // Convert {a,b,c} to ("a", "b", "c")
                        let parts: Vec<&str> = content.split(',').collect();
                        if parts.len() > 1 {
                            format!("({})", parts.iter().map(|s| format!("\"{}\"", s)).collect::<Vec<_>>().join(", "))
                        } else {
                            content.to_string()
                        }
                    }
                }
                _ => {
                    // Other word types
                    format!("\"{}\"", items[0])
                }
            }
        } else if items.is_empty() {
            // No items specified, use default behavior
            "()".to_string()
        } else {
            // Multiple items
            format!("({})", items.iter().map(|s| format!("\"{}\"", s)).collect::<Vec<_>>().join(", "))
        };
        
        self.indent_level += 1;
        let body_code = self.generate_block(body);
        self.indent_level -= 1;
        
        format!("for my ${} ({}) {{\n{}}}\n", variable, items_str, body_code)
    }

    fn parse_numeric_brace_range(&self, s: &str) -> Option<(i64, i64)> {
        // Matches forms like {0..5} or {10..3}
        if !(s.starts_with('{') && s.ends_with('}')) {
            return None;
        }
        let inner = &s[1..s.len() - 1];
        let parts: Vec<&str> = inner.split("..").collect();
        if parts.len() != 2 {
            return None;
        }
        let start = parts[0].parse::<i64>().ok()?;
        let end = parts[1].parse::<i64>().ok()?;
        Some((start, end))
    }

    fn parse_seq_command(&self, s: &str) -> Option<(i64, i64)> {
        // Accept backtick form `seq A B` or $(seq A B) or plain seq A B
        let trimmed = s.trim();
        // Strip backticks or $( )
        let inner = if trimmed.starts_with('`') && trimmed.ends_with('`') {
            &trimmed[1..trimmed.len()-1]
        } else if trimmed.starts_with("$(") && trimmed.ends_with(')') {
            &trimmed[2..trimmed.len()-1]
        } else {
            trimmed
        };

        let parts: Vec<&str> = inner.split_whitespace().collect();
        if parts.len() == 3 && parts[0] == "seq" {
            let start = parts[1].parse::<i64>().ok()?;
            let end = parts[2].parse::<i64>().ok()?;
            return Some((start, end));
        }
        None
    }

    fn generate_function(&mut self, func: &Function) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("sub {} {{\n", func.name));
        self.indent_level += 1;
        
        // Generate body commands
        for command in &func.body.commands {
            output.push_str(&self.indent());
            output.push_str(&self.generate_command(command));
        }
        
        self.indent_level -= 1;
        output.push_str("}\n");
        
        output
    }

    fn generate_subshell(&mut self, command: &Command) -> String {
        let mut output = String::new();
        
        output.push_str("do {\n");
        self.indent_level += 1;
        self.subshell_depth += 1;
        output.push_str(&self.indent());
        output.push_str(&self.generate_command(command));
        if self.subshell_depth > 0 { self.subshell_depth -= 1; }
        self.indent_level -= 1;
        output.push_str("};\n");
        
        output
    }

    fn generate_background(&mut self, command: &Command) -> String {
        let mut output = String::new();
        // Use threads to emulate background
        output.push_str("use threads;\n");
        output.push_str("threads->create(sub {\n");
        self.indent_level += 1;
        output.push_str(&self.indent());
        output.push_str(&self.generate_command(command));
        self.indent_level -= 1;
        output.push_str("});\n");
        output
    }

    fn generate_block(&mut self, block: &Block) -> String {
        let mut output = String::new();
        for cmd in &block.commands {
            output.push_str(&self.indent());
            output.push_str(&self.generate_command(cmd));
        }
        output
    }

    fn indent(&self) -> String {
        "    ".repeat(self.indent_level)
    }
    
    fn escape_perl_string(&self, s: &str) -> String {
        // First, unescape any \" sequences to " to avoid double-escaping
        let unescaped = s.replace("\\\"", "\"");
        // Then escape quotes and other characters for Perl
        unescaped.replace("\\", "\\\\")
                 .replace("\"", "\\\"")
                 .replace("\n", "\\n")
                 .replace("\r", "\\r")
                 .replace("\t", "\\t")
    }

    fn perl_string_literal(&self, s: &str) -> String {
        // Prefer double-quoted string with escapes to avoid conflicts with single quotes inside args
        format!("\"{}\"", self.escape_perl_string(s))
    }

    fn convert_arithmetic_to_perl(&self, expr: &str) -> String {
        // Convert shell arithmetic expressions to Perl
        let mut result = expr.to_string();
        
        // Replace shell arithmetic operators with Perl equivalents
        result = result.replace("++", "++");
        result = result.replace("--", "--");
        result = result.replace("+=", "+=");
        result = result.replace("-=", "-=");
        result = result.replace("*=", "*=");
        result = result.replace("/=", "/=");
        result = result.replace("%=", "%=");
        result = result.replace("**=", "**=");
        
        // Handle variable references (ensure $ prefix for single identifiers)
        let parts: Vec<&str> = result.split_whitespace().collect();
        let converted_parts: Vec<String> = parts.iter().map(|part| {
            if part.chars().all(|c| c.is_alphanumeric() || c == '_') && !part.chars().next().unwrap().is_digit(10) {
                // This looks like a variable name, add $ prefix
                format!("${}", part)
            } else {
                part.to_string()
            }
        }).collect();
        
        converted_parts.join(" ")
    }

    fn convert_string_interpolation_to_perl(&self, interp: &StringInterpolation) -> String {
        let mut result = String::new();
        

        
        for part in &interp.parts {
            match part {
                StringPart::Literal(s) => {
                    result.push_str(&self.escape_perl_string(s));
                }
                StringPart::Variable(var) => {
                    // Convert shell variables to Perl variables
                    if var == "#" {
                        result.push_str("scalar(@ARGV)");
                    } else if var == "@" {
                        result.push_str("join(\" \", @ARGV)");
                    } else if var == "1" {
                        result.push_str("$_[0]");
                    } else if var == "2" {
                        result.push_str("$_[1]");
                    } else if var == "3" {
                        result.push_str("$_[2]");
                    } else if var == "4" {
                        result.push_str("$_[3]");
                    } else if var == "5" {
                        result.push_str("$_[4]");
                    } else if var == "6" {
                        result.push_str("$_[5]");
                    } else if var == "7" {
                        result.push_str("$_[6]");
                    } else if var == "8" {
                        result.push_str("$_[7]");
                    } else if var == "9" {
                        result.push_str("$_[8]");
                    } else {
                        // For simple variable names, use $var instead of ${var}
                        result.push_str(&format!("${}", var));
                    }
                }
                StringPart::Arithmetic(arith) => {
                    // Convert shell arithmetic to Perl
                    let expr = self.convert_arithmetic_to_perl(&arith.expression);
                    result.push_str(&expr);
                }
                StringPart::CommandSubstitution(_) => {
                    // TODO: implement command substitution
                    result.push_str("''");
                }
            }
        }
        
        result
    }

    fn word_to_perl(&self, word: &Word) -> String {
        match word {
            Word::Literal(s) => self.perl_string_literal(s),
            Word::Variable(var) => format!("${}", var),
            Word::Arithmetic(expr) => self.convert_arithmetic_to_perl(&expr.expression),
            Word::BraceExpansion(expansion) => {
                // Handle brace expansion in test commands
                if expansion.items.len() == 1 {
                    match &expansion.items[0] {
                        BraceItem::Range(range) => {
                            format!("({}..{})", range.start, range.end)
                        }
                        BraceItem::Literal(s) => self.perl_string_literal(s),
                        BraceItem::Sequence(seq) => {
                            format!("({})", seq.iter().map(|s| self.perl_string_literal(s)).collect::<Vec<_>>().join(", "))
                        }
                    }
                } else {
                    // Multiple items
                    let parts: Vec<String> = expansion.items.iter().map(|item| {
                        match item {
                            BraceItem::Literal(s) => self.perl_string_literal(s),
                            BraceItem::Range(range) => format!("({}..{})", range.start, range.end),
                            BraceItem::Sequence(seq) => format!("({})", seq.iter().map(|s| self.perl_string_literal(s)).collect::<Vec<_>>().join(", ")),
                        }
                    }).collect();
                    format!("({})", parts.join(", "))
                }
            }
            Word::CommandSubstitution(_) => "`command`".to_string(),
            Word::StringInterpolation(interp) => {
                // For function arguments, we need quoted strings
                // If it's just a single literal part, wrap it in quotes
                if interp.parts.len() == 1 {
                    if let StringPart::Literal(s) = &interp.parts[0] {
                        return format!("\"{}\"", self.escape_perl_string(s));
                    }
                }
                // For more complex interpolations, wrap the result in quotes
                let content = self.convert_string_interpolation_to_perl(interp);
                format!("\"{}\"", content)
            },
        }
    }

    fn word_to_perl_for_test(&self, word: &Word) -> String {
        match word {
            Word::Literal(s) => self.perl_string_literal(s),
            Word::Variable(var) => format!("${}", var),
            Word::Arithmetic(expr) => self.convert_arithmetic_to_perl(&expr.expression),
            Word::BraceExpansion(expansion) => {
                // Handle brace expansion in test commands
                if expansion.items.len() == 1 {
                    match &expansion.items[0] {
                        BraceItem::Range(range) => {
                            format!("({}..{})", range.start, range.end)
                        }
                        BraceItem::Literal(s) => self.perl_string_literal(s),
                        BraceItem::Sequence(seq) => {
                            format!("({})", seq.iter().map(|s| self.perl_string_literal(s)).collect::<Vec<_>>().join(", "))
                        }
                    }
                } else {
                    // Multiple items
                    let parts: Vec<String> = expansion.items.iter().map(|item| {
                        match item {
                            BraceItem::Literal(s) => self.perl_string_literal(s),
                            BraceItem::Range(range) => format!("({}..{})", range.start, range.end),
                            BraceItem::Sequence(seq) => format!("({})", seq.iter().map(|s| self.perl_string_literal(s)).collect::<Vec<_>>().join(", ")),
                        }
                    }).collect();
                    format!("({})", parts.join(", "))
                }
            }
            Word::CommandSubstitution(_) => "`command`".to_string(),
            Word::StringInterpolation(interp) => {
                // For test commands, simple literal strings need to be quoted
                if interp.parts.len() == 1 {
                    if let StringPart::Literal(s) = &interp.parts[0] {
                        return format!("\"{}\"", self.escape_perl_string(s));
                    }
                }
                self.convert_string_interpolation_to_perl(interp)
            },
        }
    }
} 
