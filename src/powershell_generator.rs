use crate::ast::*;

pub struct PowerShellGenerator;

impl PowerShellGenerator {
    pub fn new() -> Self { Self }

    pub fn generate(&mut self, commands: &[Command]) -> String {
        let mut out = String::new();
        out.push_str("#requires -Version 5.0\n");
        for c in commands { out.push_str(&self.emit(c)); }
        out
    }

    fn emit(&self, c: &Command) -> String {
        match c {
            Command::Simple(cmd) => self.simple(cmd),
            Command::Pipeline(p) => self.pipeline(p),
            Command::If(i) => self.if_stmt(i),
            Command::While(_) => String::from("# while not implemented\n"),
            Command::For(_) => String::from("# for not implemented\n"),
            Command::Function(_) => String::from("# function not implemented\n"),
            Command::Subshell(cmd) => {
                // Inline execution for subshell
                self.emit(cmd)
            },
            Command::Background(cmd) => {
                // Run in background job
                let body = self.emit(cmd);
                format!("Start-Job -ScriptBlock {{\n{}\n}}\n", body)
            }
            Command::Block(block) => {
                let mut out = String::new();
                for c in &block.commands { out.push_str(&self.emit(c)); }
                out
            }
        }
    }

    fn simple(&self, cmd: &SimpleCommand) -> String {
        if cmd.name == "echo" {
            if cmd.args.is_empty() { "Write-Output \"\"\n".to_string() } else { format!("Write-Output {}\n", self.quote_join(&cmd.args)) }
        } else {
            if cmd.args.is_empty() { format!("{}\n", cmd.name) } else { format!("{} {}\n", cmd.name, self.quote_join(&cmd.args)) }
        }
    }

    fn pipeline(&self, p: &Pipeline) -> String {
        let mut parts: Vec<String> = Vec::new();
        for c in &p.commands {
            if let Command::Simple(s) = c {
                if s.args.is_empty() { parts.push(s.name.clone()); } else { parts.push(format!("{} {}", s.name, self.quote_join(&s.args))); }
            }
        }
        format!("{}\n", parts.join(" | "))
    }

    fn if_stmt(&self, i: &IfStatement) -> String {
        let mut out = String::new();
        out.push_str("# if condition\n");
        out.push_str(&self.emit(&i.then_branch));
        if let Some(e) = &i.else_branch {
            out.push_str("else {\n");
            out.push_str(&self.emit(e));
            out.push_str("}\n");
        }
        out
    }

    fn quote_join(&self, args: &[String]) -> String {
        args.iter().map(|a| format!("\"{}\"", a.replace('"', "`\""))).collect::<Vec<_>>().join(" ")
    }
}



