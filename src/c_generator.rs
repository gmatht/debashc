use crate::ast::*;
use std::collections::HashMap;

pub struct CGenerator {
    indent_level: usize,
    // Track loop variables and their inferred C types for simple interpolation
    loop_vars: Vec<(String, LoopVarType)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LoopVarType {
    Integer,
    String,
}

impl CGenerator {
    pub fn new() -> Self {
        Self { indent_level: 0, loop_vars: Vec::new() }
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
            Command::For(for_loop) => self.generate_for_loop(for_loop),
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
                // Build printf format with simple variable interpolation for loop vars
                let mut fmt = String::new();
                let mut printf_args: Vec<(String, LoopVarType)> = Vec::new();
                for (index, arg) in cmd.args.iter().enumerate() {
                    if let Some(var_name) = arg.strip_prefix('$') {
                        if let Some(vt) = self.lookup_loop_var_type(var_name) {
                            match vt {
                                LoopVarType::Integer => fmt.push_str("%lld"),
                                LoopVarType::String => fmt.push_str("%s"),
                            }
                            printf_args.push((var_name.to_string(), vt));
                        } else {
                            // Treat as literal if not a known loop var
                            fmt.push_str(&self.escape_c_string(arg));
                        }
                    } else {
                        fmt.push_str(&self.escape_c_string(arg));
                    }
                    if index + 1 < cmd.args.len() {
                        fmt.push(' ');
                    }
                }
                fmt.push('\n');
                if printf_args.is_empty() {
                    line.push_str(&format!("printf(\"{}\");\n", fmt));
                } else {
                    // Build printf with arguments
                    line.push_str("printf(\"");
                    line.push_str(&fmt);
                    line.push_str("\"");
                    for (name, vt) in printf_args {
                        match vt {
                            LoopVarType::Integer => {
                                line.push_str(&format!(", (long long){}", name));
                            }
                            LoopVarType::String => {
                                line.push_str(&format!(", {}", name));
                            }
                        }
                    }
                    line.push_str(");\n");
                }
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

    fn generate_for_loop(&mut self, for_loop: &ForLoop) -> String {
        let var = &for_loop.variable;
        // Attempt numeric range like {0..5}
        let numeric_range = for_loop
            .items
            .get(0)
            .and_then(|s| self.parse_numeric_brace_range(s));

        let mut out = String::new();
        match numeric_range {
            Some((start, end)) => {
                // Integer iteration
                let (cmp, step) = if start <= end {
                    ("<=", 1i64)
                } else {
                    (">=", -1i64)
                };
                out.push_str(&format!(
                    "for (long long {} = {}; {} {} {}; {} += {}) {{\n",
                    var, start, var, cmp, end, var, step
                ));
                self.indent_level += 1;
                self.loop_vars.push((var.clone(), LoopVarType::Integer));
                out.push_str(&self.indent());
                out.push_str(&self.generate_command(&for_loop.body));
                self.loop_vars.pop();
                self.indent_level -= 1;
                out.push_str(&self.indent());
                out.push_str("}\n");
            }
            None => {
                if for_loop.items.is_empty() {
                    out.push_str("/* for loop without items not implemented */\n");
                } else {
                    // Treat as list of strings
                    let arr_name = format!("__items_{}", var);
                    out.push_str(&format!("const char* {}[] = {{ ", arr_name));
                    for (idx, item) in for_loop.items.iter().enumerate() {
                        let escaped = self.escape_c_string(item);
                        out.push_str(&format!("\"{}\"", escaped));
                        if idx + 1 < for_loop.items.len() { out.push_str(", "); }
                    }
                    out.push_str(" };\n");
                    out.push_str(&format!(
                        "for (size_t __idx_{v} = 0; __idx_{v} < (sizeof({arr})/sizeof({arr}[0])); ++__idx_{v}) {{\n",
                        v = var,
                        arr = arr_name
                    ));
                    self.indent_level += 1;
                    out.push_str(&self.indent());
                    out.push_str(&format!("const char* {} = {}[__idx_{}];\n", var, arr_name, var));
                    self.loop_vars.push((var.clone(), LoopVarType::String));
                    out.push_str(&self.indent());
                    out.push_str(&self.generate_command(&for_loop.body));
                    self.loop_vars.pop();
                    self.indent_level -= 1;
                    out.push_str(&self.indent());
                    out.push_str("}\n");
                }
            }
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

    fn parse_numeric_brace_range(&self, s: &str) -> Option<(i64, i64)> {
        // Matches forms like {0..5} or {10..3}
        if !(s.starts_with('{') && s.ends_with('}')) {
            return None;
        }
        let inner = &s[1..s.len() - 1];
        let parts: Vec<&str> = inner.split("..").collect();
        if parts.len() != 2 {
            return None;
        }
        let start = parts[0].parse::<i64>().ok()?;
        let end = parts[1].parse::<i64>().ok()?;
        Some((start, end))
    }

    fn lookup_loop_var_type(&self, name: &str) -> Option<LoopVarType> {
        for (n, t) in self.loop_vars.iter().rev() {
            if n == name {
                return Some(*t);
            }
        }
        None
    }
}



