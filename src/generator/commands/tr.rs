use crate::ast::*;
use crate::generator::Generator;

pub fn generate_tr_command(generator: &mut Generator, cmd: &SimpleCommand, input_var: &str) -> String {
    let mut output = String::new();
    
    // tr command syntax: tr [options] set1 set2
    // For now, implement basic character translation
    if cmd.args.len() >= 2 {
        let set1 = generator.word_to_perl(&cmd.args[0]);
        let set2 = generator.word_to_perl(&cmd.args[1]);
        
        // Check for common tr options
        let mut delete_mode = false;
        let mut squeeze_mode = false;
        
        for arg in &cmd.args {
            if let Word::Literal(arg_str) = arg {
                if arg_str == "-d" {
                    delete_mode = true;
                } else if arg_str == "-s" {
                    squeeze_mode = true;
                }
            }
        }
        
        output.push_str(&format!("my @lines = split(/\\n/, {});\n", input_var));
        output.push_str("my @result;\n");
        output.push_str("foreach my $line (@lines) {\n");
        output.push_str("chomp($line);\n");
        
        if delete_mode {
            // Delete characters in set1
            output.push_str(&format!("$line =~ tr/{}/d;\n", set1));
        } else if squeeze_mode {
            // Squeeze repeated characters in set1
            output.push_str(&format!("$line =~ tr/{}/s;\n", set1));
        } else {
            // Translate characters from set1 to set2
            output.push_str(&format!("$line =~ tr/{}/{}/;\n", set1, set2));
        }
        
        output.push_str("push @result, $line;\n");
        output.push_str("}\n");
        output.push_str(&format!("{} = join(\"\\n\", @result);\n", input_var));
    } else {
        // Fallback for insufficient arguments
        output.push_str(&format!("{} = `echo \"${}\" | tr`;\n", input_var, input_var));
    }
    
    output
}
