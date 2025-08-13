use crate::ast::*;

pub struct JsGenerator {}

impl JsGenerator {
    pub fn new() -> Self { Self {} }

    pub fn generate(&mut self, commands: &[Command]) -> String {
        let mut output = String::new();
        output.push_str("#!/usr/bin/env node\n");
        output.push_str("const { execSync } = require('child_process');\n\n");
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
            Command::BlankLine => String::from("\n"),
        }
    }

    fn generate_simple_command(&self, cmd: &SimpleCommand) -> String {
        if cmd.name == "echo" {
            if cmd.args.is_empty() {
                return String::from("console.log();\n");
            } else {
                let args = cmd.args.iter().map(|arg| arg.to_string()).collect::<Vec<_>>().join(" ");
                return format!("console.log({});\n", self.escape_js_string(&args));
            }
        } else if cmd.name == "cd" {
            // Special handling for cd with tilde expansion
            if cmd.args.is_empty() {
                return String::from("// cd to current directory (no-op)\n");
            } else {
                let dir = &cmd.args[0];
                let dir_str = self.word_to_string(dir);
                
                if dir_str == "~" {
                    // Handle tilde expansion for home directory
                    return String::from("const home = process.env.HOME || process.env.USERPROFILE;\nif (home) {\n    process.chdir(home);\n} else {\n    console.error('Cannot determine home directory');\n    process.exit(1);\n}\n");
                } else if dir_str.starts_with("~/") {
                    // Handle tilde expansion with subdirectory
                    let subdir = &dir_str[2..]; // Remove "~/"
                    return format!("const home = process.env.HOME || process.env.USERPROFILE;\nif (home) {{\n    const path = require('path');\n    process.chdir(path.join(home, '{}'));\n}} else {{\n    console.error('Cannot determine home directory');\n    process.exit(1);\n}}\n", subdir);
                } else {
                    // Regular directory change
                    return format!("process.chdir('{}');\n", dir_str);
                }
            }
        } else if cmd.name == "shopt" {
            return String::from("// builtin\n");
        }
        let sys = self.command_to_shell(cmd);
        format!("execSync(\"{}\", {{ stdio: 'inherit' }});\n", self.escape_js_raw(&sys))
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

    fn generate_test_expression(&mut self, test_expr: &TestExpression) -> String {
        let mut output = String::new();
        
        // Handle test modifiers if they're set
        if test_expr.modifiers.extglob {
            output.push_str("// extglob enabled\n");
        }
        if test_expr.modifiers.nocasematch {
            output.push_str("// nocasematch enabled\n");
        }
        if test_expr.modifiers.globstar {
            output.push_str("// globstar enabled\n");
        }
        if test_expr.modifiers.nullglob {
            output.push_str("// nullglob enabled\n");
        }
        if test_expr.modifiers.failglob {
            output.push_str("// failglob enabled\n");
        }
        if test_expr.modifiers.dotglob {
            output.push_str("// dotglob enabled\n");
        }
        
        // Generate the test expression
        // For now, just generate a comment with the expression
        output.push_str(&format!("// test expression: {}\n", test_expr.expression));
        output.push_str("// TODO: implement test expression logic\n");
        
        output
    }

    fn generate_while_loop(&mut self, _while_loop: &WhileLoop) -> String {
        let mut output = String::new();
        output.push_str("// while not implemented\n");
        output
    }

    fn generate_for_loop(&mut self, _for_loop: &ForLoop) -> String {
        let mut output = String::new();
        output.push_str("// for not implemented\n");
        output
    }

    fn generate_function(&mut self, _func: &Function) -> String {
        let mut output = String::new();
        output.push_str("// function not implemented\n");
        output
    }

    fn generate_subshell(&mut self, cmd: &Command) -> String {
        // Inline execution of subshell (no isolation)
        self.generate_command(cmd)
    }

    fn generate_background(&mut self, cmd: &Command) -> String {
        // Fire-and-forget using child_process without waiting
        let body = match cmd { Command::Simple(s) => self.command_to_shell(&s), _ => String::from("") };
        format!("require('child_process').exec(\"{}\");\n", self.escape_js_raw(&body))
    }

    fn generate_block(&mut self, block: &Block) -> String {
        let mut out = String::new();
        for c in &block.commands { 
            out.push_str(&self.generate_command(c)); 
        }
        out
    }

    fn generate_pipeline(&self, pipeline: &Pipeline) -> String {
        let mut out = String::new();
        for cmd in &pipeline.commands {
            if let Command::Simple(simple) = cmd {
                let sys = self.command_to_shell(simple);
                out.push_str(&format!("execSync(\"{}\", {{ stdio: 'inherit' }});\n", self.escape_js_raw(&sys)));
            }
        }
        out
    }

    fn generate_if_statement(&mut self, if_stmt: &IfStatement) -> String {
        let mut out = String::new();
        out.push_str("// if condition\n");
        out.push_str(&self.generate_command(&if_stmt.then_branch));
        if let Some(else_b) = &if_stmt.else_branch {
            out.push_str("// else\n");
            out.push_str(&self.generate_command(else_b));
        }
        out
    }

    fn command_to_shell(&self, cmd: &SimpleCommand) -> String {
        if cmd.args.is_empty() {
            cmd.name.to_string()
        } else {
            let args = cmd.args.iter().map(|arg| arg.to_string()).collect::<Vec<_>>().join(" ");
            format!("{} {}", cmd.name, args)
        }
    }

    fn escape_js_string(&self, s: &str) -> String {
        if s.contains('"') && s.contains('\'') {
            format!("`{}`", s.replace("`", "\\`"))
        } else if s.contains('"') {
            format!("'{}'", s.replace("'", "\\'"))
        } else {
            format!("\"{}\"", s.replace("\"", "\\\""))
        }
    }

    fn word_to_string(&self, word: &Word) -> String {
        match word {
            Word::Literal(s) => s.clone(),
            Word::Variable(var) => format!("${}", var),
            _ => word.to_string(),
        }
    }

    fn escape_js_raw(&self, s: &str) -> String {
        s.replace('\\', "\\\\").replace('"', "\\\"")
    }
}



