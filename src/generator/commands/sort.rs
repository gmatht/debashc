use crate::ast::*;
use crate::generator::Generator;

pub fn generate_sort_command(_generator: &mut Generator, cmd: &SimpleCommand, input_var: &str) -> String {
    let mut output = String::new();
    
    let mut numeric = false;
    let mut reverse = false;
    
    // Check for flags
    for arg in &cmd.args {
        if let Word::Literal(arg_str) = arg {
            if arg_str == "-n" {
                numeric = true;
            } else if arg_str == "r" || arg_str == "-r" {
                reverse = true;
            } else if arg_str == "-nr" || arg_str == "-rn" {
                numeric = true;
                reverse = true;
            }
        }
    }
    
    output.push_str(&format!("my @lines = split(/\\n/, {});\n", input_var));
    if numeric {
        // For numeric sort, extract the first field (number) and sort by that
        // Use the entire line as secondary sort key for stable sort behavior
        output.push_str("my @sorted = sort { \n");
        output.push_str("    my $a_num = (split(/\\s+/, $a))[0] || 0;\n");
        output.push_str("    my $b_num = (split(/\\s+/, $b))[0] || 0;\n");
        output.push_str("    $a_num <=> $b_num || $a cmp $b;\n");
        output.push_str("} @lines;\n");
    } else {
        output.push_str("my @sorted = sort @lines;\n");
    }
    if reverse {
        output.push_str("@sorted = reverse(@sorted);\n");
    }
    output.push_str(&format!("{} = join(\"\\n\", @sorted);\n", input_var));
    output.push_str("\n");
    
    output
}
