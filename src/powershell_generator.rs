use crate::ast::*;

pub struct PowerShellGenerator;

impl PowerShellGenerator {
    pub fn new() -> Self { Self }

    pub fn generate(&mut self, commands: &[Command]) -> String {
        let mut out = String::new();
        out.push_str("#requires -Version 5.0\n");
        for c in commands { out.push_str(&self.generate_command(c)); }
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
            Command::BlankLine => "\n".to_string(),
        }
    }

    fn generate_simple_command(&mut self, cmd: &SimpleCommand) -> String {
        self.simple(cmd)
    }

    fn generate_shopt_command(&mut self, cmd: &ShoptCommand) -> String {
        self.shopt(cmd)
    }

    fn generate_test_expression(&mut self, test_expr: &TestExpression) -> String {
        let mut output = String::new();
        
        // Handle test modifiers if they're set
        if test_expr.modifiers.extglob {
            output.push_str("# extglob enabled\n");
        }
        if test_expr.modifiers.nocasematch {
            output.push_str("# nocasematch enabled\n");
        }
        if test_expr.modifiers.globstar {
            output.push_str("# globstar enabled\n");
        }
        if test_expr.modifiers.nullglob {
            output.push_str("# nullglob enabled\n");
        }
        if test_expr.modifiers.failglob {
            output.push_str("# failglob enabled\n");
        }
        if test_expr.modifiers.dotglob {
            output.push_str("# dotglob enabled\n");
        }
        
        // Generate the test expression
        // For now, just generate a comment with the expression
        output.push_str(&format!("# test expression: {}\n", test_expr.expression));
        output.push_str("# TODO: implement test expression logic\n");
        
        output
    }

    fn generate_pipeline(&mut self, pipeline: &Pipeline) -> String {
        self.pipeline(pipeline)
    }

    fn generate_if_statement(&mut self, if_stmt: &IfStatement) -> String {
        // Call if_stmt directly to avoid trait method issues
        self.if_stmt(if_stmt)
    }

    fn generate_while_loop(&mut self, _while_loop: &WhileLoop) -> String {
        String::from("# while not implemented\n")
    }

    fn generate_for_loop(&mut self, _for_loop: &ForLoop) -> String {
        String::from("# for not implemented\n")
    }

    fn generate_function(&mut self, _func: &Function) -> String {
        String::from("# function not implemented\n")
    }

    fn generate_subshell(&mut self, cmd: &Command) -> String {
        // Inline execution for subshell
        self.generate_command(cmd)
    }

    fn generate_background(&mut self, cmd: &Command) -> String {
        // Run in background job
        let body = self.generate_command(cmd);
        format!("Start-Job -ScriptBlock {{\n{}\n}}\n", body)
    }

    fn generate_block(&mut self, block: &Block) -> String {
        let mut out = String::new();
        for c in &block.commands { 
            out.push_str(&self.generate_command(c)); 
        }
        out
    }

    fn simple(&self, cmd: &SimpleCommand) -> String {
        if cmd.name == "echo" {
            if cmd.args.is_empty() { "Write-Output \"\"\n".to_string() } else { format!("Write-Output {}\n", self.quote_join(&cmd.args.iter().map(|arg| arg.to_string()).collect::<Vec<_>>())) }
        } else if cmd.name == "cd" {
            // Special handling for cd with tilde expansion
            if cmd.args.is_empty() {
                return String::from("# cd to current directory (no-op)\n");
            } else {
                let dir = &cmd.args[0];
                let dir_str = dir.as_str();
                
                if dir_str == "~" {
                    // Handle tilde expansion for home directory
                    return String::from("$home = $env:USERPROFILE; if ($home) { Set-Location $home } else { Write-Error 'Cannot determine home directory'; exit 1 }\n");
                } else if dir_str.starts_with("~/") {
                    // Handle tilde expansion with subdirectory
                    let subdir = &dir_str[2..]; // Remove "~/"
                    return format!("$home = $env:USERPROFILE; if ($home) {{ Set-Location (Join-Path $home '{}') }} else {{ Write-Error 'Cannot determine home directory'; exit 1 }}\n", subdir);
                } else {
                    // Regular directory change
                    return format!("Set-Location '{}'\n", dir_str);
                }
            }
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
            if cmd.args.is_empty() { format!("{}\n", cmd.name) } else { format!("{} {}\n", cmd.name, self.quote_join(&cmd.args.iter().map(|arg| arg.to_string()).collect::<Vec<_>>())) }
        }
    }

    fn shopt(&self, cmd: &ShoptCommand) -> String {
        let mut output = String::new();
        
        // Handle shopt command for shell options
        if cmd.enable {
            match cmd.option.as_str() {
                "extglob" => {
                    output.push_str("# extglob option enabled\n");
                }
                "nocasematch" => {
                    output.push_str("# nocasematch option enabled\n");
                }
                _ => {
                    output.push_str(&format!("# shopt -s {} not implemented\n", cmd.option));
                }
            }
        } else {
            match cmd.option.as_str() {
                "extglob" => {
                    output.push_str("# extglob option disabled\n");
                }
                "nocasematch" => {
                    output.push_str("# nocasematch option disabled\n");
                }
                _ => {
                    output.push_str(&format!("# shopt -u {} not implemented\n", cmd.option));
                }
            }
        }
        
        output
    }

    fn pipeline(&self, p: &Pipeline) -> String {
        let mut parts: Vec<String> = Vec::new();
        for c in &p.commands {
            if let Command::Simple(s) = c {
                if s.args.is_empty() { parts.push(s.name.to_string()); } else { parts.push(format!("{} {}", s.name, self.quote_join(&s.args.iter().map(|arg| arg.to_string()).collect::<Vec<_>>()))); }
            }
        }
        format!("{}\n", parts.join(" | "))
    }

    fn if_stmt(&mut self, i: &IfStatement) -> String {
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
        out.push_str(&self.generate_command(&i.then_branch));
        if let Some(e) = &i.else_branch {
            out.push_str("} else {\n");
            out.push_str(&self.generate_command(e));
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



