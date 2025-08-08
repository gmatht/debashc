use crate::ast::*;

pub struct CGenerator {
    indent_level: usize,
}

impl CGenerator {
    pub fn new() -> Self {
        Self { indent_level: 0 }
    }

    pub fn generate(&mut self, commands: &[Command]) -> String {
        let mut output = String::new();
        output.push_str("#include <stdio.h>\n");
        output.push_str("#include <stdlib.h>\n\n");
        output.push_str("int main(void) {\n");
        self.indent_level += 1;

        for command in commands {
            output.push_str(&self.indent());
            output.push_str(&self.generate_command(command));
        }

        output.push_str(&self.indent());
        output.push_str("return 0;\n");
        self.indent_level -= 1;
        output.push_str("}\n");
        output
    }

    fn generate_command(&mut self, command: &Command) -> String {
        match command {
            Command::Simple(cmd) => self.generate_simple_command(cmd),
            Command::Pipeline(pipeline) => self.generate_pipeline(pipeline),
            Command::If(if_stmt) => self.generate_if_statement(if_stmt),
            Command::While(_) => String::from("/* while loop not implemented */\n"),
            Command::For(_) => String::from("/* for loop not implemented */\n"),
            Command::Function(_) => String::from("/* function not implemented */\n"),
            Command::Subshell(_) => String::from("/* subshell not implemented */\n"),
        }
    }

    fn generate_simple_command(&self, cmd: &SimpleCommand) -> String {
        let mut line = String::new();
        if cmd.name == "echo" {
            if cmd.args.is_empty() {
                line.push_str("printf(\"\\n\");\n");
            } else {
                let args = cmd.args.join(" ");
                let escaped = self.escape_c_string(&args);
                line.push_str(&format!("printf(\"{}\\n\");\n", escaped));
            }
        } else {
            // Fallback to system()
            let sys = self.command_to_shell(cmd);
            line.push_str(&format!("system(\"{}\");\n", sys));
        }
        line
    }

    fn generate_pipeline(&self, pipeline: &Pipeline) -> String {
        // Not implementing real piping; emit sequential system() calls as an approximation
        let mut out = String::new();
        out.push_str("/* pipeline */\n");
        for cmd in &pipeline.commands {
            if let Command::Simple(simple) = cmd {
                let sys = self.command_to_shell(simple);
                out.push_str(&format!("system(\"{}\");\n", sys));
            }
        }
        out
    }

    fn generate_if_statement(&mut self, if_stmt: &IfStatement) -> String {
        // Very naive: treat condition as comment and emit then/else bodies
        let mut out = String::new();
        out.push_str("/* if condition */\n");
        out.push_str(&self.generate_command(&if_stmt.then_branch));
        if let Some(else_b) = &if_stmt.else_branch {
            out.push_str("/* else */\n");
            out.push_str(&self.generate_command(else_b));
        }
        out
    }

    fn command_to_shell(&self, cmd: &SimpleCommand) -> String {
        if cmd.args.is_empty() {
            cmd.name.clone()
        } else {
            let args = cmd.args.join(" ");
            format!("{} {}", cmd.name, args)
        }
    }

    fn indent(&self) -> String {
        "    ".repeat(self.indent_level)
    }

    fn escape_c_string(&self, s: &str) -> String {
        s.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
    }
}


