// Removed unused import: use crate::ast::*;

use std::fs;
use std::io;
use std::io::Write;

/// Shared utilities for shell script generators
pub struct SharedUtils;

impl SharedUtils {
    // Removed unused parse functions to simplify code

    /// Convert glob pattern to regex pattern    // Removed unused convert_extglob_to_regex function

    // Removed unused expand_brace_expression function

    // Removed unused escape_string_for_language function

    /// Generate indentation string    // Removed unused extract_var_name function

    /// Write content to file with proper UTF-8 encoding
    pub fn write_utf8_file(path: &str, content: &str) -> io::Result<()> {
        // Ensure the content is valid UTF-8 and write with BOM if needed
        let mut file = fs::File::create(path)?;
        // Write UTF-8 BOM for better compatibility
        file.write_all(&[0xEF, 0xBB, 0xBF])?;
        file.write_all(content.as_bytes())?;
        file.flush()?;
        Ok(())
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
                // Split by operators, not just whitespace
                let operators = ['+', '-', '*', '/', '%', '(', ')', ' ', '\t', '\n'];
                let parts: Vec<&str> = result.split(|c| operators.contains(&c)).collect();
                let mut final_result = String::new();
                let mut last_pos = 0;
                
                for part in parts {
                    let part = part.trim();
                    if !part.is_empty() {
                        // Find where this part appears in the original string
                        if let Some(pos) = result[last_pos..].find(part) {
                            // Add any operators that come before this part
                            let actual_pos = last_pos + pos;
                            if actual_pos > last_pos {
                                final_result.push_str(&result[last_pos..actual_pos]);
                            }
                            
                            // Add the part (with $ prefix if it's a variable)
                            if Self::is_variable_name(part) {
                                final_result.push_str(&format!("${}", part));
                            } else {
                                final_result.push_str(part);
                            }
                            
                            last_pos = actual_pos + part.len();
                        }
                    }
                }
                
                // Add any remaining characters
                if last_pos < result.len() {
                    final_result.push_str(&result[last_pos..]);
                }
                
                final_result
            }
            "rust" => {
                // Rust variables don't need special prefix in expressions
                result
            }
            _ => result,
        }
    }
}
