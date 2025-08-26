use crate::ast::*;
use crate::generator::Generator;

pub fn generate_sed_command(generator: &mut Generator, cmd: &SimpleCommand, input_var: &str) -> String {
    let mut output = String::new();
    
    // For now, implement basic sed-like functionality
    // This can be extended to handle more complex sed patterns
    output.push_str(&format!("my @lines = split(/\\n/, {});\n", input_var));
    output.push_str("my @result;\n");
    output.push_str("foreach my $line (@lines) {\n");
    output.push_str("chomp($line);\n");
    
    // Handle common sed operations
    if let Some(operation) = cmd.args.first() {
        if let Word::Literal(op) = operation {
            if op.starts_with("s/") {
                // Basic substitution: s/pattern/replacement/
                let parts: Vec<&str> = op.split('/').collect();
                if parts.len() >= 3 {
                    let pattern = parts[1];
                    let replacement = parts[2];
                    output.push_str(&format!("$line =~ s/{}/{}/g;\n", pattern, replacement));
                }
            } else if op == "d" {
                // Delete lines
                output.push_str("next;\n");
            }
        }
    }
    
    output.push_str("push @result, $line;\n");
    output.push_str("}\n");
    output.push_str(&format!("{} = join(\"\\n\", @result);\n", input_var));
    
    output
}
