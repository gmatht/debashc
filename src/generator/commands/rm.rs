use crate::ast::*;
use crate::generator::Generator;

pub fn generate_rm_command(generator: &mut Generator, cmd: &SimpleCommand) -> String {
    let mut output = String::new();
    
    // rm command syntax: rm [options] file...
    let mut recursive = false;
    let mut force = false;
    let mut files = Vec::new();
    
    // Parse rm options
    for arg in &cmd.args {
        if let Word::Literal(arg_str) = arg {
            match arg_str.as_str() {
                "-r" | "-R" | "--recursive" => recursive = true,
                "-f" | "--force" => force = true,
                _ => {
                    if !arg_str.starts_with('-') {
                        files.push(generator.word_to_perl(arg));
                    }
                }
            }
        } else {
            files.push(generator.word_to_perl(arg));
        }
    }
    
    if files.is_empty() {
        output.push_str("die \"rm: missing operand\\n\";\n");
    } else {
        output.push_str("use File::Path qw(remove_tree);\n");
        
        for file in &files {
            output.push_str(&format!("if (-e {}) {{\n", file));
            
            if recursive {
                // Recursive removal
                output.push_str(&format!("if (-d {}) {{\n", file));
                output.push_str(&format!("remove_tree({}, {{error => \\$err}});\n", file));
                output.push_str("if (@$err) {\n");
                if force {
                    output.push_str(&format!("warn \"rm: warning: could not remove {}: $err->[0]\\n\";\n", file));
                } else {
                    output.push_str(&format!("die \"rm: cannot remove {}: $err->[0]\\n\";\n", file));
                }
                output.push_str("} else {\n");
                output.push_str(&format!("print \"rm: removed directory {}\\n\";\n", file));
                output.push_str("}\n");
                output.push_str("} else {\n");
                // File removal
                output.push_str(&format!("if (unlink({})) {{\n", file));
                output.push_str(&format!("print \"rm: removed file {}\\n\";\n", file));
                output.push_str("} else {\n");
                if force {
                    output.push_str(&format!("warn \"rm: warning: could not remove {}: $!\\n\";\n", file));
                } else {
                    output.push_str(&format!("die \"rm: cannot remove {}: $!\\n\";\n", file));
                }
                output.push_str("}\n");
                output.push_str("}\n");
            } else {
                // Non-recursive removal
                output.push_str(&format!("if (-d {}) {{\n", file));
                if force {
                    output.push_str(&format!("warn \"rm: warning: {} is a directory (use -r to remove recursively)\\n\";\n", file));
                } else {
                    output.push_str(&format!("die \"rm: {} is a directory (use -r to remove recursively)\\n\";\n", file));
                }
                output.push_str("} else {\n");
                output.push_str(&format!("if (unlink({})) {{\n", file));
                output.push_str(&format!("print \"rm: removed file {}\\n\";\n", file));
                output.push_str("} else {\n");
                if force {
                    output.push_str(&format!("warn \"rm: warning: could not remove {}: $!\\n\";\n", file));
                } else {
                    output.push_str(&format!("die \"rm: cannot remove {}: $!\\n\";\n", file));
                }
                output.push_str("}\n");
                output.push_str("}\n");
            }
            
            output.push_str("} else {\n");
            if force {
                output.push_str(&format!("warn \"rm: warning: {}: No such file or directory\\n\";\n", file));
            } else {
                output.push_str(&format!("die \"rm: {}: No such file or directory\\n\";\n", file));
            }
            output.push_str("}\n");
        }
    }
    
    output
}
