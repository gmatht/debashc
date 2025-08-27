use crate::ast::*;
use crate::generator::Generator;

pub fn generate_wc_command(_generator: &mut Generator, cmd: &SimpleCommand, input_var: &str) -> String {
    let mut output = String::new();
    
    // wc command syntax: wc [options] [file]
    let mut count_lines = false;
    let mut count_words = false;
    let mut count_chars = false;
    let mut count_bytes = false;
    
    // Parse wc options
    for arg in &cmd.args {
        if let Word::Literal(arg_str) = arg {
            match arg_str.as_str() {
                "-l" => count_lines = true,
                "-w" => count_words = true,
                "-c" => count_chars = true,
                "-m" => count_bytes = true,
                _ => {}
            }
        }
    }
    
    // If no specific options, count all
    if !count_lines && !count_words && !count_chars && !count_bytes {
        count_lines = true;
        count_words = true;
        count_chars = true;
    }
    
    output.push_str(&format!("my @lines = split(/\\n/, {});\n", input_var));
    
    if count_lines {
        output.push_str("my $line_count = scalar(@lines);\n");
    }
    
    if count_words {
        output.push_str("my $word_count = 0;\n");
        output.push_str("foreach my $line (@lines) {\n");
        output.push_str("my @words = split(/\\s+/, $line);\n");
        output.push_str("$word_count += scalar(@words);\n");
        output.push_str("}\n");
    }
    
    if count_chars {
        output.push_str("my $char_count = length(join('', @lines));\n");
    }
    
    if count_bytes {
        output.push_str("my $byte_count = length(join('', @lines));\n");
    }
    
    // Format output
    output.push_str("my $result = '';\n");
    if count_lines {
        output.push_str("$result .= \"$line_count \";\n");
    }
    if count_words {
        output.push_str("$result .= \"$word_count \";\n");
    }
    if count_chars {
        output.push_str("$result .= \"$char_count \";\n");
    }
    if count_bytes {
        output.push_str("$result .= \"$byte_count \";\n");
    }
    output.push_str("$result =~ s/\\s+$//;\n"); // Remove trailing space
    output.push_str(&format!("{} = $result;\n", input_var));
    
    output
}
