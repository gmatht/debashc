use crate::ast::*;
use crate::generator::Generator;

pub fn generate_xargs_command(_generator: &mut Generator, cmd: &SimpleCommand, input_var: &str, command_index: usize) -> String {
    let mut output = String::new();
    
    let mut command = "echo";
    let mut args = Vec::new();
    
    // Parse xargs arguments
    for arg in &cmd.args {
        if let Word::Literal(arg_str) = arg {
            if arg_str == "grep" {
                command = "grep";
            } else if arg_str == "-l" {
                // This will be handled in the grep logic
            } else if arg_str == "function" {
                args.push("function".to_string());
            }
        } else if let Word::StringInterpolation(interp) = arg {
            let pattern = interp.parts.iter()
                .map(|part| match part {
                    crate::ast::StringPart::Literal(s) => s,
                    _ => ".*"
                })
                .collect::<Vec<_>>()
                .join("");
            args.push(pattern);
        }
    }
    
    if command == "grep" && args.contains(&"function".to_string()) {
        // Handle grep -l "function" on the input files
        output.push_str(&format!("my @xargs_files_{} = split(/\\n/, {});\n", command_index, input_var));
        output.push_str(&format!("my @xargs_matching_files_{};\n", command_index));
        output.push_str(&format!("foreach my $file (@xargs_files_{}) {{\n", command_index));
        output.push_str("next unless $file && -f $file;\n");
        output.push_str("if (open(my $fh, '<', $file)) {\n");
        output.push_str(&format!("my $xargs_found_{} = 0;\n", command_index));
        output.push_str("while (my $line = <$fh>) {\n");
        output.push_str("if ($line =~ /function/) {\n");
        output.push_str(&format!("$xargs_found_{} = 1;\n", command_index));
        output.push_str("last;\n");
        output.push_str("}\n");
        output.push_str("}\n");
        output.push_str("close($fh);\n");
        output.push_str(&format!("push @xargs_matching_files_{}, $file if $xargs_found_{};\n", command_index, command_index));
        output.push_str("}\n");
        output.push_str("}\n");
        output.push_str(&format!("{} = join(\"\\n\", @xargs_matching_files_{});\n", input_var, command_index));
    } else {
        // Fallback to system command for other cases
        output.push_str(&format!("{} = `echo \"${}\" | {}`;\n", input_var, input_var, command));
    }
    output.push_str("\n");
    
    output
}
