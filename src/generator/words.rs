use crate::ast::*;
use super::Generator;

pub fn word_to_perl_impl(generator: &mut Generator, word: &Word) -> String {
    match word {
        Word::Literal(s) => {
            // Handle literal strings
            if s.contains("..") {
                generator.handle_range_expansion(s)
            } else if s.contains(',') {
                generator.handle_comma_expansion(s)
            } else {
                s.clone()
            }
        },
        Word::ParameterExpansion(pe) => generator.generate_parameter_expansion(pe),
        Word::Array(name, elements) => {
            let elements_str = elements.iter()
                .map(|e| format!("'{}'", e.replace("'", "\\'")))
                .collect::<Vec<_>>()
                .join(", ");
            format!("@{} = ({});", name, elements_str)
        },
        Word::StringInterpolation(interp) => generator.convert_string_interpolation_to_perl(interp),
        Word::Arithmetic(expr) => generator.convert_arithmetic_to_perl(&expr.expression),
        Word::BraceExpansion(expansion) => generator.handle_brace_expansion(expansion),
        Word::CommandSubstitution(cmd) => {
            // Execute the command and capture its output
            // For command substitution, we need to generate Perl code that produces the same output
            // as our custom command implementations
            match cmd.as_ref() {
                Command::Simple(simple_cmd) => {
                    let cmd_name = generator.word_to_perl(&simple_cmd.name);
                    let args: Vec<String> = simple_cmd.args.iter()
                        .map(|arg| generator.word_to_perl(arg))
                        .collect();
                    
                    // Use our custom command implementations for consistency
                    if cmd_name == "ls" {
                        // Use the custom ls implementation for command substitution
                        use crate::generator::commands::ls::generate_ls_for_substitution;
                        generate_ls_for_substitution(generator, simple_cmd)
                    } else {
                        // For other commands, fall back to system command for now
                        if args.is_empty() {
                            format!("`{}`", cmd_name)
                        } else {
                            format!("`{} {}`", cmd_name, args.join(" "))
                        }
                    }
                },
                _ => {
                    // For other command types (like pipelines), generate the command
                    // and wrap it in a way that ensures proper variable scoping
                    let command_code = match &**cmd {
                        Command::Pipeline(pipeline) => {
                            // For pipelines in command substitution, don't print, just return the value
                            use crate::generator::commands::pipeline_commands::generate_pipeline_with_print_option;
                            generate_pipeline_with_print_option(generator, pipeline, false)
                        },
                        _ => generator.generate_command(cmd)
                    };
                    // For pipelines and complex commands, we need to ensure proper variable scoping
                    // by wrapping in a do block and capturing the output.
                    // The pipeline generation now handles streaming without buffering.
                    format!("do {{\n{}}}", command_code.trim_end_matches('\n'))
                }
            }
        },
        Word::Variable(var) => {
            // Handle special shell variables
            match var.as_str() {
                "#" => "scalar(@ARGV)".to_string(),  // $# -> scalar(@ARGV) for argument count
                "@" => "@ARGV".to_string(),          // $@ -> @ARGV for arguments array
                "*" => "@ARGV".to_string(),          // $* -> @ARGV for arguments array
                _ => format!("${}", var)             // Regular variable
            }
        },
        _ => format!("{:?}", word)
    }
}

pub fn word_to_perl_for_test_impl(generator: &mut Generator, word: &Word) -> String {
    match word {
        Word::Literal(s) => s.clone(),
        Word::ParameterExpansion(pe) => generator.generate_parameter_expansion(pe),
        _ => format!("{:?}", word)
    }
}

// Helper methods
pub fn handle_range_expansion_impl(_generator: &Generator, s: &str) -> String {
    let parts: Vec<&str> = s.split("..").collect();
    if parts.len() == 2 {
        if let (Ok(start), Ok(end)) = (parts[0].parse::<i64>(), parts[1].parse::<i64>()) {
            let values: Vec<String> = (start..=end)
                .map(|i| i.to_string())
                .collect();
            // Format as Perl array: (1, 2, 3, 4, 5)
            format!("({})", values.join(", "))
        } else {
            s.to_string()
        }
    } else {
        s.to_string()
    }
}

pub fn handle_comma_expansion_impl(_generator: &Generator, s: &str) -> String {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() > 1 {
        parts.join(" ")
    } else {
        s.to_string()
    }
}

pub fn handle_brace_expansion_impl(generator: &mut Generator, expansion: &BraceExpansion) -> String {
    if expansion.items.len() == 1 {
        generator.word_to_perl(&generator.brace_item_to_word(&expansion.items[0]))
    } else {
        let items: Vec<String> = expansion.items.iter()
            .map(|item| generator.word_to_perl(&generator.brace_item_to_word(item)))
            .collect();
        items.join(" ")
    }
}

pub fn brace_item_to_word_impl(_generator: &Generator, item: &BraceItem) -> Word {
    match item {
        BraceItem::Literal(s) => Word::Literal(s.clone()),
        BraceItem::Range(range) => Word::Literal(format!("{}..{}", range.start, range.end)),
        BraceItem::Sequence(seq) => Word::Literal(seq.join(" ")),
    }
}

pub fn convert_string_interpolation_to_perl_impl(generator: &Generator, interp: &StringInterpolation) -> String {
    // Convert string interpolation to a single Perl interpolated string
    let mut combined_string = String::new();
    
    for part in &interp.parts {
        match part {
            StringPart::Literal(s) => {
                // Add the literal text directly to the interpolated string
                combined_string.push_str(s);
            },
            StringPart::Variable(var) => {
                // Handle special shell variables
                match var.as_str() {
                    "#" => combined_string.push_str("scalar(@ARGV)"),  // $# -> scalar(@ARGV) for argument count
                    "@" => combined_string.push_str("@ARGV"),          // Arrays don't need $ in interpolation
                    "*" => combined_string.push_str("@ARGV"),          // Arrays don't need $ in interpolation
                    _ => {
                        // Check if this is a shell positional parameter ($1, $2, etc.)
                        if var.chars().all(|c| c.is_digit(10)) {
                            // Convert $1 to $_[0], $2 to $_[1], etc.
                            let index = var.parse::<usize>().unwrap_or(0);
                            combined_string.push_str(&format!("$_[{}]", index - 1)); // Perl arrays are 0-indexed
                        } else {
                            // Regular variable - add directly for interpolation
                            combined_string.push_str(&format!("${}", var));
                        }
                    }
                }
            },
            StringPart::MapAccess(map_name, key) => {
                if map_name == "map" {
                    combined_string.push_str(&format!("$map{{{}}}", key));
                } else {
                    combined_string.push_str(&format!("${}{{{}}}", map_name, key));
                }
            }
            _ => {
                // Handle other StringPart variants by converting them to debug format for now
                combined_string.push_str(&format!("{:?}", part));
            }
        }
    }
    
    // Return as a single interpolated string
    format!("\"{}\"", combined_string)
}

pub fn convert_arithmetic_to_perl_impl(_generator: &Generator, expr: &str) -> String {
    // Convert shell arithmetic expression to Perl syntax
    let result = expr.to_string();
    
    // Convert shell variables to Perl variables (e.g., i -> $i)
    // This is a simple regex-based approach - in practice, the parser should handle this better
    
    // Split the expression into parts and convert each part
    let parts: Vec<&str> = result.split_whitespace().collect();
    let converted_parts: Vec<String> = parts.iter().map(|part| {
        // Check if this part looks like a variable (starts with a letter and contains only alphanumeric chars)
        if part.chars().next().map_or(false, |c| c.is_alphabetic()) && 
           part.chars().all(|c| c.is_alphanumeric() || c == '_') {
            // This looks like a variable, prefix with $
            format!("${}", part)
        } else {
            // This is an operator, number, or already formatted variable
            part.to_string()
        }
    }).collect();
    
    // Rejoin the parts
    converted_parts.join(" ")
}
