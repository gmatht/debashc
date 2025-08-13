use crate::ast::*;

pub struct BatchGenerator;

impl BatchGenerator {
    pub fn new() -> Self { Self }

    pub fn generate(&mut self, commands: &[Command]) -> String {
        let mut output = String::new();
        output.push_str("@echo off\n");
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
            Command::Pipeline(pipeline) => self.generate_pipeline_wrapper(pipeline),
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

    fn generate_simple_command(&mut self, cmd: &SimpleCommand) -> String {
        self.generate_simple(cmd)
    }

    fn generate_shopt_command(&mut self, cmd: &ShoptCommand) -> String {
        self.generate_shopt(cmd)
    }

    fn generate_test_expression(&mut self, test_expr: &TestExpression) -> String {
        let mut output = String::new();
        
        // Handle test modifiers if they're set
        if test_expr.modifiers.extglob {
            output.push_str("REM extglob enabled\n");
        }
        if test_expr.modifiers.nocasematch {
            output.push_str("REM nocasematch enabled\n");
        }
        if test_expr.modifiers.globstar {
            output.push_str("REM globstar enabled\n");
        }
        if test_expr.modifiers.nullglob {
            output.push_str("REM nullglob enabled\n");
        }
        if test_expr.modifiers.failglob {
            output.push_str("REM failglob enabled\n");
        }
        if test_expr.modifiers.dotglob {
            output.push_str("REM dotglob enabled\n");
        }
        
        // Generate the test expression
        // For now, just generate a comment with the expression
        output.push_str(&format!("REM test expression: {}\n", test_expr.expression));
        output.push_str("REM TODO: implement test expression logic\n");
        
        output
    }

    fn generate_pipeline_wrapper(&mut self, pipeline: &Pipeline) -> String {
        self.generate_pipeline_impl(pipeline)
    }

    fn generate_if_statement(&mut self, if_stmt: &IfStatement) -> String {
        self.generate_if(if_stmt)
    }

    fn generate_while_loop(&mut self, _while_loop: &WhileLoop) -> String {
        String::from("REM while not implemented\n")
    }

    fn generate_for_loop(&mut self, _for_loop: &ForLoop) -> String {
        String::from("REM for not implemented\n")
    }

    fn generate_function(&mut self, _func: &Function) -> String {
        String::from("REM function not implemented\n")
    }

    fn generate_subshell(&mut self, _cmd: &Command) -> String {
        String::from("REM subshell not implemented\n")
    }

    fn generate_background(&mut self, cmd: &Command) -> String {
        // Start in background using start /B
        if let Command::Simple(s) = cmd {
            if s.args.is_empty() { 
                format!("start /B {}\n", s.name) 
            } else { 
                format!("start /B {} {}\n", s.name, s.args.join(" ")) 
            }
        } else {
            String::from("REM background compound command not implemented\n")
        }
    }

    fn generate_block(&mut self, block: &Block) -> String {
        let mut out = String::new();
        for c in &block.commands { 
            out.push_str(&self.generate_command(c)); 
        }
        out
    }

    fn generate_simple(&self, cmd: &SimpleCommand) -> String {
        if cmd.name == "echo" {
            if cmd.args.is_empty() { "echo.\n".to_string() } else { format!("echo {}\n", cmd.args.join(" ")) }
        } else if cmd.name == "cd" {
            // Special handling for cd with tilde expansion
            if cmd.args.is_empty() {
                "REM cd to current directory (no-op)\n".to_string()
            } else {
                let dir = &cmd.args[0];
                let dir_str = dir.as_str();
                
                if dir_str == "~" {
                    // Handle tilde expansion for home directory
                    "if defined USERPROFILE (\n    cd /d \"%USERPROFILE%\"\n) else (\n    echo Cannot determine home directory\n    exit /b 1\n)\n".to_string()
                } else if dir_str.starts_with("~/") {
                    // Handle tilde expansion with subdirectory
                    let subdir = &dir_str[2..]; // Remove "~/"
                    format!("if defined USERPROFILE (\n    cd /d \"%USERPROFILE%\\{}\"\n) else (\n    echo Cannot determine home directory\n    exit /b 1\n)\n", subdir)
                } else {
                    // Regular directory change
                    format!("cd /d \"{}\"\n", dir_str)
                }
            }
        } else if cmd.name == "shopt" {
            // Builtin: ignore
            "REM builtin\n".to_string()
        } else {
            if cmd.args.is_empty() { format!("{}\n", cmd.name) } else { format!("{} {}\n", cmd.name, cmd.args.join(" ")) }
        }
    }

    fn generate_shopt(&self, cmd: &ShoptCommand) -> String {
        let mut output = String::new();
        
        // Handle shopt command for shell options
        if cmd.enable {
            match cmd.option.as_str() {
                "extglob" => {
                    output.push_str("REM extglob option enabled\n");
                }
                "nocasematch" => {
                    output.push_str("REM nocasematch option enabled\n");
                }
                _ => {
                    output.push_str(&format!("REM shopt -s {} not implemented\n", cmd.option));
                }
            }
        } else {
            match cmd.option.as_str() {
                "extglob" => {
                    output.push_str("REM extglob option disabled\n");
                }
                "nocasematch" => {
                    output.push_str("REM nocasematch option disabled\n");
                }
                _ => {
                    output.push_str(&format!("REM shopt -u {} not implemented\n", cmd.option));
                }
            }
        }
        
        output
    }

    fn generate_pipeline_impl(&self, pipeline: &Pipeline) -> String {
        let mut out = String::new();
        out.push_str("REM pipeline approximation\n");
        for c in &pipeline.commands {
            if let Command::Simple(s) = c {
                out.push_str(&self.generate_simple(s));
            }
        }
        out
    }

    fn generate_if(&mut self, if_stmt: &IfStatement) -> String {
        let mut out = String::new();
        out.push_str("REM if condition\n");
        out.push_str(&self.generate_command(&if_stmt.then_branch));
        if let Some(else_b) = &if_stmt.else_branch {
            out.push_str("REM else\n");
            out.push_str(&self.generate_command(else_b));
        }
        out
    }
}



