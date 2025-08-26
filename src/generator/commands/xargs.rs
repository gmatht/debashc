use crate::ast::*;
use crate::generator::Generator;

pub fn generate_xargs_command(generator: &mut Generator, cmd: &SimpleCommand, input_var: &str) -> String {
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
        output.push_str(&format!("my @files = split(/\\n/, {});\n", input_var));
        output.push_str("my @matching_files;\n");
        output.push_str("foreach my $file (@files) {\n");
        output.push_str("next unless $file && -f $file;\n");
        output.push_str("if (open(my $fh, '<', $file)) {\n");
        output.push_str("my $found = 0;\n");
        output.push_str("while (my $line = <$fh>) {\n");
        output.push_str("if ($line =~ /function/) {\n");
        output.push_str("$found = 1;\n");
        output.push_str("last;\n");
        output.push_str("}\n");
        output.push_str("}\n");
        output.push_str("close($fh);\n");
        output.push_str("push @matching_files, $file if $found;\n");
        output.push_str("}\n");
        output.push_str("}\n");
        output.push_str(&format!("{} = join(\"\\n\", @matching_files);\n", input_var));
    } else {
        // Fallback to system command for other cases
        output.push_str(&format!("{} = `echo \"${}\" | {}`;\n", input_var, input_var, command));
    }
    
    output
}
