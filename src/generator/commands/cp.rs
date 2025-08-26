use crate::ast::*;
use crate::generator::Generator;

pub fn generate_cp_command(generator: &mut Generator, cmd: &SimpleCommand) -> String {
    let mut output = String::new();
    
    // cp command syntax: cp [options] source... destination
    let mut recursive = false;
    let mut preserve = false;
    let mut sources = Vec::new();
    let mut destination = "".to_string();
    
    // Parse cp options
    for arg in &cmd.args {
        if let Word::Literal(arg_str) = arg {
            match arg_str.as_str() {
                "-r" | "-R" | "--recursive" => recursive = true,
                "-p" | "--preserve" => preserve = true,
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
        output.push_str("die \"cp: missing file operand\\n\";\n");
    } else {
        // Last argument is the destination
        destination = sources.pop().unwrap();
        
        output.push_str("use File::Copy qw(copy);\n");
        output.push_str("use File::Path qw(make_path);\n");
        
        for source in &sources {
            output.push_str(&format!("if (-e {}) {{\n", source));
            
            if recursive && format!("-d {}", source).contains("-d") {
                // Recursive copy for directories
                output.push_str(&format!("if (-d {}) {{\n", source));
                output.push_str(&format!("my $dest_dir = {};\n", destination));
                output.push_str("if (-e $dest_dir && !-d $dest_dir) {\n");
                output.push_str("die \"cp: $dest_dir: not a directory\\n\";\n");
                output.push_str("}\n");
                output.push_str("if (!-d $dest_dir) {\n");
                output.push_str("make_path($dest_dir, {error => \\$err});\n");
                output.push_str("if (@$err) {\n");
                output.push_str(&format!("die \"cp: cannot create directory $dest_dir: $err->[0]\\n\";\n"));
                output.push_str("}\n");
                output.push_str("}\n");
                output.push_str(&format!("my $cmd = \"cp -r {} $dest_dir\";\n", source));
                output.push_str("my $result = system($cmd);\n");
                output.push_str("if ($result == 0) {\n");
                output.push_str(&format!("print \"cp: copied directory {} to $dest_dir\\n\";\n", source));
                output.push_str("} else {\n");
                output.push_str(&format!("die \"cp: failed to copy directory {}\\n\";\n", source));
                output.push_str("}\n");
                output.push_str("} else {\n");
                // Copy single file
                output.push_str(&format!("my $dest = {};\n", destination));
                output.push_str("if (-d $dest) {\n");
                output.push_str(&format!("$dest = \"$dest/{}\";\n", source));
                output.push_str("}\n");
                output.push_str(&format!("if (copy({}, $dest)) {{\n", source));
                if preserve {
                    output.push_str("my ($atime, $mtime) = (stat($source))[8,9];\n");
                    output.push_str("utime($atime, $mtime, $dest);\n");
                }
                output.push_str(&format!("print \"cp: copied {} to $dest\\n\";\n", source));
                output.push_str("} else {\n");
                output.push_str(&format!("die \"cp: cannot copy {} to $dest: $!\\n\";\n", source));
                output.push_str("}\n");
                output.push_str("}\n");
            } else {
                // Copy single file
                output.push_str(&format!("my $dest = {};\n", destination));
                output.push_str("if (-d $dest) {\n");
                output.push_str(&format!("$dest = \"$dest/{}\";\n", source));
                output.push_str("}\n");
                output.push_str(&format!("if (copy({}, $dest)) {{\n", source));
                if preserve {
                    output.push_str("my ($atime, $mtime) = (stat($source))[8,9];\n");
                    output.push_str("utime($atime, $mtime, $dest);\n");
                }
                output.push_str(&format!("print \"cp: copied {} to $dest\\n\";\n", source));
                output.push_str("} else {\n");
                output.push_str(&format!("die \"cp: cannot copy {} to $dest: $!\\n\";\n", source));
                output.push_str("}\n");
                output.push_str("}\n");
            }
            
            output.push_str("} else {\n");
            output.push_str(&format!("die \"cp: {}: No such file or directory\\n\";\n", source));
            output.push_str("}\n");
        }
    }
    
    output
}
