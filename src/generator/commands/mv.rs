use crate::ast::*;
use crate::generator::Generator;

pub fn generate_mv_command(generator: &mut Generator, cmd: &SimpleCommand) -> String {
    let mut output = String::new();
    
    // mv command syntax: mv [options] source... destination
    let mut _force = false;
    let mut sources = Vec::new();
    let mut destination = "".to_string();
    
    // Parse mv options
    for arg in &cmd.args {
        if let Word::Literal(arg_str) = arg {
            match arg_str.as_str() {
                "-f" | "--force" => _force = true,
                _ => {
                    if !arg_str.starts_with('-') {
                        sources.push(generator.word_to_perl(arg));
                    }
                }
            }
        } else {
            sources.push(generator.word_to_perl(arg));
        }
    }
    
    if sources.len() < 2 {
        output.push_str("die \"mv: missing file operand\\n\";\n");
    } else {
        // Last argument is the destination
        destination = sources.pop().unwrap();
        
        output.push_str("use File::Copy qw(move);\n");
        output.push_str("use File::Path qw(make_path);\n");
        
        for source in &sources {
            output.push_str(&format!("if (-e {}) {{\n", source));
            
            // Check if destination exists and is a directory
            output.push_str(&format!("my $dest = {};\n", destination));
            output.push_str("if (-e $dest && -d $dest) {\n");
            // Destination is a directory, append source name
            output.push_str(&format!("$dest = \"$dest/{}\";\n", source));
            output.push_str("}\n");
            
            // Check if destination already exists
            output.push_str("if (-e $dest && !$force) {\n");
            output.push_str("die \"mv: $dest: File exists (use -f to force overwrite)\\n\";\n");
            output.push_str("}\n");
            
            // Create destination directory if it doesn't exist
            output.push_str("my $dest_dir = $dest;\n");
            output.push_str("$dest_dir =~ s/\\/[^\\/]*$//;\n");
            output.push_str("if ($dest_dir ne '' && !-d $dest_dir) {\n");
            output.push_str("make_path($dest_dir, {error => \\$err});\n");
            output.push_str("if (@$err) {\n");
            output.push_str("die \"mv: cannot create directory $dest_dir: $err->[0]\\n\";\n");
            output.push_str("}\n");
            output.push_str("}\n");
            
            // Perform the move
            output.push_str(&format!("if (move({}, $dest)) {{\n", source));
            output.push_str(&format!("print \"mv: moved {} to $dest\\n\";\n", source));
            output.push_str("} else {\n");
            output.push_str(&format!("die \"mv: cannot move {} to $dest: $!\\n\";\n", source));
            output.push_str("}\n");
            
            output.push_str("} else {\n");
            output.push_str(&format!("die \"mv: {}: No such file or directory\\n\";\n", source));
            output.push_str("}\n");
        }
    }
    
    output
}
