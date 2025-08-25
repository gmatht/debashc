use crate::ast::*;
use super::PerlGenerator;

impl PerlGenerator {
    pub fn get_unique_file_handle(&mut self) -> String {
        self.file_handle_counter += 1;
        format!("$fh_{}", self.file_handle_counter)
    }

    pub fn get_unique_dir_handle(&mut self) -> String {
        self.file_handle_counter += 1;
        format!("$dh_{}", self.file_handle_counter)
    }

    pub fn extract_array_key<'a>(&self, var: &'a str) -> Option<(&'a str, &'a str)> {
        if var.contains('[') && var.ends_with(']') {
            var.find('[').map(|bracket_start| {
                let array_name = &var[..bracket_start];
                let key = &var[bracket_start + 1..var.len() - 1];
                (array_name, key)
            })
        } else {
            None
        }
    }

    pub fn extract_array_elements<'a>(&self, value: &'a Word) -> Option<Vec<&'a str>> {
        match value {
            Word::Array(_, elements) => Some(elements.iter().map(|s| s.as_str()).collect()),
            Word::Literal(literal) if literal.starts_with('(') && literal.ends_with(')') => {
                let content = &literal[1..literal.len()-1];
                Some(content.split_whitespace().collect())
            }
            _ => None
        }
    }

    pub fn extract_simple_pattern(&mut self, word: &Word) -> String {
        match word {
            Word::Literal(s) => {
                if s.contains('*') || s.contains('?') || s.contains('[') {
                    // This is a glob pattern, convert to regex
                    let mut pattern = s.to_string();
                    // Escape regex special characters
                    pattern = pattern.replace("\\", "\\\\");
                    pattern = pattern.replace(".", "\\.");
                    pattern = pattern.replace("+", "\\+");
                    pattern = pattern.replace("|", "\\|");
                    pattern = pattern.replace("(", "\\(");
                    pattern = pattern.replace(")", "\\)");
                    pattern = pattern.replace("[", "\\[");
                    pattern = pattern.replace("]", "\\]");
                    pattern = pattern.replace("^", "\\^");
                    pattern = pattern.replace("$", "\\$");
                    // Convert glob patterns to regex
                    pattern = pattern.replace("*", ".*");
                    pattern = pattern.replace("?", ".");
                    format!("qr/^{}$/", pattern)
                } else {
                    format!("qr/^{}$/", regex::escape(s))
                }
            }
            Word::Array(_, elements) => {
                let patterns: Vec<String> = elements.iter().map(|e| self.extract_simple_pattern(&Word::Literal(e.clone()))).collect();
                format!("qr/({})/", patterns.join("|"))
            }
            _ => "qr//".to_string()
        }
    }

    pub fn perl_string_literal(&self, word: &Word) -> String {
        match word {
            Word::Literal(s) => {
                if s.contains('\'') || s.contains('"') || s.contains('\\') {
                    // Use qq{} for strings with quotes
                    let escaped = s.replace("\\", "\\\\")
                                   .replace("}", "\\}")
                                   .replace("$", "\\$");
                    format!("qq{{{}}}", escaped)
                } else {
                    format!("'{}'", s)
                }
            }
            Word::Array(_, elements) => {
                let elements_str: Vec<String> = elements.iter().map(|e| self.perl_string_literal(&Word::Literal(e.clone()))).collect();
                format!("({})", elements_str.join(", "))
            }
            _ => "''".to_string()
        }
    }

    pub fn handle_control_char_literal(&self, s: &str) -> String {
        let mut result = String::new();
        let mut chars = s.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '\\' {
                if let Some(&next_ch) = chars.peek() {
                    match next_ch {
                        'n' => { result.push_str("\\n"); chars.next(); }
                        't' => { result.push_str("\\t"); chars.next(); }
                        'r' => { result.push_str("\\r"); chars.next(); }
                        '\\' => { result.push_str("\\\\"); chars.next(); }
                        '"' => { result.push_str("\\\""); chars.next(); }
                        '\'' => { result.push_str("\\'"); chars.next(); }
                        _ => result.push(ch)
                    }
                } else {
                    result.push(ch);
                }
            } else {
                result.push(ch);
            }
        }
        
        result
    }
}
