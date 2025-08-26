use crate::ast::*;
use crate::generator::Generator;

pub fn generate_mkdir_command(generator: &mut Generator, cmd: &SimpleCommand) -> String {
    let mut output = String::new();
    
    // mkdir command syntax: mkdir [options] directory...
    let mut create_parents = false;
    let mut directories = Vec::new();
    
    // Parse mkdir options
    for arg in &cmd.args {
        if let Word::Literal(arg_str) = arg {
            match arg_str.as_str() {
                "-p" | "--parents" => create_parents = true,
                _ => {
                    if !arg_str.starts_with('-') {
                        directories.push(generator.word_to_perl(arg));
                    }
                }
            }
        } else {
            directories.push(generator.word_to_perl(arg));
        }
    }
    
    if directories.is_empty() {
        output.push_str("die \"mkdir: missing operand\\n\";\n");
    } else {
        output.push_str("use File::Path qw(make_path);\n");
        
        for dir in &directories {
            if create_parents {
                output.push_str(&format!("if (!-d {}) {{\n", dir));
                output.push_str(&format!("make_path({}, {{error => \\$err}});\n", dir));
                output.push_str("if (@$err) {\n");
                output.push_str(&format!("die \"mkdir: cannot create directory {}: $err->[0]\\n\";\n", dir));
                output.push_str("} else {\n");
                output.push_str(&format!("print \"mkdir: created directory {}\\n\";\n", dir));
                output.push_str("}\n");
                output.push_str("} else {\n");
                output.push_str(&format!("print \"mkdir: directory {} already exists\\n\";\n", dir));
                output.push_str("}\n");
            } else {
                output.push_str(&format!("if (!-d {}) {{\n", dir));
                output.push_str(&format!("if (mkdir({})) {{\n", dir));
                output.push_str(&format!("print \"mkdir: created directory {}\\n\";\n", dir));
                output.push_str("} else {\n");
                output.push_str(&format!("die \"mkdir: cannot create directory {}: $!\\n\";\n", dir));
                output.push_str("}\n");
                output.push_str("} else {\n");
                output.push_str(&format!("die \"mkdir: cannot create directory {}: File exists\\n\";\n", dir));
                output.push_str("}\n");
            }
        }
    }
    
    output
}
