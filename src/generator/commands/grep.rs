use crate::generator::Generator;
use crate::ast::*;

pub fn generate_grep_command(generator: &mut Generator, cmd: &SimpleCommand, input_var: &str, command_index: usize) -> String {
    let mut output = String::new();
    
    // Parse grep options and pattern
    let mut pattern = String::new();
    let mut count_only = false;
    let mut line_numbers = false;
    let mut ignore_case = false;
    let mut invert_match = false;
    let mut word_match = false;
    
    for arg in &cmd.args {
        if let Word::Literal(s) = arg {
            if s.starts_with('-') {
                if s.contains('c') { count_only = true; }
                if s.contains('n') { line_numbers = true; }
                if s.contains('i') { ignore_case = true; }
                if s.contains('v') { invert_match = true; }
                if s.contains('w') { word_match = true; }
            } else {
                pattern = s.clone();
            }
        } else {
            pattern = generator.word_to_perl(arg);
        }
    }
    
    if pattern.is_empty() {
        // No pattern provided, return error
        output.push_str("warn \"grep: no pattern specified\";\n");
        output.push_str("exit(1);\n");
    } else {
        // Split input into lines and apply grep logic
        output.push_str(&format!("my @grep_lines_{} = split(/\\n/, {});\n", command_index, input_var));
        
        // Escape the pattern for Perl regex
        // For patterns like "\.txt$", the backslash is escaping the dot in shell
        // In Perl, we need to escape it properly
        let escaped_pattern = pattern.to_string();
        // Remove quotes if they exist around the pattern
        let regex_pattern = if escaped_pattern.starts_with('"') && escaped_pattern.ends_with('"') {
            &escaped_pattern[1..escaped_pattern.len()-1]
        } else {
            &escaped_pattern
        };
        if invert_match {
            // Negative grep: exclude lines that match the pattern
            output.push_str(&format!("my @grep_filtered_{} = grep !/{}/, @grep_lines_{};\n", command_index, regex_pattern, command_index));
        } else {
            // Positive grep: include lines that match the pattern
            output.push_str(&format!("my @grep_filtered_{} = grep /{}/, @grep_lines_{};\n", command_index, regex_pattern, command_index));
        }
        
        if count_only {
            output.push_str(&format!("{} = scalar(@grep_filtered_{});\n", input_var, command_index));
        } else if line_numbers {
            output.push_str(&format!("my @grep_numbered_{};\n", command_index));
            output.push_str(&format!("for (my $i = 0; $i < @grep_lines_{}; $i++) {{\n", command_index));
            output.push_str(&format!("    if (grep {{ $_ eq $grep_lines_{}[$i] }} @grep_filtered_{}) {{\n", command_index, command_index));
            output.push_str(&format!("        push @grep_numbered_{}, sprintf(\"%d:%s\", $i + 1, $grep_lines_{}[$i]);\n", command_index, command_index));
            output.push_str("    }\n");
            output.push_str("}\n");
            output.push_str(&format!("{} = join(\"\\n\", @grep_numbered_{});\n", input_var, command_index));
        } else {
            output.push_str(&format!("{} = join(\"\\n\", @grep_filtered_{});\n", input_var, command_index));
        }
    }
    
    output
}
