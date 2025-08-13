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
                    options.pattern = Some(s.to_string());
                } else if options.file.is_none() {
                    options.file = Some(s.to_string());
                } else if options.recursive && options.search_path == "." {
                    options.search_path = s.to_string();
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

    /// Generate Perl code for main grep command functionality
    pub fn generate_main_grep_perl(
        cmd: &SimpleCommand,
        output: &mut String,
        has_here_string: bool,
        word_to_perl: &mut dyn FnMut(&Word) -> String,
        get_unique_file_handle: &mut dyn FnMut() -> String,
    ) {
        // Parse grep options using the existing parse_grep_args method
        let options = Self::parse_grep_args(cmd, word_to_perl);
        
        let pattern = options.pattern.unwrap_or_else(|| "".to_string());
        let file = options.file.map_or("STDIN".to_string(), |w| w.to_string());
        
        // Handle different grep modes based on options
        if options.list_files_only || options.list_files_without_matches {
            // Handle file listing mode (-l, -L)
            if &file == "STDIN" {
                // For stdin, we need to read filenames and check each one
                output.push_str("my @file_list;\n");
                output.push_str("while (my $filename = <STDIN>) {\n");
                output.push_str("    chomp($filename);\n");
                output.push_str("    if ($filename ne '' && -f $filename) {\n");
                output.push_str("        my $found = 0;\n");
                output.push_str("        if (open(my $fh, '<', $filename)) {\n");
                output.push_str("            while (my $line = <$fh>) {\n");
                
                if options.literal_mode {
                    output.push_str(&format!("                if (index($line, \"{}\") != -1) {{\n", pattern));
                } else {
                    let regex_flags = if options.ignore_case { "i" } else { "" };
                    output.push_str(&format!("                if ($line =~ /{}/{}) {{\n", pattern, regex_flags));
                }
                
                output.push_str("                    $found = 1;\n");
                output.push_str("                    last;\n");
                output.push_str("                }\n");
                output.push_str("            }\n");
                output.push_str("            close($fh);\n");
                output.push_str("        }\n");
                output.push_str("        if ($found) {\n");
                if options.list_files_only {
                    output.push_str("            push @file_list, $filename;\n");
                }
                output.push_str("        } else {\n");
                if options.list_files_without_matches {
                    output.push_str("            push @file_list, $filename;\n");
                }
                output.push_str("        }\n");
                output.push_str("    }\n");
                output.push_str("}\n");
                output.push_str("print join(\"\\n\", @file_list);\n");
            } else {
                // For a single file, just check if it contains the pattern
                let fh = get_unique_file_handle();
                output.push_str(&format!("my $found = 0;\n"));
                output.push_str(&format!("if (open(my {}, '<', '{}')) {{\n", fh, file));
                output.push_str(&format!("    while (my $line = <{}>) {{\n", fh));
                
                if options.literal_mode {
                    output.push_str(&format!("        if (index($line, \"{}\") != -1) {{\n", pattern));
                } else {
                    let regex_flags = if options.ignore_case { "i" } else { "" };
                    output.push_str(&format!("        if ($line =~ /{}/{}) {{\n", pattern, regex_flags));
                }
                
                output.push_str("            $found = 1;\n");
                output.push_str("            last;\n");
                output.push_str("        }\n");
                output.push_str("    }\n");
                output.push_str(&format!("    close({});\n", fh));
                output.push_str("}\n");
                if options.list_files_only {
                    output.push_str(&format!("if ($found) {{ print \"{}\" }}\n", file));
                } else if options.list_files_without_matches {
                    output.push_str(&format!("if (!$found) {{ print \"{}\" }}\n", file));
                }
            }
        } else if options.quiet_mode {
            // Quiet mode - just check if pattern exists
            if &file == "STDIN" {
                output.push_str("my $found = 0;\n");
                if has_here_string {
                    output.push_str("my @here_lines = split(/\\n/, $here_string_content);\n");
                    output.push_str("foreach my $line (@here_lines) {\n");
                    
                    if options.literal_mode {
                        output.push_str(&format!("    if (index($line, \"{}\") != -1) {{\n", pattern));
                    } else {
                        let regex_flags = if options.ignore_case { "i" } else { "" };
                        output.push_str(&format!("    if ($line =~ /{}/{}) {{\n", pattern, regex_flags));
                    }
                    
                    output.push_str("        $found = 1;\n");
                    output.push_str("        last;\n");
                    output.push_str("    }\n");
                    output.push_str("}\n");
                } else {
                    output.push_str("while (my $line = <STDIN>) {\n");
                    
                    if options.literal_mode {
                        output.push_str(&format!("    if (index($line, \"{}\") != -1) {{\n", pattern));
                    } else {
                        let regex_flags = if options.ignore_case { "i" } else { "" };
                        output.push_str(&format!("    if ($line =~ /{}/{}) {{\n", pattern, regex_flags));
                    }
                    
                    output.push_str("        $found = 1;\n");
                    output.push_str("        last;\n");
                    output.push_str("    }\n");
                    output.push_str("}\n");
                }
                output.push_str("exit($found ? 0 : 1);\n");
            } else {
                let fh = get_unique_file_handle();
                output.push_str(&format!("my $found = 0;\n"));
                output.push_str(&format!("if (open(my {}, '<', '{}')) {{\n", fh, file));
                output.push_str(&format!("    while (my $line = <{}>) {{\n", fh));
                
                if options.literal_mode {
                    output.push_str(&format!("        if (index($line, \"{}\") != -1) {{\n", pattern));
                } else {
                    let regex_flags = if options.ignore_case { "i" } else { "" };
                    output.push_str(&format!("        if ($line =~ /{}/{}) {{\n", pattern, regex_flags));
                }
                
                output.push_str("            $found = 1;\n");
                output.push_str("            last;\n");
                output.push_str("        }\n");
                output.push_str("    }\n");
                output.push_str(&format!("    close({});\n", fh));
                output.push_str("}\n");
                output.push_str("exit($found ? 0 : 1);\n");
            }
        } else if options.only_matching {
            // Only matching mode (-o)
            if &file == "STDIN" {
                if has_here_string {
                    output.push_str("my @here_lines = split(/\\n/, $here_string_content);\n");
                    output.push_str("foreach my $line (@here_lines) {\n");
                    
                    if options.literal_mode {
                        output.push_str(&format!("    if (index($line, \"{}\") != -1) {{\n", pattern));
                    } else {
                        let regex_flags = if options.ignore_case { "i" } else { "" };
                        output.push_str(&format!("    if ($line =~ /({})/{}) {{\n", pattern, regex_flags));
                    }
                    
                    output.push_str("        print \"$1\\n\";\n");
                    output.push_str("    }\n");
                    output.push_str("}\n");
                } else {
                    output.push_str("while (my $line = <STDIN>) {\n");
                    
                    if options.literal_mode {
                        output.push_str(&format!("    if (index($line, \"{}\") != -1) {{\n", pattern));
                    } else {
                        let regex_flags = if options.ignore_case { "i" } else { "" };
                        output.push_str(&format!("    if ($line =~ /({})/{}) {{\n", pattern, regex_flags));
                    }
                    
                    output.push_str("        print \"$1\\n\";\n");
                    output.push_str("    }\n");
                    output.push_str("}\n");
                }
            } else {
                let fh = get_unique_file_handle();
                output.push_str(&format!("if (open(my {}, '<', '{}')) {{\n", fh, file));
                output.push_str(&format!("    while (my $line = <{}>) {{\n", fh));
                
                if options.literal_mode {
                    output.push_str(&format!("        if (index($line, \"{}\") != -1) {{\n", pattern));
                } else {
                    let regex_flags = if options.ignore_case { "i" } else { "" };
                    output.push_str(&format!("        if ($line =~ /({})/{}) {{\n", pattern, regex_flags));
                }
                
                output.push_str("            print \"$1\\n\";\n");
                output.push_str("        }\n");
                output.push_str("    }\n");
                output.push_str(&format!("    close({});\n", fh));
                output.push_str("}\n");
            }
        } else {
            // Normal mode
            if &file == "STDIN" {
                if has_here_string {
                    output.push_str("my @here_lines = split(/\\n/, $here_string_content);\n");
                    output.push_str("foreach my $line (@here_lines) {\n");
                    
                    if options.literal_mode {
                        output.push_str(&format!("    if (index($line, \"{}\") != -1) {{\n", pattern));
                    } else {
                        let regex_flags = if options.ignore_case { "i" } else { "" };
                        output.push_str(&format!("    if ($line =~ /{}/{}) {{\n", pattern, regex_flags));
                    }
                    
                    if options.show_byte_offset {
                        output.push_str(&format!("        my $offset = index($line, \"{}\");\n", pattern));
                        output.push_str("        print \"$offset:$line\";\n");
                    } else {
                        output.push_str("        print \"$line\";\n");
                    }
                    output.push_str("    }\n");
                    output.push_str("}\n");
                } else {
                    output.push_str("while (my $line = <STDIN>) {\n");
                    
                    if options.literal_mode {
                        output.push_str(&format!("    if (index($line, \"{}\") != -1) {{\n", pattern));
                    } else {
                        let regex_flags = if options.ignore_case { "i" } else { "" };
                        output.push_str(&format!("    if ($line =~ /{}/{}) {{\n", pattern, regex_flags));
                    }
                    
                    if options.show_byte_offset {
                        output.push_str(&format!("        my $offset = index($line, \"{}\");\n", pattern));
                        output.push_str("        print \"$offset:$line\";\n");
                    } else {
                        output.push_str("        print \"$line\";\n");
                    }
                    output.push_str("    }\n");
                    output.push_str("}\n");
                }
            } else {
                let fh = get_unique_file_handle();
                output.push_str(&format!("if (open(my {}, '<', '{}')) {{\n", fh, file));
                output.push_str(&format!("    while (my $line = <{}>) {{\n", fh));
                
                if options.literal_mode {
                    output.push_str(&format!("        if (index($line, \"{}\") != -1) {{\n", pattern));
                } else {
                    let regex_flags = if options.ignore_case { "i" } else { "" };
                    output.push_str(&format!("        if ($line =~ /{}/{}) {{\n", pattern, regex_flags));
                }
                
                if options.show_byte_offset {
                    output.push_str(&format!("            my $offset = index($line, \"{}\");\n", pattern));
                    output.push_str("            print \"$offset:$line\";\n");
                } else {
                    output.push_str("            print \"$line\";\n");
                }
                output.push_str("        }\n");
                output.push_str("    }\n");
                output.push_str(&format!("    close({});\n", fh));
                output.push_str("}\n");
            }
        }
    }
}
