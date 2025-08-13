use crate::ast::*;

/// Represents the parsed options and arguments for a grep command
#[derive(Debug, Clone)]
pub struct GrepOptions {
    pub pattern: Option<String>,
    pub file: Option<String>,
    pub max_count: Option<usize>,
    pub show_byte_offset: bool,
    pub only_matching: bool,
    pub quiet_mode: bool,
    pub literal_mode: bool,
    pub ignore_case: bool,
    pub list_files_only: bool,
    pub list_files_without_matches: bool,
    pub recursive: bool,
    pub context_after: Option<usize>,
    pub context_before: Option<usize>,
    pub context_both: Option<usize>,
    pub include_pattern: Option<String>,
    pub search_path: String,
}

impl Default for GrepOptions {
    fn default() -> Self {
        Self {
            pattern: None,
            file: None,
            max_count: None,
            show_byte_offset: false,
            only_matching: false,
            quiet_mode: false,
            literal_mode: false,
            ignore_case: false,
            list_files_only: false,
            list_files_without_matches: false,
            recursive: false,
            context_after: None,
            context_before: None,
            context_both: None,
            include_pattern: None,
            search_path: ".".to_string(),
        }
    }
}

/// Handler for grep command functionality
pub struct GrepHandler;

impl GrepHandler {
    /// Parse grep command arguments and extract options
    pub fn parse_grep_args(cmd: &SimpleCommand, word_to_perl: &mut dyn FnMut(&Word) -> String) -> GrepOptions {
        let mut options = GrepOptions::default();
        let mut i = 0;
        
        while i < cmd.args.len() {
            let arg = &cmd.args[i];
            if let Word::Literal(s) = arg {
                if s.starts_with('-') {
                    // Handle flags
                    match s.as_str() {
                        "-m" if i + 1 < cmd.args.len() => {
                            if let Word::Literal(count_str) = &cmd.args[i + 1] {
                                options.max_count = count_str.parse::<usize>().ok();
                                i += 1; // Skip the count argument
                            }
                        }
                        "-b" => options.show_byte_offset = true,
                        "-o" => options.only_matching = true,
                        "-q" => options.quiet_mode = true,
                        "-F" => options.literal_mode = true,
                        "-i" => options.ignore_case = true,
                        "-Z" => {}, // -Z flag for null-terminated output
                        "-l" => options.list_files_only = true,
                        "-L" => options.list_files_without_matches = true,
                        "-r" => options.recursive = true,
                        "-A" if i + 1 < cmd.args.len() => {
                            if let Word::Literal(count_str) = &cmd.args[i + 1] {
                                options.context_after = count_str.parse::<usize>().ok();
                                i += 1; // Skip the count argument
                            }
                        }
                        "-B" if i + 1 < cmd.args.len() => {
                            if let Word::Literal(count_str) = &cmd.args[i + 1] {
                                options.context_before = count_str.parse::<usize>().ok();
                                i += 1; // Skip the count argument
                            }
                        }
                        "-C" if i + 1 < cmd.args.len() => {
                            if let Word::Literal(count_str) = &cmd.args[i + 1] {
                                options.context_both = count_str.parse::<usize>().ok();
                                i += 1; // Skip the count argument
                            }
                        }
                        "--include" if i + 1 < cmd.args.len() => {
                            if let Word::Literal(pattern_str) = &cmd.args[i + 1] {
                                options.include_pattern = Some(pattern_str.clone());
                                i += 1; // Skip the pattern argument
                            }
                        }
                        _ => {} // Ignore unknown flags
                    }
                } else if options.pattern.is_none() {
                    options.pattern = Some(s.clone());
                } else if options.file.is_none() {
                    options.file = Some(s.clone());
                } else if options.recursive && options.search_path == "." {
                    options.search_path = s.clone();
                }
            } else if options.pattern.is_none() {
                options.pattern = Some(word_to_perl(arg));
            } else if options.file.is_none() {
                options.file = Some(word_to_perl(arg));
            } else if options.recursive && options.search_path == "." {
                options.search_path = word_to_perl(arg);
            }
            i += 1;
        }
        
        // Apply context_both to both before and after if specified
        if let Some(context_both) = options.context_both {
            options.context_before = Some(context_both);
            options.context_after = Some(context_both);
        }
        
        options
    }
    
    /// Generate Perl code for xargs grep functionality
    pub fn generate_xargs_grep_perl(
        pattern: &str,
        output: &mut String,
        pipeline_id: usize,
        ignore_case: bool,
    ) {
        output.push_str(&format!("my @xargs_files_{};\n", pipeline_id));
        output.push_str(&format!("for my $file (split(/\\n/, $output_{})) {{\n", pipeline_id));
        output.push_str(&format!("    if ($file ne '') {{\n"));
        output.push_str(&format!("        # Use Perl's built-in file reading instead of system grep for cross-platform compatibility\n"));
        output.push_str(&format!("        my $found = 0;\n"));
        output.push_str(&format!("        if (open(my $fh, '<', $file)) {{\n"));
        output.push_str(&format!("            while (my $line = <$fh>) {{\n"));
        
        let regex_flags = if ignore_case { "i" } else { "" };
        output.push_str(&format!("                if ($line =~ /{}/{}) {{\n", pattern, regex_flags));
        output.push_str(&format!("                    $found = 1;\n"));
        output.push_str(&format!("                    last;\n"));
        output.push_str(&format!("                }}\n"));
        output.push_str(&format!("            }}\n"));
        output.push_str(&format!("            close($fh);\n"));
        output.push_str(&format!("        }}\n"));
        output.push_str(&format!("        if ($found) {{\n"));
        output.push_str(&format!("            push @xargs_files_{}, $file;\n", pipeline_id));
        output.push_str(&format!("        }}\n"));
        output.push_str(&format!("    }}\n"));
        output.push_str(&format!("}}\n"));
        output.push_str(&format!("$output_{} = join(\"\\n\", @xargs_files_{});\n", pipeline_id, pipeline_id));
    }
}
