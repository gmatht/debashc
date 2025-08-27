use crate::ast::*;
use crate::generator::Generator;

pub fn generate_tail_command(_generator: &mut Generator, cmd: &SimpleCommand, input_var: &str) -> String {
    let mut output = String::new();
    
    // tail command syntax: tail [options] [file...]
    let mut num_lines = 10; // Default to last 10 lines
    let mut follow = false;
    
    // Parse tail options
    for arg in &cmd.args {
        if let Word::Literal(arg_str) = arg {
            match arg_str.as_str() {
                "-f" | "--follow" => follow = true,
                _ => {
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
        }
    }
    
    if follow {
        // Follow mode - this would require more complex logic in a real implementation
        output.push_str("warn \"tail: -f option not fully implemented in this version\\n\";\n");
    }
    
    output.push_str(&format!("my @lines = split(/\\n/, {});\n", input_var));
    output.push_str(&format!("my $num_lines = {};\n", num_lines));
    output.push_str("if ($num_lines > scalar(@lines)) {\n");
    output.push_str("$num_lines = scalar(@lines);\n");
    output.push_str("}\n");
    output.push_str("my $start_index = scalar(@lines) - $num_lines;\n");
    output.push_str("if ($start_index < 0) { $start_index = 0; }\n");
    output.push_str("my @result = @lines[$start_index..$#lines];\n");
    output.push_str(&format!("{} = join(\"\\n\", @result);\n", input_var));
    output.push_str("\n");
    
    output
}
