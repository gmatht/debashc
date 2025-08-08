use crate::ast::*;

pub struct JsGenerator {
    indent_level: usize,
}

impl JsGenerator {
    pub fn new() -> Self {
        Self { indent_level: 0 }
    }

    pub fn generate(&mut self, commands: &[Command]) -> String {
        let mut output = String::new();
        output.push_str("#!/usr/bin/env node\n");
        output.push_str("const { execSync } = require('child_process');\n\n");
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
            Command::While(_) => String::from("// while not implemented\n"),
            Command::For(_) => String::from("// for not implemented\n"),
            Command::Function(_) => String::from("// function not implemented\n"),
            Command::Subshell(_) => String::from("// subshell not implemented\n"),
        }
    }

    fn generate_simple_command(&self, cmd: &SimpleCommand) -> String {
        if cmd.name == "echo" {
            if cmd.args.is_empty() {
                return String::from("console.log();\n");
            } else {
                let args = cmd.args.join(" ");
                return format!("console.log({});\n", self.escape_js_string(&args));
            }
        }
        let sys = self.command_to_shell(cmd);
        format!("execSync(\"{}\", {{ stdio: 'inherit' }});\n", self.escape_js_raw(&sys))
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
            cmd.name.clone()
        } else {
            let args = cmd.args.join(" ");
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

    fn escape_js_raw(&self, s: &str) -> String {
        s.replace('\\', "\\\\").replace('"', "\\\"")
    }
}


