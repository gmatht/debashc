use std::process::Command;

/// Extract line and column information from error messages
pub fn extract_line_col(e: &dyn std::error::Error) -> Option<(usize, usize)> {
    let msg = e.to_string();
    // Try to find pattern " at line:col" we emit in our errors
    let parts: Vec<&str> = msg.split_whitespace().collect();
    for window in parts.windows(2) {
        if window[0] == "at" {
            if let Some((l, c)) = parse_line_col(window[1]) { return Some((l, c)); }
        }
    }
    None
}

/// Parse line:col format from string
pub fn parse_line_col(s: &str) -> Option<(usize, usize)> {
    let mut it = s.split(':');
    let line = it.next()?.trim_end_matches(',');
    let col = it.next()?.trim_end_matches(',');
    Some((line.parse().ok()?, col.parse().ok()?))
}

/// Generate a caret snippet for error reporting
pub fn caret_snippet(input: &str, line: usize, col: usize) -> Option<String> {
    let lines: Vec<&str> = input.lines().collect();
    if line == 0 || line > lines.len() { return None; }
    let src_line = lines[line - 1];
    let mut caret = String::new();
    let prefix = format!("{:>4} | ", line);
    caret.push_str(&prefix);
    caret.push_str(src_line);
    caret.push('\n');
    caret.push_str(&" ".repeat(prefix.len().saturating_sub(0) + col.saturating_sub(1)));
    caret.push('^');
    Some(caret)
}

/// Check if a generator is available in PATH
pub fn check_generator_available(generator: &str) -> bool {
    match generator {
        "perl" => Command::new("perl").arg("--version").output().is_ok(),
        _ => false
    }
}

/// Clean up temporary files
pub fn cleanup_tmp(lang: &str, tmp_file: &str) {
    let _ = std::fs::remove_file(tmp_file);
    if lang == "rust" {
        let _ = std::fs::remove_file("__tmp_test_bin");
        if cfg!(target_os = "windows") {
            let _ = std::fs::remove_file(format!("{}.exe", "__tmp_test_bin"));
            let _ = std::fs::remove_file(format!("{}.pdb", "__tmp_test_bin"));
        }
    }
}

/// Generate unified diff format for comparing two strings
pub fn generate_unified_diff(expected: &str, actual: &str, expected_label: &str, actual_label: &str) -> String {
    let expected_lines: Vec<&str> = expected.lines().collect();
    let actual_lines: Vec<&str> = actual.lines().collect();
    
    let mut diff = String::new();
    diff.push_str(&format!("--- {}\n", expected_label));
    diff.push_str(&format!("+++ {}\n", actual_label));
    
    // Find the longest common subsequence to identify unchanged lines
    let mut i = 0;
    let mut j = 0;
    
    while i < expected_lines.len() && j < actual_lines.len() {
        if expected_lines[i] == actual_lines[j] {
            // Lines are identical - show with space prefix
            diff.push_str(&format!(" {}\n", expected_lines[i]));
            i += 1;
            j += 1;
        } else {
            // Lines differ - need to find the next match
            let mut found_match = false;
            
            // Look ahead in expected lines for a match
            for look_ahead in i + 1..expected_lines.len() {
                if look_ahead < expected_lines.len() && j < actual_lines.len() && 
                   expected_lines[look_ahead] == actual_lines[j] {
                    // Found a match ahead - the lines from i to look_ahead-1 were deleted
                    for k in i..look_ahead {
                        diff.push_str(&format!("-{}\n", expected_lines[k]));
                    }
                    i = look_ahead;
                    found_match = true;
                    break;
                }
            }
            
            // Look ahead in actual lines for a match
            if !found_match {
                for look_ahead in j + 1..actual_lines.len() {
                    if i < expected_lines.len() && look_ahead < actual_lines.len() && 
                       expected_lines[i] == actual_lines[look_ahead] {
                        // Found a match ahead - the lines from j to look_ahead-1 were inserted
                        for k in j..look_ahead {
                            diff.push_str(&format!("+{}\n", actual_lines[k]));
                        }
                        j = look_ahead;
                        found_match = true;
                        break;
                    }
                }
            }
            
            // If no match found ahead, treat as modification
            if !found_match {
                diff.push_str(&format!("-{}\n", expected_lines[i]));
                diff.push_str(&format!("+{}\n", actual_lines[j]));
                i += 1;
                j += 1;
            }
        }
    }
    
    // Handle remaining lines
    while i < expected_lines.len() {
        diff.push_str(&format!("-{}\n", expected_lines[i]));
        i += 1;
    }
    
    while j < actual_lines.len() {
        diff.push_str(&format!("+{}\n", actual_lines[j]));
        j += 1;
    }
    
    diff
}

/// Check if the generated Perl code contains forbidden patterns specified in PERL_MUST_NOT_CONTAIN comments
pub fn check_perl_must_not_contain(shell_content: &str, perl_code: &str) -> Result<(), String> {
    let lines: Vec<&str> = shell_content.lines().collect();
    let mut violations = Vec::new();
    
    for (line_num, line) in lines.iter().enumerate() {
        if line.contains("#PERL_MUST_NOT_CONTAIN:") {
            // Extract the forbidden pattern after the comment
            if let Some(pattern_start) = line.find("#PERL_MUST_NOT_CONTAIN:") {
                let pattern = line[pattern_start + "#PERL_MUST_NOT_CONTAIN:".len()..].trim();
                if !pattern.is_empty() {
                    // Check if the forbidden pattern exists in the generated Perl code
                    if perl_code.contains(pattern) {
                        violations.push(format!("Line {}: Found forbidden pattern '{}' in generated Perl code", line_num + 1, pattern));
                    }
                }
            }
        }
    }
    
    if violations.is_empty() {
        Ok(())
    } else {
        Err(format!("PERL_MUST_NOT_CONTAIN violations:\n{}", violations.join("\n")))
    }
}

/// Check if the AST string representation contains forbidden patterns specified in AST_MUST_NOT_CONTAIN comments
pub fn check_ast_must_not_contain(shell_content: &str, ast_string: &str) -> Result<(), String> {
    let lines: Vec<&str> = shell_content.lines().collect();
    let mut violations = Vec::new();
    
    for (line_num, line) in lines.iter().enumerate() {
        if line.contains("#AST_MUST_NOT_CONTAIN:") {
            // Extract the forbidden patterns after the comment
            if let Some(pattern_start) = line.find("#AST_MUST_NOT_CONTAIN:") {
                let pattern_text = line[pattern_start + "#AST_MUST_NOT_CONTAIN:".len()..].trim();
                if !pattern_text.is_empty() {
                    // Parse the pattern list like [Literal("-"), Literal("1")]
                    if let Some(patterns) = parse_ast_pattern_list(pattern_text) {
                        // Check if ALL forbidden patterns exist in the AST string
                        let all_patterns_found = patterns.iter().all(|pattern| ast_string.contains(pattern));
                        if all_patterns_found {
                            violations.push(format!("Line {}: Found all forbidden AST patterns {:?} in AST string", line_num + 1, patterns));
                        }
                    } else {
                        violations.push(format!("Line {}: Invalid AST_MUST_NOT_CONTAIN pattern format: {}", line_num + 1, pattern_text));
                    }
                }
            }
        }
    }
    
    if violations.is_empty() {
        Ok(())
    } else {
        Err(format!("AST_MUST_NOT_CONTAIN violations:\n{}", violations.join("\n")))
    }
}

/// Check if the AST string representation contains required patterns specified in AST_MUST_CONTAIN comments
pub fn check_ast_must_contain(shell_content: &str, ast_string: &str) -> Result<(), String> {
    let lines: Vec<&str> = shell_content.lines().collect();
    let mut violations = Vec::new();
    
    for (line_num, line) in lines.iter().enumerate() {
        if line.contains("#AST_MUST_CONTAIN:") {
            // Extract the required patterns after the comment
            if let Some(pattern_start) = line.find("#AST_MUST_CONTAIN:") {
                let pattern_text = line[pattern_start + "#AST_MUST_CONTAIN:".len()..].trim();
                if !pattern_text.is_empty() {
                    // Parse the pattern list like [Literal("-1")]
                    if let Some(patterns) = parse_ast_pattern_list(pattern_text) {
                        // Check if ALL required patterns exist in the AST string
                        let all_patterns_found = patterns.iter().all(|pattern| ast_string.contains(pattern));
                        if !all_patterns_found {
                            let missing_patterns: Vec<_> = patterns.iter()
                                .filter(|pattern| !ast_string.contains(pattern.as_str()))
                                .collect();
                            violations.push(format!("Line {}: Missing required AST patterns {:?} in AST string", line_num + 1, missing_patterns));
                        }
                    } else {
                        violations.push(format!("Line {}: Invalid AST_MUST_CONTAIN pattern format: {}", line_num + 1, pattern_text));
                    }
                }
            }
        }
    }
    
    if violations.is_empty() {
        Ok(())
    } else {
        Err(format!("AST_MUST_CONTAIN violations:\n{}", violations.join("\n")))
    }
}

/// Parse a pattern list like [Literal("-"), Literal("1")] into a vector of strings
pub fn parse_ast_pattern_list(pattern_text: &str) -> Option<Vec<String>> {
    // Remove outer brackets and split by comma
    let trimmed = pattern_text.trim();
    if !trimmed.starts_with('[') || !trimmed.ends_with(']') {
        return None;
    }
    
    let content = &trimmed[1..trimmed.len()-1];
    let patterns: Vec<String> = content
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    
    if patterns.is_empty() {
        None
    } else {
        Some(patterns)
    }
}
