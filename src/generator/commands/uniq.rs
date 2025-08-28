use crate::ast::*;
use crate::generator::Generator;

pub fn generate_uniq_command(generator: &mut Generator, cmd: &SimpleCommand, input_var: &str, command_index: usize) -> String {
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
    
    output.push_str(&format!("my @uniq_lines_{} = split(/\\n/, {});\n", command_index, input_var));
    if count {
        output.push_str(&format!("my %uniq_counts_{};\n", command_index));
        output.push_str(&format!("foreach my $line (@uniq_lines_{}) {{\n", command_index));
        output.push_str(&format!("$uniq_counts_{}{{$line}}++;\n", command_index));
        output.push_str("}\n");
        output.push_str(&format!("my @uniq_result_{};\n", command_index));
        output.push_str(&format!("foreach my $line (keys %uniq_counts_{}) {{\n", command_index));
        output.push_str(&format!("push @uniq_result_{}, sprintf(\"%7d %s\", $uniq_counts_{}{{$line}}, $line);\n", command_index, command_index));
        output.push_str("}\n");
        output.push_str(&format!("{} = join(\"\\n\", @uniq_result_{});\n", input_var, command_index));
    } else {
        output.push_str(&format!("my %uniq_seen_{};\n", command_index));
        output.push_str(&format!("my @uniq_result_{};\n", command_index));
        output.push_str(&format!("foreach my $line (@uniq_lines_{}) {{\n", command_index));
        output.push_str(&format!("push @uniq_result_{}, $line unless $uniq_seen_{}{{$line}}++;\n", command_index, command_index));
        output.push_str("}\n");
        output.push_str(&format!("{} = join(\"\\n\", @uniq_result_{});\n", input_var, command_index));
    }
    
    output
}
