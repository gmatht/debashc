use crate::ast::*;
use crate::generator::Generator;

pub fn generate_uniq_command(generator: &mut Generator, cmd: &SimpleCommand, input_var: &str) -> String {
    let mut output = String::new();
    
    let mut count = false;
    
    // Check for flags
    for arg in &cmd.args {
        if let Word::Literal(arg_str) = arg {
            if arg_str == "-c" {
                count = true;
            }
        }
    }
    
    output.push_str(&format!("my @lines = split(/\\n/, {});\n", input_var));
    if count {
        output.push_str("my %counts;\n");
        output.push_str("foreach my $line (@lines) {\n");
        output.push_str("$counts{$line}++;\n");
        output.push_str("}\n");
        output.push_str("my @result;\n");
        output.push_str("foreach my $line (keys %counts) {\n");
        output.push_str("push @result, sprintf(\"%d %s\", $counts{$line}, $line);\n");
        output.push_str("}\n");
        output.push_str(&format!("{} = join(\"\\n\", @result);\n", input_var));
    } else {
        output.push_str("my %seen;\n");
        output.push_str("my @result;\n");
        output.push_str("foreach my $line (@lines) {\n");
        output.push_str("push @result, $line unless $seen{$line}++;\n");
        output.push_str("}\n");
        output.push_str(&format!("{} = join(\"\\n\", @result);\n", input_var));
    }
    
    output
}
