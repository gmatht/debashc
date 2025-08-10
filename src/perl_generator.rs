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
        if cmd.name == "true" && !cmd.env_vars.is_empty() && cmd.args.is_empty() {
            // Assignment-only shell locals: e.g., a=1
            for (var, value) in &cmd.env_vars {
                let val = self.perl_string_literal(value);
                if self.subshell_depth > 0 || !self.declared_locals.contains(var) {
                    output.push_str(&format!("my ${} = {};;\n", var, val));
                } else {
                    output.push_str(&format!("${} = {};;\n", var, val));
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
                if cmd.args.len() == 1 && cmd.args[0] == "$#" {
                    output.push_str("print(scalar(@ARGV) . \"\\n\");\n");
                } else {
                    let args = cmd.args.join(" ");
                    let escaped_args = self.escape_perl_string(&args);
                    // Allow interpolation ($var) intentionally by not escaping '$'
                    output.push_str(&format!("print(\"{}\\n\");\n", escaped_args));
                }
            }
        } else if cmd.name == "cd" {
            // Special handling for cd
            let empty_string = "".to_string();
            let dir = cmd.args.first().unwrap_or(&empty_string);
            output.push_str(&format!("chdir('{}') or die \"Cannot change to directory: $!\\n\";\n", dir));
        } else if cmd.name == "ls" {
            // Special handling for ls (ignore flags like -la); default to current dir
            let dir = if cmd.args.is_empty() {
                ".".to_string()
            } else if cmd.args[0].starts_with('-') {
                ".".to_string()
            } else {
                cmd.args[0].clone()
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
            // Special handling for cat
            for arg in &cmd.args {
                output.push_str(&format!("open(my $fh, '<', '{}') or die \"Cannot open file: $!\\n\";\n", arg));
                output.push_str("while (my $line = <$fh>) {\n");
                output.push_str("    print($line);\n");
                output.push_str("}\n");
                output.push_str("close($fh);\n");
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
            // Generic command execution with proper escaping
            let name = self.perl_string_literal(&cmd.name);
            let args = cmd
                .args
                .iter()
                .map(|arg| self.perl_string_literal(arg))
                .collect::<Vec<_>>()
                .join(", ");
            output.push_str(&format!("system({}, {});\n", name, args));
        }
        if has_env { output.push_str("}\n"); }
        output
    }

    fn generate_test_command(&self, cmd: &SimpleCommand, output: &mut String) {
        // Convert test conditions to Perl
        if cmd.args.len() >= 2 {
            let operator = &cmd.args[0];
            let operand = &cmd.args[1];
            
            match operator.as_str() {
                "-f" => {
                    output.push_str(&format!("if (-f '{}') {{\n", operand));
                }
                "-d" => {
                    output.push_str(&format!("if (-d '{}') {{\n", operand));
                }
                "-e" => {
                    output.push_str(&format!("if (-e '{}') {{\n", operand));
                }
                "-r" => {
                    output.push_str(&format!("if (-r '{}') {{\n", operand));
                }
                "-w" => {
                    output.push_str(&format!("if (-w '{}') {{\n", operand));
                }
                "-x" => {
                    output.push_str(&format!("if (-x '{}') {{\n", operand));
                }
                "-z" => {
                    output.push_str(&format!("if (-z '{}') {{\n", operand));
                }
                "-n" => {
                    output.push_str(&format!("if (-s '{}') {{\n", operand));
                }
                _ => {
                    output.push_str(&format!("if ('{}' {} '{}') {{\n", operand, operator, operand));
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
            // For now, handle simple pipelines
            output.push_str("my $output;\n");
            for (i, command) in pipeline.commands.iter().enumerate() {
                if i == 0 {
                    output.push_str(&format!("$output = `{}`;\n", self.command_to_string(command)));
                } else {
                    output.push_str(&format!("$output = `echo \"$output\" | {}`;\n", self.command_to_string(command)));
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
        output.push_str(&self.generate_command(&if_stmt.condition));
        
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
        
        output.push_str("while (1) {\n");
        self.indent_level += 1;
        
        // Generate condition check
        output.push_str(&self.indent());
        output.push_str("my $condition = ");
        output.push_str(&self.generate_command(&while_loop.condition));
        output.push_str(&self.indent());
        output.push_str("last unless $condition;\n");
        
        // Generate body
        output.push_str(&self.indent());
        output.push_str(&self.generate_command(&while_loop.body));
        
        self.indent_level -= 1;
        output.push_str("}\n");
        
        output
    }

    fn generate_for_loop(&mut self, for_loop: &ForLoop) -> String {
        let mut output = String::new();
        
        if for_loop.items.is_empty() {
            // For loop with no items (infinite loop)
            output.push_str("while (1) {\n");
            self.indent_level += 1;
            output.push_str(&self.indent());
            output.push_str(&self.generate_command(&for_loop.body));
            self.indent_level -= 1;
            output.push_str("}\n");
        } else {
            // For loop with items
            // Special-case numeric brace range like {0..5}
            if for_loop.items.len() == 1 {
                let first = &for_loop.items[0];
                // Special-case "$@" to iterate over @ARGV
                if first == "$@" || first == "${@}" {
                    output.push_str(&format!(
                        "foreach my ${} (@ARGV) {{\n",
                        for_loop.variable
                    ));
                    self.indent_level += 1;
                    output.push_str(&self.indent());
                    output.push_str(&self.generate_command(&for_loop.body));
                    self.indent_level -= 1;
                    output.push_str("}\n");
                    return output;
                }
                if let Some((start, end)) = self.parse_numeric_brace_range(first) {
                    output.push_str(&format!(
                        "foreach my ${} ({}..{}) {{\n",
                        for_loop.variable, start, end
                    ));
                    self.indent_level += 1;
                    output.push_str(&self.indent());
                    output.push_str(&self.generate_command(&for_loop.body));
                    self.indent_level -= 1;
                    output.push_str("}\n");
                    return output;
                } else if let Some((start, end)) = self.parse_seq_command(first) {
                    output.push_str(&format!(
                        "foreach my ${} ({}..{}) {{\n",
                        for_loop.variable, start, end
                    ));
                    self.indent_level += 1;
                    output.push_str(&self.indent());
                    output.push_str(&self.generate_command(&for_loop.body));
                    self.indent_level -= 1;
                    output.push_str("}\n");
                    return output;
                }
            }

            output.push_str(&format!(
                "foreach my ${} (qw({})) {{\n",
                for_loop.variable,
                for_loop.items.join(" ")
            ));
            self.indent_level += 1;
            output.push_str(&self.indent());
            output.push_str(&self.generate_command(&for_loop.body));
            self.indent_level -= 1;
            output.push_str("}\n");
        }
        
        output
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
        output.push_str(&self.indent());
        output.push_str(&self.generate_command(&func.body));
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
} 