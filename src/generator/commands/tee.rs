use crate::ast::*;
use crate::generator::Generator;

pub fn generate_tee_command(generator: &mut Generator, cmd: &SimpleCommand, input_var: &str) -> String {
    let mut output = String::new();
    
    // tee command syntax: tee [options] file
    let mut append_mode = false;
    let mut files = Vec::new();
    
    // Parse tee options
    for arg in &cmd.args {
        if let Word::Literal(arg_str) = arg {
            if arg_str == "-a" {
                append_mode = true;
            } else if !arg_str.starts_with('-') {
                files.push(generator.word_to_perl(arg));
            }
        } else {
            files.push(generator.word_to_perl(arg));
        }
    }
    
    if files.is_empty() {
        // No files specified, just pass through
        output.push_str(&format!("{} = {};\n", input_var, input_var));
    } else {
        // Write to specified files
        output.push_str(&format!("my @lines = split(/\\n/, {});\n", input_var));
        
        for file in &files {
            let mode = if append_mode { ">>" } else { ">" };
            output.push_str(&format!("if (open(my $fh, '{}', {})) {{\n", mode, file));
            output.push_str("foreach my $line (@lines) {\n");
            output.push_str("print $fh \"$line\\n\";\n");
            output.push_str("}\n");
            output.push_str("close($fh);\n");
            output.push_str("} else {\n");
            output.push_str(&format!("warn \"tee: Cannot open {}: $!\";\n", file));
            output.push_str("}\n");
        }
        
        // Keep the output for further processing
        output.push_str(&format!("{} = join(\"\\n\", @lines);\n", input_var));
    }
    
    output
}
