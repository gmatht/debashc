use crate::ast::*;
use crate::generator::Generator;

pub fn generate_wget_command(generator: &mut Generator, cmd: &SimpleCommand) -> String {
    let mut output = String::new();
    
    // wget command syntax: wget [options] URL
    if let Some(url) = cmd.args.last() {
        let url_str = generator.word_to_perl(url);
        let mut output_file = "".to_string();
        
        // Parse wget options
        for arg in &cmd.args {
            if let Word::Literal(arg_str) = arg {
                if arg_str == "-O" {
                    // Output file option
                    if let Some(next_arg) = cmd.args.iter().find(|&a| a == arg) {
                        if let Some(idx) = cmd.args.iter().position(|a| a == arg) {
                            if idx + 1 < cmd.args.len() {
                                output_file = generator.word_to_perl(&cmd.args[idx + 1]);
                            }
                        }
                    }
                }
            }
        }
        
        output.push_str("use LWP::Simple;\n");
        output.push_str(&format!("my $url = {};\n", url_str));
        
        if !output_file.is_empty() {
            output.push_str(&format!("my $output_file = {};\n", output_file));
            output.push_str("my $content = get($url);\n");
            output.push_str("if (defined $content) {\n");
            output.push_str(&format!("open(my $fh, '>', $output_file) or die \"Cannot open $output_file: $!\";\n"));
            output.push_str("print $fh $content;\n");
            output.push_str("close($fh);\n");
            output.push_str("print \"Downloaded to $output_file\\n\";\n");
            output.push_str("} else {\n");
            output.push_str("die \"Failed to download $url\\n\";\n");
            output.push_str("}\n");
        } else {
            output.push_str("my $content = get($url);\n");
            output.push_str("if (defined $content) {\n");
            output.push_str("print $content;\n");
            output.push_str("} else {\n");
            output.push_str("die \"Failed to download $url\\n\";\n");
            output.push_str("}\n");
        }
    } else {
        output.push_str("die \"wget: missing URL\\n\";\n");
    }
    
    output
}
