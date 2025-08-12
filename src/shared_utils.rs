// Removed unused import: use crate::ast::*;

/// Shared utilities for shell script generators
pub struct SharedUtils;

impl SharedUtils {
    /// Parse numeric brace range like {1..5} or {10..3}
    pub fn parse_numeric_brace_range(s: &str) -> Option<(i64, i64)> {
        if !(s.starts_with('{') && s.ends_with('}')) {
            return None;
        }
        let inner = &s[1..s.len() - 1];
        let parts: Vec<&str> = inner.split("..").collect();
        if parts.len() != 2 {
            return None;
        }
        let start = parts[0].parse::<i64>().ok()?;
        let end = parts[1].parse::<i64>().ok()?;
        Some((start, end))
    }

    /// Parse seq command like `seq A B` or $(seq A B) or plain seq A B
    pub fn parse_seq_command(s: &str) -> Option<(i64, i64)> {
        let trimmed = s.trim();
        // Strip backticks or $( )
        let inner = if trimmed.starts_with('`') && trimmed.ends_with('`') {
            &trimmed[1..trimmed.len()-1]
        } else if trimmed.starts_with("$(") && trimmed.ends_with(')') {
            &trimmed[2..trimmed.len()-1]
        } else {
            trimmed
        };

        let parts: Vec<&str> = inner.split_whitespace().collect();
        if parts.len() == 3 && parts[0] == "seq" {
            let start = parts[1].parse::<i64>().ok()?;
            let end = parts[2].parse::<i64>().ok()?;
            return Some((start, end));
        }
        None
    }

    /// Convert glob pattern to regex pattern
    pub fn convert_glob_to_regex(pattern: &str) -> String {
        let mut result = String::new();
        let mut chars = pattern.chars().peekable();
        
        while let Some(ch) = chars.next() {
            match ch {
                '.' => result.push_str("\\."),
                '*' => result.push_str(".*"),
                '?' => result.push_str("."),
                '[' => {
                    result.push(ch);
                    // Handle character classes
                    while let Some(&next_ch) = chars.peek() {
                        if next_ch == ']' {
                            result.push(chars.next().unwrap());
                            break;
                        }
                        result.push(chars.next().unwrap());
                    }
                }
                ']' => result.push(ch),
                '(' => result.push_str("\\("),
                ')' => result.push_str("\\)"),
                '|' => result.push_str("\\|"),
                '^' => result.push_str("\\^"),
                '$' => result.push_str("\\$"),
                '+' => result.push_str("\\+"),
                '\\' => result.push_str("\\\\"),
                _ => result.push(ch),
            }
        }
        
        result
    }

    /// Convert extended glob pattern to regex
    pub fn convert_extglob_to_regex(pattern: &str) -> String {
        // Handle extended glob patterns like +(pattern) or *(pattern)
        if pattern.contains('(') && pattern.contains(')') {
            let mut result = String::new();
            let mut chars = pattern.chars().peekable();
            
            while let Some(ch) = chars.next() {
                match ch {
                    '+' => result.push_str("(?:.*)?"), // One or more
                    '*' => result.push_str("(?:.*)?"), // Zero or more
                    '?' => result.push_str("(?:.*)?"), // Zero or one
                    '@' => result.push_str("(?:.*)?"), // Exactly one
                    '!' => result.push_str("(?!.*)"),  // Negation
                    '(' => result.push_str("(?:.*)?"), // Group
                    ')' => result.push_str(")"),
                    _ => result.push(ch),
                }
            }
            result
        } else {
            Self::convert_glob_to_regex(pattern)
        }
    }

    /// Expand brace expression like {1..5} or {a..c}
    pub fn expand_brace_expression(expr: &str) -> Option<Vec<String>> {
        // Handle simple numeric ranges like {1..5}
        if let Some(range) = expr.strip_prefix('{').and_then(|s| s.strip_suffix('}')) {
            if let Some((start, end)) = range.split_once("..") {
                if let (Ok(start_num), Ok(end_num)) = (start.parse::<i64>(), end.parse::<i64>()) {
                    let mut result = Vec::new();
                    for i in start_num..=end_num {
                        result.push(i.to_string());
                    }
                    return Some(result);
                }
            }
            
            // Handle alphabetic ranges like {a..c}
            if let Some((start, end)) = range.split_once("..") {
                if start.len() == 1 && end.len() == 1 {
                    if let (Some(start_char), Some(end_char)) = (start.chars().next(), end.chars().next()) {
                        if start_char.is_ascii_lowercase() && end_char.is_ascii_lowercase() {
                            let mut result = Vec::new();
                            for c in start_char..=end_char {
                                result.push(c.to_string());
                            }
                            return Some(result);
                        }
                    }
                }
            }
            
            // Handle step ranges like {00..04..2}
            if let Some((range_part, step_part)) = range.split_once("..") {
                if let Some((start, end)) = range_part.split_once("..") {
                    if let (Ok(start_num), Ok(end_num), Ok(step)) = (
                        start.parse::<i64>(), 
                        end.parse::<i64>(), 
                        step_part.parse::<i64>()
                    ) {
                        let mut result = Vec::new();
                        let mut i = start_num;
                        while i <= end_num {
                            result.push(format!("{:02}", i)); // Zero-pad to 2 digits
                            i += step;
                        }
                        return Some(result);
                    }
                }
            }
        }
        None
    }

    /// Escape string for various programming languages
    pub fn escape_string_for_language(s: &str, language: &str) -> String {
        match language {
            "perl" => {
                let unescaped = s.replace("\\\"", "\"");
                unescaped.replace("\\", "\\\\")
                         .replace("\"", "\\\"")
                         .replace("\n", "\\n")
                         .replace("\r", "\\r")
                         .replace("\t", "\\t")
            }
            "rust" => {
                s.replace("\\", "\\\\")
                 .replace("\"", "\\\"")
                 .replace("\n", "\\n")
                 .replace("\r", "\\r")
                 .replace("\t", "\\t")
            }
            "python" => {
                s.replace("\\", "\\\\")
                 .replace("\"", "\\\"")
                 .replace("\n", "\\n")
                 .replace("\r", "\\r")
                 .replace("\t", "\\t")
            }
            "javascript" => {
                s.replace("\\", "\\\\")
                 .replace("\"", "\\\"")
                 .replace("\n", "\\n")
                 .replace("\r", "\\r")
                 .replace("\t", "\\t")
            }
            "lua" => {
                s.replace("\\", "\\\\")
                 .replace("\"", "\\\"")
                 .replace("\n", "\\n")
                 .replace("\r", "\\r")
                 .replace("\t", "\\t")
            }
            "c" => {
                s.replace("\\", "\\\\")
                 .replace("\"", "\\\"")
                 .replace("\n", "\\n")
                 .replace("\r", "\\r")
                 .replace("\t", "\\t")
            }
            _ => s.to_string(),
        }
    }

    /// Generate indentation string
    pub fn indent(level: usize) -> String {
        "    ".repeat(level)
    }

    /// Extract variable name from shell syntax
    pub fn extract_var_name(arg: &str) -> Option<String> {
        if arg.starts_with('$') {
            Some(arg[1..].to_string())
        } else {
            None
        }
    }

    /// Check if a string looks like a variable name
    pub fn is_variable_name(s: &str) -> bool {
        if s.is_empty() {
            return false;
        }
        
        let first_char = s.chars().next().unwrap();
        if !first_char.is_alphabetic() && first_char != '_' {
            return false;
        }
        
        s.chars().all(|c| c.is_alphanumeric() || c == '_')
    }

    /// Convert shell arithmetic operators to language-specific equivalents
    pub fn convert_arithmetic_operators(expr: &str, language: &str) -> String {
        let mut result = expr.to_string();
        
        // Common arithmetic operators that are usually the same
        let operators = ["++", "--", "+=", "-=", "*=", "/=", "%=", "**="];
        for op in &operators {
            result = result.replace(op, op);
        }
        
        // Handle variable references based on language
        match language {
            "perl" => {
                // Ensure $ prefix for single identifiers
                let parts: Vec<&str> = result.split_whitespace().collect();
                let converted_parts: Vec<String> = parts.iter().map(|part| {
                    if Self::is_variable_name(part) {
                        format!("${}", part)
                    } else {
                        part.to_string()
                    }
                }).collect();
                converted_parts.join(" ")
            }
            "rust" => {
                // Rust variables don't need special prefix in expressions
                result
            }
            _ => result,
        }
    }
}
