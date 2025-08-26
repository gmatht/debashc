use crate::ast::*;
use crate::generator::Generator;

pub fn generate_sort_command(generator: &mut Generator, cmd: &SimpleCommand, input_var: &str) -> String {
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
        output.push_str("my @sorted = sort { $a <=> $b } @lines;\n");
    } else {
        output.push_str("my @sorted = sort @lines;\n");
    }
    if reverse {
        output.push_str("@sorted = reverse(@sorted);\n");
    }
    output.push_str(&format!("{} = join(\"\\n\", @sorted);\n", input_var));
    
    output
}
