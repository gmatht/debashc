use crate::ast::*;

pub struct PowerShellGenerator;

impl PowerShellGenerator {
    pub fn new() -> Self { Self }

    pub fn generate(&mut self, commands: &[Command]) -> String {
        let mut out = String::new();
        out.push_str("#requires -Version 5.0\n");
        for c in commands { out.push_str(&self.emit(c)); }
        while out.ends_with('\n') { out.pop(); }
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
            Command::BlankLine => "\n".to_string(),
        }
    }

    fn simple(&self, cmd: &SimpleCommand) -> String {
        if cmd.name == "echo" {
            if cmd.args.is_empty() { "Write-Output \"\"\n".to_string() } else { format!("Write-Output {}\n", self.quote_join(&cmd.args)) }
        } else if cmd.name == "shopt" {
            // Builtin: ignore
            return String::from("# builtin\n");
        } else if cmd.name == "cat" {
            // Special handling for cat including heredocs
            let mut output = String::new();
            let mut printed_any = false;
            for redir in &cmd.redirects {
                if matches!(redir.operator, RedirectOperator::Heredoc | RedirectOperator::HeredocTabs) {
                    if let Some(body) = &redir.heredoc_body {
                        // Normalize line endings to handle Windows vs Unix line endings
                        let normalized_body = body.replace("\r\n", "\n").replace("\r", "\n");
                        // Escape PowerShell string properly
                        let escaped_body = normalized_body.replace("\"", "`\"").replace("$", "`$");
                        output.push_str(&format!("Write-Output @\"\n{}\"@\n", escaped_body));
                        printed_any = true;
                    }
                }
            }
            if !printed_any {
                for arg in &cmd.args {
                    return format!("Get-Content \"{}\" | Write-Output\n", arg);
                }
            }
            return output;
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
        // For now, implement a simple condition check
        match &*i.condition {
            Command::Simple(cmd) if cmd.name == "[" || cmd.name == "test" => {
                if let Some(test_op) = cmd.args.get(0) {
                    match test_op.as_str() {
                        "-f" => {
                            if let Some(file) = cmd.args.get(1) {
                                out.push_str(&format!("if (Test-Path '{}' -PathType Leaf) {{\n", file));
                            } else {
                                out.push_str("if ($false) {\n");
                            }
                        }
                        "-d" => {
                            if let Some(dir) = cmd.args.get(1) {
                                out.push_str(&format!("if (Test-Path '{}' -PathType Container) {{\n", dir));
                            } else {
                                out.push_str("if ($false) {\n");
                            }
                        }
                        "-e" => {
                            if let Some(path) = cmd.args.get(1) {
                                out.push_str(&format!("if (Test-Path '{}') {{\n", path));
                            } else {
                                out.push_str("if ($false) {\n");
                            }
                        }
                        _ => {
                            out.push_str("if ($true) {\n");
                        }
                    }
                } else {
                    out.push_str("if ($true) {\n");
                }
            }
            _ => {
                out.push_str("if ($true) {\n");
            }
        }
        out.push_str(&self.emit(&i.then_branch));
        if let Some(e) = &i.else_branch {
            out.push_str("} else {\n");
            out.push_str(&self.emit(e));
            out.push_str("}\n");
        } else {
            out.push_str("}\n");
        }
        out
    }

    fn quote_join(&self, args: &[String]) -> String {
        args.iter().map(|a| self.translate_arg(a)).collect::<Vec<_>>().join(" ")
    }

    fn translate_arg(&self, arg: &str) -> String {
        match arg {
            "$#" => "$($args.Count)".to_string(),
            "$@" => "$args".to_string(),
            "$*" => "$($args -join ' ')".to_string(),
            _ => {
                // Handle strings with quotes more carefully
                if arg.contains('"') || arg.contains('$') {
                    // Use single quotes to avoid escaping issues
                    format!("'{}'", arg.replace("'", "''"))
                } else {
                    format!("\"{}\"", arg)
                }
            }
        }
    }
}



