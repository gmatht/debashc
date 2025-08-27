use crate::ast::*;
use crate::generator::Generator;

pub fn generate_comm_command(_generator: &mut Generator, _cmd: &SimpleCommand, input_var: &str) -> String {
    let mut output = String::new();
    
    // comm compares two sorted files and shows lines unique to each
    // For now, implement a basic version that works with the input
    output.push_str(&format!("my @lines = split(/\\n/, {});\n", input_var));
    output.push_str("my %seen;\n");
    output.push_str("my @result;\n");
    output.push_str("foreach my $line (@lines) {\n");
    output.push_str("chomp($line);\n");
    output.push_str("if (!exists($seen{$line})) {\n");
    output.push_str("$seen{$line} = 1;\n");
    output.push_str("push @result, $line;\n");
    output.push_str("} else {\n");
    output.push_str("$seen{$line}++;\n");
    output.push_str("}\n");
    output.push_str("}\n");
    output.push_str(&format!("{} = join(\"\\n\", @result);\n", input_var));
    
    output
}
