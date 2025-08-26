use crate::ast::*;
use crate::generator::Generator;

pub fn generate_touch_command(generator: &mut Generator, cmd: &SimpleCommand) -> String {
    let mut output = String::new();
    
    // touch command syntax: touch [options] file...
    let mut files = Vec::new();
    
    // Parse touch options (currently just collecting files)
    for arg in &cmd.args {
        if let Word::Literal(arg_str) = arg {
            if !arg_str.starts_with('-') {
                files.push(generator.word_to_perl(arg));
            }
        } else {
            files.push(generator.word_to_perl(arg));
        }
    }
    
    if files.is_empty() {
        output.push_str("die \"touch: missing file operand\\n\";\n");
    } else {
        output.push_str("use POSIX qw(time);\n");
        
        for file in &files {
            output.push_str(&format!("if (-e {}) {{\n", file));
            // File exists, update timestamp
            output.push_str(&format!("my $current_time = time();\n"));
            output.push_str(&format!("utime($current_time, $current_time, {});\n", file));
            output.push_str(&format!("print \"touch: updated timestamp for {}\\n\";\n", file));
            output.push_str("} else {\n");
            // File doesn't exist, create it
            output.push_str(&format!("if (open(my $fh, '>', {})) {{\n", file));
            output.push_str("close($fh);\n");
            output.push_str(&format!("print \"touch: created file {}\\n\";\n", file));
            output.push_str("} else {\n");
            output.push_str(&format!("die \"touch: cannot create {}: $!\\n\";\n", file));
            output.push_str("}\n");
            output.push_str("}\n");
        }
    }
    
    output
}
