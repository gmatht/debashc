use crate::ast::*;

pub struct EnglishGenerator;

impl EnglishGenerator {
    pub fn new() -> Self { Self }

    pub fn generate(&mut self, commands: &[Command]) -> String {
        let mut out = String::new();
        for c in commands {
            out.push_str(&self.describe_command(c));
        }
        out
    }

    fn describe_command(&self, c: &Command) -> String {
        match c {
            Command::Simple(cmd) => {
                if cmd.name == "echo" {
                    if cmd.args.is_empty() { "Print a blank line.\n".to_string() } else { format!("Print: {}.\n", cmd.args.join(" ")) }
                } else {
                    if cmd.args.is_empty() { format!("Run '{}'.\n", cmd.name) } else { format!("Run '{}' with arguments '{}'.\n", cmd.name, cmd.args.join(" ")) }
                }
            }
            Command::Pipeline(p) => {
                let mut s = String::from("Create a pipeline: ");
                let parts: Vec<String> = p.commands.iter().map(|pc| match pc { Command::Simple(sc) => sc.name.clone(), _ => String::from("command") }).collect();
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
            Command::BlankLine => String::from("\n"),
        }
    }
}



