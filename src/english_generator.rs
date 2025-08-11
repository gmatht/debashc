use crate::ast::*;

pub struct EnglishGenerator;

impl EnglishGenerator {
    pub fn new() -> Self { Self }

    pub fn generate(&mut self, commands: &[Command]) -> String {
        let mut out = String::new();
        for c in commands {
            out.push_str(&self.generate_command(c));
        }
        while out.ends_with('\n') { out.pop(); }
        out
    }

    fn generate_command(&mut self, c: &Command) -> String {
        match c {
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

    fn generate_simple_command(&mut self, cmd: &SimpleCommand) -> String {
        self.describe_command(&Command::Simple(cmd.clone()))
    }

    fn generate_shopt_command(&mut self, cmd: &ShoptCommand) -> String {
        self.describe_command(&Command::ShoptCommand(cmd.clone()))
    }

    fn generate_test_expression(&mut self, test_expr: &TestExpression) -> String {
        let mut output = String::new();
        
        // Handle test modifiers if they're set
        if test_expr.modifiers.extglob {
            output.push_str("Enable extended globbing.\n");
        }
        if test_expr.modifiers.nocasematch {
            output.push_str("Enable case-insensitive matching.\n");
        }
        if test_expr.modifiers.globstar {
            output.push_str("Enable globstar pattern matching.\n");
        }
        if test_expr.modifiers.nullglob {
            output.push_str("Enable nullglob pattern matching.\n");
        }
        if test_expr.modifiers.failglob {
            output.push_str("Enable failglob pattern matching.\n");
        }
        if test_expr.modifiers.dotglob {
            output.push_str("Enable dotglob pattern matching.\n");
        }
        
        // Generate the test expression description
        output.push_str(&format!("Evaluate test expression: {}.\n", test_expr.expression));
        
        output
    }

    fn generate_pipeline(&mut self, pipeline: &Pipeline) -> String {
        self.describe_command(&Command::Pipeline(pipeline.clone()))
    }

    fn generate_if_statement(&mut self, if_stmt: &IfStatement) -> String {
        self.describe_command(&Command::If(if_stmt.clone()))
    }

    fn generate_while_loop(&mut self, while_loop: &WhileLoop) -> String {
        self.describe_command(&Command::While(while_loop.clone()))
    }

    fn generate_for_loop(&mut self, for_loop: &ForLoop) -> String {
        self.describe_command(&Command::For(for_loop.clone()))
    }

    fn generate_function(&mut self, func: &Function) -> String {
        self.describe_command(&Command::Function(func.clone()))
    }

    fn generate_subshell(&mut self, cmd: &Command) -> String {
        self.describe_command(&Command::Subshell(Box::new(cmd.clone())))
    }

    fn generate_background(&mut self, cmd: &Command) -> String {
        self.describe_command(&Command::Background(Box::new(cmd.clone())))
    }

    fn generate_block(&mut self, block: &Block) -> String {
        self.describe_command(&Command::Block(block.clone()))
    }

    fn describe_command(&self, c: &Command) -> String {
        match c {
            Command::Simple(cmd) => {
                if cmd.name == "echo" {
                    if cmd.args.is_empty() { "Print a blank line.\n".to_string() } else { format!("Print: {}.\n", cmd.args.iter().map(|arg| arg.to_string()).collect::<Vec<_>>().join(" ")) }
                } else {
                    if cmd.args.is_empty() { format!("Run '{}'.\n", cmd.name) } else { format!("Run '{}' with arguments '{}'.\n", cmd.name, cmd.args.iter().map(|arg| arg.to_string()).collect::<Vec<_>>().join(" ")) }
                }
            }
            Command::ShoptCommand(cmd) => {
                if cmd.enable {
                    format!("Enable shell option '{}'.\n", cmd.option)
                } else {
                    format!("Disable shell option '{}'.\n", cmd.option)
                }
            }
            Command::TestExpression(test_expr) => {
                let mut output = String::new();
                output.push_str(&format!("Evaluate test expression: {}.\n", test_expr.expression));
                output
            }
            Command::Pipeline(p) => {
                let mut s = String::from("Create a pipeline: ");
                let parts: Vec<String> = p.commands.iter().map(|pc| match pc { Command::Simple(sc) => sc.name.to_string(), _ => String::from("command") }).collect();
                s.push_str(&parts.join(" | "));
                s.push_str(".\n");
                s
            }
            Command::If(ifc) => {
                let mut s = String::from("If condition holds, then: \n");
                s.push_str(&self.describe_command(&ifc.then_branch));
                if let Some(e) = &ifc.else_branch {
                    s.push_str("Otherwise: \n");
                    s.push_str(&self.describe_command(e));
                }
                s
            }
            Command::While(_) => String::from("Repeat while condition holds.\n"),
            Command::For(_) => String::from("Loop over items.\n"),
            Command::Function(f) => format!("Define function '{}'.\n", f.name),
            Command::Subshell(_) => String::from("Run in a subshell.\n"),
            Command::Background(cmd) => {
                let mut s = String::from("Run in background: \n");
                s.push_str(&self.describe_command(cmd));
                s
            }
            Command::Block(block) => {
                let mut s = String::from("Execute a block of commands:\n");
                for c in &block.commands { s.push_str(&self.describe_command(c)); }
                s
            }
            Command::BuiltinCommand(_) => "Execute a builtin command.\n".to_string(),
            Command::BlankLine => String::from("\n"),
        }
    }
}



