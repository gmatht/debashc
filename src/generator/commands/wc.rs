use crate::ast::*;
use crate::generator::Generator;

pub fn generate_wc_command(_generator: &mut Generator, cmd: &SimpleCommand, input_var: &str, command_index: usize) -> String {
    let mut output = String::new();
    
    // Parse wc options
    let mut count_lines = false;
    let mut count_words = false;
    let mut count_chars = false;
    let mut count_bytes = false;
    
    for arg in &cmd.args {
        if let Word::Literal(s) = arg {
            if s.starts_with('-') {
                if s.contains('l') { count_lines = true; }
                if s.contains('w') { count_words = true; }
                if s.contains('c') { count_chars = true; }
                if s.contains('m') { count_chars = true; } // -m is equivalent to -c for character count
            }
        }
    }
    
    // If no specific options, count all
    if !count_lines && !count_words && !count_chars && !count_bytes {
        count_lines = true;
        count_words = true;
        count_chars = true;
        count_bytes = true;
    }
    
    // Generate Perl code for wc
    output.push_str(&format!("my @wc_lines_{} = split(/\\n/, {});\n", command_index, input_var));
    
    if count_lines {
        output.push_str(&format!("my $wc_line_count_{} = scalar(@wc_lines_{});\n", command_index, command_index));
    }
    
    if count_words {
        output.push_str(&format!("my $wc_word_count_{} = 0;\n", command_index));
        output.push_str(&format!("foreach my $line (@wc_lines_{}) {{\n", command_index));
        output.push_str(&format!("    my @wc_words_{} = split(/\\s+/, $line);\n", command_index));
        output.push_str(&format!("    $wc_word_count_{} += scalar(@wc_words_{});\n", command_index, command_index));
        output.push_str("}\n");
    }
    
    if count_chars {
        output.push_str(&format!("my $wc_char_count_{} = length(join('', @wc_lines_{}));\n", command_index, command_index));
    }
    
    if count_bytes {
        output.push_str(&format!("my $wc_byte_count_{} = length(join('', @wc_lines_{}));\n", command_index, command_index));
    }
    
    // Format output
    output.push_str(&format!("my $wc_result_{} = '';\n", command_index));
    if count_lines {
        output.push_str(&format!("$wc_result_{} .= \"$wc_line_count_{} \";\n", command_index, command_index));
    }
    if count_words {
        output.push_str(&format!("$wc_result_{} .= \"$wc_word_count_{} \";\n", command_index, command_index));
    }
    if count_chars {
        output.push_str(&format!("$wc_result_{} .= \"$wc_char_count_{} \";\n", command_index, command_index));
    }
    if count_bytes {
        output.push_str(&format!("$wc_result_{} .= \"$wc_byte_count_{} \";\n", command_index, command_index));
    }
    output.push_str(&format!("$wc_result_{} =~ s/\\s+$//;\n", command_index)); // Remove trailing space
    output.push_str(&format!("{} = $wc_result_{};\n", input_var, command_index));
    
    output
}
