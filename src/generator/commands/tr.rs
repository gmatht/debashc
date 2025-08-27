use crate::ast::*;
use crate::generator::Generator;

fn extract_character_set_from_word(generator: &mut Generator, word: &Word) -> String {
    let raw_pattern = match word {
        Word::StringInterpolation(interp) => {
            // Extract the literal content without quotes
            interp.parts.iter()
                .map(|part| match part {
                    crate::ast::StringPart::Literal(s) => s.clone(),
                    _ => String::new()
                })
                .collect::<Vec<_>>()
                .join("")
        }
        Word::Literal(s) => s.clone(),
        _ => generator.word_to_perl(word)
    };
    
    // For tr operator, use the raw pattern as-is for now
    raw_pattern
}

pub fn generate_tr_command(generator: &mut Generator, cmd: &SimpleCommand, input_var: &str) -> String {
    let mut output = String::new();
    
    // tr command syntax: tr [options] set1 set2
    // For now, implement basic character translation
    if cmd.args.len() >= 2 {
        // Check for common tr options
        let mut delete_mode = false;
        let mut squeeze_mode = false;
        let mut char_set = String::new();
        
        for arg in &cmd.args {
            match arg {
                Word::Literal(arg_str) => {
                    if arg_str == "-d" {
                        delete_mode = true;
                    } else if arg_str == "-s" {
                        squeeze_mode = true;
                    } else if !arg_str.starts_with('-') {
                        // This is the character set, not an option
                        char_set = extract_character_set_from_word(generator, arg);
                    }
                }
                _ => {
                    // Handle other word types (like StringInterpolation)
                    if !delete_mode && !squeeze_mode {
                        // If we haven't seen options yet, this might be the character set
                        char_set = extract_character_set_from_word(generator, arg);
                    } else if char_set.is_empty() {
                        // If we've seen options but no character set yet, this is it
                        char_set = extract_character_set_from_word(generator, arg);
                    }
                }
            }
        }
        
        output.push_str(&format!("my @lines = split(/\\n/, {});\n", input_var));
        output.push_str("my @result;\n");
        output.push_str("foreach my $line (@lines) {\n");
        output.push_str("chomp($line);\n");
        
        if delete_mode {
            // Delete characters in char_set - use | as delimiter to avoid conflicts with /
            output.push_str(&format!("$line =~ tr|{}||d;\n", char_set));
            output.push_str("push @result, $line;\n");
        } else if squeeze_mode {
            // Squeeze repeated characters in char_set - use | as delimiter to avoid conflicts with /
            output.push_str(&format!("$line =~ tr|{}||s;\n", char_set));
            output.push_str("push @result, $line;\n");
        } else {
            // For now, just pass through if no options (would need two char sets)
            output.push_str("push @result, $line;\n");
        }
        
        output.push_str("}\n");
        output.push_str(&format!("{} = join(\"\\n\", @result);\n", input_var));
    } else {
        // Fallback for insufficient arguments
        output.push_str(&format!("{} = `echo \"${}\" | tr`;\n", input_var, input_var));
    }
    output.push_str("\n");
    
    output
}
