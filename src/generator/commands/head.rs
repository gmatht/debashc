use crate::ast::*;
use crate::generator::Generator;

pub fn generate_head_command(generator: &mut Generator, cmd: &SimpleCommand, input_var: &str) -> String {
    let mut output = String::new();
    
    // head command syntax: head [options] [file...]
    let mut num_lines = 10; // Default to first 10 lines
    
    // Parse head options
    for arg in &cmd.args {
        if let Word::Literal(arg_str) = arg {
            if arg_str.starts_with("-n") {
                if let Some(num_str) = arg_str.strip_prefix("-n") {
                    if let Ok(num) = num_str.parse::<usize>() {
                        num_lines = num;
                    }
                }
            } else if arg_str.starts_with("-") && arg_str.len() > 1 {
                // Handle -10, -20 style line counts
                if let Ok(num) = arg_str[1..].parse::<usize>() {
                    num_lines = num;
                }
            }
        }
    }
    
    output.push_str(&format!("my @lines = split(/\\n/, {});\n", input_var));
    output.push_str(&format!("my $num_lines = {};\n", num_lines));
    output.push_str("if ($num_lines > scalar(@lines)) {\n");
    output.push_str("$num_lines = scalar(@lines);\n");
    output.push_str("}\n");
    output.push_str("my @result = @lines[0..$num_lines-1];\n");
    output.push_str(&format!("{} = join(\"\\n\", @result);\n", input_var));
    
    output
}
