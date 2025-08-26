use crate::ast::*;
use crate::generator::Generator;

pub fn generate_cut_command(generator: &mut Generator, cmd: &SimpleCommand, input_var: &str) -> String {
    let mut output = String::new();
    
    // cut command syntax: cut -d delimiter -f fields
    let mut delimiter = "\t".to_string(); // Default tab delimiter
    let mut fields = "1".to_string(); // Default to first field
    
    // Parse cut options
    let mut i = 0;
    while i < cmd.args.len() {
        if let Word::Literal(arg) = &cmd.args[i] {
            if arg == "-d" && i + 1 < cmd.args.len() {
                if let Some(next_arg) = cmd.args.get(i + 1) {
                    delimiter = generator.word_to_perl(next_arg);
                    i += 1; // Skip the delimiter argument
                }
            } else if arg == "-f" && i + 1 < cmd.args.len() {
                if let Some(next_arg) = cmd.args.get(i + 1) {
                    fields = generator.word_to_perl(next_arg);
                    i += 1; // Skip the fields argument
                }
            }
        }
        i += 1;
    }
    
    output.push_str(&format!("my @lines = split(/\\n/, {});\n", input_var));
    output.push_str("my @result;\n");
    output.push_str("foreach my $line (@lines) {\n");
    output.push_str("chomp($line);\n");
    output.push_str(&format!("my @fields = split(/{}/, $line);\n", delimiter));
    
    // Handle field selection (simple implementation for now)
    output.push_str(&format!("if (@fields > 0) {{\n"));
    output.push_str(&format!("push @result, $fields[0];\n")); // Default to first field
    output.push_str("}\n");
    output.push_str("}\n");
    output.push_str(&format!("{} = join(\"\\n\", @result);\n", input_var));
    
    output
}
