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
        output
    }

    fn generate_command(&self, command: &Command) -> String {
        match command {
            Command::Simple(cmd) => self.generate_simple(cmd),
            Command::Pipeline(p) => self.generate_pipeline(p),
            Command::If(i) => self.generate_if(i),
            Command::While(_) => String::from("REM while not implemented\n"),
            Command::For(_) => String::from("REM for not implemented\n"),
            Command::Function(_) => String::from("REM function not implemented\n"),
            Command::Subshell(_) => String::from("REM subshell not implemented\n"),
        }
    }

    fn generate_simple(&self, cmd: &SimpleCommand) -> String {
        if cmd.name == "echo" {
            if cmd.args.is_empty() { "echo.\n".to_string() } else { format!("echo {}\n", cmd.args.join(" ")) }
        } else {
            if cmd.args.is_empty() { format!("{}\n", cmd.name) } else { format!("{} {}\n", cmd.name, cmd.args.join(" ")) }
        }
    }

    fn generate_pipeline(&self, pipeline: &Pipeline) -> String {
        let mut out = String::new();
        out.push_str("REM pipeline approximation\n");
        for c in &pipeline.commands {
            if let Command::Simple(s) = c {
                out.push_str(&self.generate_simple(s));
            }
        }
        out
    }

    fn generate_if(&self, if_stmt: &IfStatement) -> String {
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


