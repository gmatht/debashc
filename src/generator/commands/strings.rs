use crate::ast::*;
use crate::generator::Generator;

pub fn generate_strings_command(_generator: &mut Generator, cmd: &SimpleCommand, input_var: &str) -> String {
    let mut output = String::new();
    
    // strings command syntax: strings [options] file
    // Extracts printable strings from binary files
    let mut min_length = 4; // Default minimum string length
    
    // Parse strings options
    for arg in &cmd.args {
        if let Word::Literal(arg_str) = arg {
            if arg_str.starts_with("-n") {
                // Parse minimum length option
                if let Some(length_str) = arg_str.strip_prefix("-n") {
                    if let Ok(length) = length_str.parse::<usize>() {
                        min_length = length;
                    }
                }
            }
        }
    }
    
    output.push_str(&format!("my @lines = split(/\\n/, {});\n", input_var));
    output.push_str("my @result;\n");
    output.push_str("foreach my $line (@lines) {\n");
    output.push_str("chomp($line);\n");
    output.push_str(&format!("if (length($line) >= {}) {{\n", min_length));
    output.push_str("if ($line =~ /^[\\x20-\\x7E]+$/) {\n"); // Printable ASCII only
    output.push_str("push @result, $line;\n");
    output.push_str("}\n");
    output.push_str("}\n");
    output.push_str("}\n");
    output.push_str(&format!("{} = join(\"\\n\", @result);\n", input_var));
    output.push_str("\n");
    
    output
}
