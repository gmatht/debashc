use crate::ast::*;
use super::PerlGenerator;

impl PerlGenerator {
    pub fn generate_parameter_expansion(&self, pe: &ParameterExpansion) -> String {
        // Basic parameter expansion - just return the variable name
        format!("${{{}}}", pe.variable)
    }

    pub fn extract_parameter_expansion(&self, var: &str) -> Option<ParameterExpansion> {
        // Basic parameter expansion extraction
        if var.starts_with('$') && var.contains('{') && var.contains('}') {
            let var_name = var.trim_start_matches('$').trim_matches('{').trim_matches('}');
            Some(ParameterExpansion {
                variable: var_name.to_string(),
                operator: ParameterExpansionOperator::None,
            })
        } else {
            None
        }
    }

    pub fn generate_glob_handler(&mut self, pattern: &str, action: &str) -> String {
        // Convert glob pattern to Perl glob
        let mut perl_pattern = pattern.to_string();
        
        // Escape regex special characters
        perl_pattern = perl_pattern.replace(".", "\\.");
        perl_pattern = perl_pattern.replace("+", "\\+");
        perl_pattern = perl_pattern.replace("(", "\\(");
        perl_pattern = perl_pattern.replace(")", "\\)");
        perl_pattern = perl_pattern.replace("[", "\\[");
        perl_pattern = perl_pattern.replace("]", "\\]");
        perl_pattern = perl_pattern.replace("^", "\\^");
        perl_pattern = perl_pattern.replace("$", "\\$");
        perl_pattern = perl_pattern.replace("|", "\\|");
        
        // Convert glob patterns to regex
        perl_pattern = perl_pattern.replace("*", ".*");
        perl_pattern = perl_pattern.replace("?", ".");
        
        match action {
            "match" => format!("qr/^{}$/", perl_pattern),
            "find" => format!("grep {{ /^{}$/ }} glob('*')", perl_pattern),
            _ => format!("qr/^{}$/", perl_pattern)
        }
    }

    pub fn generate_cartesian_product(&self, expansion_values: &[Vec<String>], result: &mut Vec<Vec<String>>, depth: usize, current: &mut Vec<String>) {
        if depth == expansion_values.len() {
            result.push(current.clone());
            return;
        }
        
        for value in &expansion_values[depth] {
            current.push(value.clone());
            self.generate_cartesian_product(expansion_values, result, depth + 1, current);
            current.pop();
        }
    }

    pub fn expand_numeric_range_helper(&self, start_str: &str, end_str: &str, step_str: &str) -> Option<String> {
        let (start, end, step) = (
            start_str.parse::<i64>().ok()?,
            end_str.parse::<i64>().ok()?,
            step_str.parse::<i64>().ok()?
        );
        
        if start > end {
            return None;
        }
        
        let values: Vec<String> = (start..=end)
            .step_by(step as usize)
            .map(|i| {
                // Preserve leading zeros by formatting with the same width as the original
                if start_str.starts_with('0') && start_str.len() > 1 {
                    format!("{:0width$}", i, width = start_str.len())
                } else {
                    i.to_string()
                }
            })
            .collect();
        
        Some(values.join(" "))
    }

    pub fn expand_brace_literal(&self, s: &str) -> String {
        if s.contains("..") {
            let parts: Vec<&str> = s.split("..").collect();
            match parts.len() {
                2 => {
                    // Simple range like "a..c"
                    if let (Some(start_char), Some(end_char)) = (parts[0].chars().next(), parts[1].chars().next()) {
                        if let Some(expanded) = self.expand_char_range(start_char, end_char, None) {
                            return expanded;
                        }
                    }
                }
                3 => {
                    if parts[1].contains("..") {
                        // Character range with step like "a..z..3"
                        let sub_parts: Vec<&str> = parts[1].split("..").collect();
                        if sub_parts.len() == 2 {
                            if let (Some(start_char), Some(end_char)) = (parts[0].chars().next(), sub_parts[1].chars().next()) {
                                if let Ok(step) = parts[2].parse::<usize>() {
                                    if let Some(expanded) = self.expand_char_range(start_char, end_char, Some(step)) {
                                        return expanded;
                                    }
                                }
                            }
                        }
                    } else {
                        // Range with step like "00..04..2"
                        if let Some(expanded) = self.expand_numeric_range_helper(parts[0], parts[1], parts[2]) {
                            return expanded;
                        }
                    }
                }
                _ => {}
            }
        } else if s.contains(',') {
            // Handle comma-separated sequences like "a,b,c"
            let parts: Vec<&str> = s.split(',').collect();
            if parts.len() > 1 {
                return parts.join(" ");
            }
        }
        
        // Return reference instead of cloning
        s.to_string()
    }

    pub fn expand_char_range(&self, start: char, end: char, step: Option<usize>) -> Option<String> {
        let step = step.unwrap_or(1);
        let start_byte = start as u8;
        let end_byte = end as u8;
        
        if start_byte > end_byte {
            return None;
        }
        
        let values: Vec<String> = (start_byte..=end_byte)
            .step_by(step)
            .map(|b| char::from(b).to_string())
            .collect();
        
        Some(values.join(" "))
    }
}
