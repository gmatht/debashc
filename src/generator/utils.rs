use crate::ast::*;
use super::Generator;

pub fn extract_array_key_impl(var: &str) -> Option<(String, String)> {
    // Check if this is an associative array assignment like map[foo]=bar
    if let Some(bracket_start) = var.find('[') {
        if let Some(bracket_end) = var.find(']') {
            if bracket_start < bracket_end {
                let array_name = var[..bracket_start].to_string();
                let key = var[bracket_start + 1..bracket_end].to_string();
                return Some((array_name, key));
            }
        }
    }
    None
}

pub fn extract_array_elements_impl(value: &str) -> Option<Vec<String>> {
    // Check if this is an indexed array assignment like arr=(one two three)
    if value.starts_with('(') && value.ends_with(')') {
        let content = &value[1..value.len() - 1];
        if !content.is_empty() {
            let elements: Vec<String> = content
                .split_whitespace()
                .map(|s| s.to_string())
                .collect();
            return Some(elements);
        }
    }
    None
}

pub fn perl_string_literal_impl(generator: &mut Generator, word: &Word) -> String {
    match word {
        Word::Literal(s) => {
            // Escape quotes and backslashes for Perl string literals
            let escaped = s.replace("\\", "\\\\")
                          .replace("\"", "\\\"")
                          .replace("\n", "\\n")
                          .replace("\t", "\\t")
                          .replace("\r", "\\r");
            format!("\"{}\"", escaped)
        }
        Word::Arithmetic(expr) => {
            // Handle arithmetic expressions by converting them to Perl
            generator.convert_arithmetic_to_perl(&expr.expression)
        }
        Word::ParameterExpansion(pe) => {
            // Handle parameter expansion
            generator.generate_parameter_expansion(pe)
        }
        _ => format!("{:?}", word)
    }
}

pub fn get_unique_file_handle_impl(generator: &mut Generator) -> String {
    generator.file_handle_counter += 1;
    format!("fh_{}", generator.file_handle_counter)
}
