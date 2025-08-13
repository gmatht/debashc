use crate::ast::*;

pub trait TestExpressionHandler {
    fn generate_test_expression(&self, test_expr: &TestExpression) -> String;
}

impl<T: TestExpressionHandler> TestExpressionHandler for T {
    fn generate_test_expression(&self, test_expr: &TestExpression) -> String {
        // Parse the test expression to extract components
        let expr = &test_expr.expression;
        let modifiers = &test_expr.modifiers;
        
        // Parse the expression to determine the type of test
        if expr.contains(" =~ ") {
            // Regex matching: [[ $var =~ pattern ]]
            let parts: Vec<&str> = expr.split(" =~ ").collect();
            if parts.len() == 2 {
                let var = parts[0].trim();
                let pattern = parts[1].trim();
                
                // Convert to Perl regex matching
                format!("({} =~ /{}/)", var, pattern)
            } else {
                "0".to_string()
            }
        } else if expr.contains(" == ") {
            // Pattern matching: [[ $var == pattern ]]
            let parts: Vec<&str> = expr.split(" == ").collect();
            if parts.len() == 2 {
                let var = parts[0].trim();
                let pattern = parts[1].trim();
                
                if modifiers.extglob {
                    // Handle extglob patterns
                    let regex_pattern = self.convert_extglob_to_perl_regex(pattern);
                    if modifiers.nocasematch {
                        format!("({} =~ /{}/i)", var, regex_pattern)
                    } else {
                        format!("({} =~ /{}/)", var, regex_pattern)
                    }
                } else {
                    // Regular glob pattern matching - convert glob to regex
                    let regex_pattern = self.convert_glob_to_regex(pattern);
                    if modifiers.nocasematch {
                        // Case-insensitive matching
                        format!("({} =~ /^{}$/i)", var, regex_pattern)
                    } else {
                        // Case-sensitive matching
                        format!("({} =~ /^{}$/)", var, regex_pattern)
                    }
                }
            } else {
                "0".to_string()
            }
        } else if expr.contains(" = ") {
            // String equality: [[ $var = value ]]
            let parts: Vec<&str> = expr.split(" = ").collect();
            if parts.len() == 2 {
                let var = parts[0].trim();
                let value = parts[1].trim();
                
                // Handle tilde expansion for home directory
                if var == "~" {
                    // Remove quotes from value if it's a shell variable reference
                    let clean_value = if value.starts_with('"') && value.ends_with('"') && value.contains('$') {
                        let unquoted = value[1..value.len()-1].to_string();
                        // Convert shell variables to Perl environment variables
                        if unquoted == "$HOME" {
                            "$ENV{'HOME'}".to_string()
                        } else {
                            unquoted
                        }
                    } else {
                        value.to_string()
                    };
                    format!("($ENV{{'HOME'}} eq {})", clean_value)
                } else if var.starts_with("~/") {
                    let path = var[2..].to_string();
                    // Remove quotes from value if it's a shell variable reference
                    let clean_value = if value.starts_with('"') && value.ends_with('"') && value.contains('$') {
                        let unquoted = value[1..value.len()-1].to_string();
                        // Convert shell variables to Perl environment variables
                        if unquoted == "$HOME" {
                            "$ENV{'HOME'}".to_string()
                        } else {
                            unquoted
                        }
                    } else {
                        value.to_string()
                    };
                    
                    // Handle the case where the value is a path that should be concatenated
                    if clean_value.contains('/') && clean_value.starts_with('$') {
                        // Convert $HOME/Documents to $ENV{'HOME'} . '/Documents'
                        let clean_path = clean_value.replace("$HOME", "$ENV{'HOME'}");
                        // Split the path and reconstruct it properly
                        if clean_path.contains('/') {
                            let path_parts: Vec<&str> = clean_path.split('/').collect();
                            if path_parts.len() == 2 && path_parts[0] == "$ENV{'HOME'}" {
                                format!("($ENV{{'HOME'}} . '/{}') eq ($ENV{{'HOME'}} . '/{}')", path, path_parts[1])
                            } else {
                                format!("($ENV{{'HOME'}} . '/{}') eq {}", path, clean_path)
                            }
                        } else {
                            format!("($ENV{{'HOME'}} . '/{}') eq {}", path, clean_path)
                        }
                    } else {
                        format!("($ENV{{'HOME'}} . '/{}') eq {}", path, clean_value)
                    }
                } else {
                    format!("({} eq {})", var, value)
                }
            } else {
                "0".to_string()
            }
        } else if expr.contains(" != ") {
            // Pattern matching: [[ $var != pattern ]]
            let parts: Vec<&str> = expr.split(" != ").collect();
            if parts.len() == 2 {
                let var = parts[0].trim();
                let pattern = parts[1].trim();
                
                if modifiers.extglob {
                    // Handle extglob patterns
                    let regex_pattern = self.convert_extglob_to_perl_regex(pattern);
                    if modifiers.nocasematch {
                        format!("({} !~ /{}/i)", var, regex_pattern)
                    } else {
                        format!("({} !~ /{}/)", var, regex_pattern)
                    }
                } else {
                    // Regular pattern matching
                    if modifiers.nocasematch {
                        // Case-insensitive matching
                        format!("lc({}) !~ /^{}$/i", var, pattern.replace("*", ".*"))
                    } else {
                        // Case-sensitive matching
                        format!("{} !~ /^{}$/", var, pattern.replace("*", ".*"))
                    }
                }
            } else {
                "0".to_string()
            }
        } else if expr.contains(" -eq ") {
            // Numeric equality: [[ $var -eq value ]]
            let parts: Vec<&str> = expr.split(" -eq ").collect();
            if parts.len() == 2 {
                let var = parts[0].trim();
                let value = parts[1].trim();
                
                format!("{} == {}", var, value)
            } else {
                "0".to_string()
            }
        } else if expr.contains(" -ne ") {
            // Numeric inequality: [[ $var -ne value ]]
            let parts: Vec<&str> = expr.split(" -ne ").collect();
            if parts.len() == 2 {
                let var = parts[0].trim();
                let value = parts[1].trim();
                
                format!("{} != {}", var, value)
            } else {
                "0".to_string()
            }
        } else if expr.contains(" -lt ") {
            // Less than: [[ $var -lt value ]]
            let parts: Vec<&str> = expr.split(" -lt ").collect();
            if parts.len() == 2 {
                let var = parts[0].trim();
                let value = parts[1].trim();
                
                format!("{} < {}", var, value)
            } else {
                "0".to_string()
            }
        } else if expr.contains(" -le ") {
            // Less than or equal: [[ $var -le value ]]
            let parts: Vec<&str> = expr.split(" -le ").collect();
            if parts.len() == 2 {
                let var = parts[0].trim();
                let value = parts[1].trim();
                
                format!("{} <= {}", var, value)
            } else {
                "0".to_string()
            }
        } else if expr.contains(" -gt ") {
            // Greater than: [[ $var -gt value ]]
            let parts: Vec<&str> = expr.split(" -gt ").collect();
            if parts.len() == 2 {
                let var = parts[0].trim();
                let value = parts[1].trim();
                
                format!("{} > {}", var, value)
            } else {
                "0".to_string()
            }
        } else if expr.contains(" -ge ") {
            // Greater than or equal: [[ $var -ge value ]]
            let parts: Vec<&str> = expr.split(" -ge ").collect();
            if parts.len() == 2 {
                let var = parts[0].trim();
                let value = parts[1].trim();
                
                format!("{} >= {}", var, value)
            } else {
                "0".to_string()
            }
        } else if expr.contains(" -z ") {
            // String is empty: [[ -z $var ]]
            let var_str = expr.replace("-z", "").trim().to_string();
            format!("{} eq ''", var_str)
        } else if expr.contains(" -n ") {
            // String is not empty: [[ -n $var ]]
            let var_str = expr.replace("-n", "").trim().to_string();
            format!("{} ne ''", var_str)
        } else if expr.contains(" -f ") {
            // File exists and is regular: [[ -f $var ]]
            let var_str = expr.replace("-f", "").trim().to_string();
            format!("-f {}", var_str)
        } else if expr.contains(" -d ") {
            // Directory exists: [[ -d $var ]]
            let var_str = expr.replace("-d", "").trim().to_string();
            format!("-d {}", var_str)
        } else if expr.contains(" -e ") {
            // File exists: [[ -e $var ]]
            let var_str = expr.replace("-e", "").trim().to_string();
            format!("-e {}", var_str)
        } else if expr.contains(" -r ") {
            // File is readable: [[ -r $var ]]
            let var_str = expr.replace("-r", "").trim().to_string();
            format!("-r {}", var_str)
        } else if expr.contains(" -w ") {
            // File is writable: [[ -w $var ]]
            let var_str = expr.replace("-w", "").trim().to_string();
            format!("-w {}", var_str)
        } else if expr.contains(" -x ") {
            // File is executable: [[ -x $var ]]
            let var_str = expr.replace("-x", "").trim().to_string();
            format!("-x {}", var_str)
        } else {
            // Try to parse the expression as a single string that might contain test operators
            // This handles cases where the parser captured the entire test expression as one string
            // First, strip any outer quotes from the expression
            let clean_expr = expr.trim_matches('"').trim_matches('\'');
            
            // Handle the case where the expression is a single quoted string like '-f "file.txt"'
            if clean_expr.starts_with("-f ") {
                let operand = clean_expr[3..].trim().trim_matches('"').trim_matches('\'');
                format!("-f \"{}\"", operand)
            } else if clean_expr.starts_with("-d ") {
                let operand = clean_expr[3..].trim().trim_matches('"').trim_matches('\'');
                format!("-d \"{}\"", operand)
            } else if clean_expr.starts_with("-e ") {
                let operand = clean_expr[3..].trim().trim_matches('"').trim_matches('\'');
                format!("-e \"{}\"", operand)
            } else if clean_expr.starts_with("-r ") {
                let operand = clean_expr[3..].trim().trim_matches('"').trim_matches('\'');
                format!("-r \"{}\"", operand)
            } else if clean_expr.starts_with("-w ") {
                let operand = clean_expr[3..].trim().trim_matches('"').trim_matches('\'');
                format!("-w \"{}\"", operand)
            } else if clean_expr.starts_with("-x ") {
                let operand = clean_expr[3..].trim().trim_matches('"').trim_matches('\'');
                format!("-x \"{}\"", operand)
            } else if clean_expr.starts_with("-z ") {
                let operand = clean_expr[3..].trim().trim_matches('"').trim_matches('\'');
                format!("{} eq ''", operand)
            } else if clean_expr.starts_with("-n ") {
                let operand = clean_expr[3..].trim().trim_matches('"').trim_matches('\'');
                format!("{} ne ''", operand)
            } else {
                // Unknown test expression
                format!("0 # Unknown test: {}", expr)
            }
        }
    }
}

// Helper trait for extglob conversion
trait ExtglobConverter {
    fn convert_extglob_to_perl_regex(&self, pattern: &str) -> String;
    fn convert_glob_to_regex(&self, pattern: &str) -> String;
    fn convert_extglob_negated_pattern(&self, pattern: &str) -> String;
    fn convert_simple_pattern_to_regex(&self, pattern: &str) -> String;
}

impl<T: TestExpressionHandler> ExtglobConverter for T {
    fn convert_extglob_to_perl_regex(&self, pattern: &str) -> String {
        // Handle extglob patterns like !(*.min).js
        if pattern.starts_with("!(") && pattern.contains(")") {
            if let Some(close_paren) = pattern.find(')') {
                let negated_pattern = &pattern[2..close_paren];
                let suffix = &pattern[close_paren + 1..];
                
                // Convert the negated pattern to a regex
                let negated_regex = self.convert_extglob_negated_pattern(negated_pattern);
                
                if suffix.is_empty() {
                    // No suffix, just negate the pattern
                    format!("^(?!{})$", negated_regex)
                } else {
                    // Has suffix, we need to check if the string ends with the suffix
                    // but the part before the suffix doesn't match the negated pattern
                    let suffix_regex = self.convert_simple_pattern_to_regex(suffix);
                    
                    // For !(*.min).js, we want to match strings that:
                    // 1. End with .js
                    // 2. The part before .js doesn't match *.min
                    
                    // The correct approach is to check if the string doesn't match the pattern
                    // that would be formed by combining the negated pattern with the suffix
                    // For !(*.min).js, we want to avoid matching strings that end with .min.js
                    
                    // The regex should be: ^(?!.*\.min\.js$).*\.js$
                    // This means: start of string, not followed by anything ending in .min.js, then anything, then .js, then end
                    
                    // For !(*.min).js, we want to avoid matching strings that end with .min.js
                    // So we check if the string doesn't match the pattern that would be formed
                    // by combining the negated pattern with the suffix
                    
                    // The negated_regex already starts with .* (from the * conversion),
                    // so we don't need to add another .* in front
                    let combined_negated = format!("{}{}", negated_regex, suffix_regex);
                    
                    // We need to allow any content before the suffix, so the final regex should be:
                    // ^(?!.*\.min\.js$).*\.js$ - this allows any content before .js
                    format!("^(?!{}){}$", combined_negated, ".*".to_string() + &suffix_regex)
                }
            } else {
                // Fallback if parentheses don't match
                self.convert_simple_pattern_to_regex(pattern)
            }
        } else {
            // Not an extglob pattern, use regular conversion
            self.convert_simple_pattern_to_regex(pattern)
        }
    }
    
    fn convert_extglob_negated_pattern(&self, pattern: &str) -> String {
        // For extglob negated patterns like *.min, we need to handle * specially
        // The * in extglob means "any sequence of characters" 
        // We want to create a regex that matches the literal pattern
        // For *.min, we want to match any sequence followed by .min
        // First escape special characters, then convert * to .*
        pattern
            .replace(".", "\\.") // Escape dots first
            .replace("[", "\\[") // Escape brackets
            .replace("]", "\\]") // Escape brackets
            .replace("(", "\\(") // Escape parentheses
            .replace(")", "\\)") // Escape parentheses
            .replace("*", ".*")  // Convert * to .* for regex
            .replace("?", ".")   // Convert ? to . for regex
    }
    
    fn convert_simple_pattern_to_regex(&self, pattern: &str) -> String {
        // Convert shell glob patterns to regex
        pattern
            .replace("*", ".*")
            .replace("?", ".")
            .replace(".", "\\.")
            .replace("[", "\\[")
            .replace("]", "\\]")
            .replace("(", "\\(")
            .replace(")", "\\)")
    }
    
    fn convert_glob_to_regex(&self, pattern: &str) -> String {
        // Convert shell glob patterns to regex
        pattern
            .replace("*", ".*")
            .replace("?", ".")
            .replace(".", "\\.")
            .replace("[", "\\[")
            .replace("]", "\\]")
            .replace("(", "\\(")
            .replace(")", "\\)")
    }
}
