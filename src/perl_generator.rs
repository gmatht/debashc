use crate::ast::*;
// HashMap import removed as it's not used

pub struct PerlGenerator {
    indent_level: usize,
}

impl PerlGenerator {
    pub fn new() -> Self {
        Self { indent_level: 0 }
    }

    pub fn generate(&mut self, commands: &[Command]) -> String {
        let mut output = String::new();
        output.push_str("#!/usr/bin/env perl\n");
        output.push_str("use strict;\n");
        output.push_str("use warnings;\n\n");

        for command in commands {
            output.push_str(&self.generate_command(command));
        }

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
        }
    }

    fn generate_simple_command(&self, cmd: &SimpleCommand) -> String {
        let mut output = String::new();
        
        // Handle environment variables
        for (var, value) in &cmd.env_vars {
            output.push_str(&format!("$ENV{{{}}} = '{}';\n", var, value));
        }

        // Generate the command
        if cmd.name == "echo" {
            // Special handling for echo
            if cmd.args.is_empty() {
                output.push_str("print(\"\\n\");\n");
            } else {
                let args = cmd.args.join(" ");
                let escaped_args = self.escape_perl_string(&args);
                output.push_str(&format!("print(\"{}\\n\");\n", escaped_args));
            }
        } else if cmd.name == "cd" {
            // Special handling for cd
            let empty_string = "".to_string();
            let dir = cmd.args.first().unwrap_or(&empty_string);
            output.push_str(&format!("chdir('{}') or die \"Cannot change to directory: $!\\n\";\n", dir));
        } else if cmd.name == "ls" {
            // Special handling for ls
            let args = if cmd.args.is_empty() { "." } else { &cmd.args[0] };
            output.push_str(&format!("opendir(my $dh, '{}') or die \"Cannot open directory: $!\\n\";\n", args));
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
        } else {
            // Generic command execution
            let args = cmd.args.iter().map(|arg| format!("'{}'", arg)).collect::<Vec<_>>().join(", ");
            output.push_str(&format!("system('{}', {});\n", cmd.name, args));
        }

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
        
        if pipeline.commands.len() == 1 {
            output.push_str(&self.generate_command(&pipeline.commands[0]));
        } else {
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
            output.push_str(&format!("foreach my ${} (qw({})) {{\n", 
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
        
        output.push_str("my $result = do {\n");
        self.indent_level += 1;
        output.push_str(&self.indent());
        output.push_str(&self.generate_command(command));
        self.indent_level -= 1;
        output.push_str("};\n");
        
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
} 