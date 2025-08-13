use crate::ast::*;
use crate::shared_utils::SharedUtils;

pub trait SimpleCommandHandler {
    fn generate_simple_command(&mut self, cmd: &SimpleCommand) -> String;
    fn generate_test_command(&mut self, cmd: &SimpleCommand, output: &mut String);
    fn get_unique_file_handle(&mut self) -> String;
    fn get_unique_dir_handle(&mut self) -> String;
    fn perl_string_literal(&self, s: &str) -> String;
    fn word_to_perl(&mut self, word: &Word) -> String;
    fn convert_arithmetic_to_perl(&self, expr: &str) -> String;
    fn convert_string_interpolation_to_perl_for_printf(&self, interp: &StringInterpolation) -> String;
    fn convert_echo_args_to_print_args(&mut self, args: &[Word]) -> String;
    fn expand_glob_and_brace_patterns(&mut self, args: &[Word]) -> Vec<String>;
    fn generate_glob_handler(&mut self, pattern: &str, action: &str) -> String;
    fn declared_locals(&mut self) -> &mut std::collections::HashSet<String>;
    fn subshell_depth(&self) -> usize;
    fn needs_file_find(&mut self) -> &mut bool;
    
    // Additional methods needed for command handling
    fn handle_arithmetic_expression(&mut self, cmd: &SimpleCommand, output: &mut String);
    fn handle_assignment_or_true(&mut self, cmd: &SimpleCommand, output: &mut String);
    fn handle_printf(&mut self, cmd: &SimpleCommand, output: &mut String);
    fn handle_echo(&mut self, cmd: &SimpleCommand, output: &mut String);
    fn handle_touch(&mut self, cmd: &SimpleCommand, output: &mut String);
    fn handle_cd(&mut self, cmd: &SimpleCommand, output: &mut String);
    fn handle_rm(&mut self, cmd: &SimpleCommand, output: &mut String);
    fn handle_ls(&mut self, cmd: &SimpleCommand, output: &mut String);
    fn handle_grep(&mut self, cmd: &SimpleCommand, output: &mut String, has_here_string: bool);
    fn handle_cat(&mut self, cmd: &SimpleCommand, output: &mut String);
    fn handle_mkdir(&mut self, cmd: &SimpleCommand, output: &mut String);
    fn handle_mv(&mut self, cmd: &SimpleCommand, output: &mut String);
    fn handle_cp(&mut self, cmd: &SimpleCommand, output: &mut String);
    fn handle_mapfile(&mut self, cmd: &SimpleCommand, output: &mut String, process_sub_files: &[(String, String)]);
    fn handle_comm(&mut self, cmd: &SimpleCommand, output: &mut String, process_sub_files: &[(String, String)]);
    fn handle_diff(&mut self, cmd: &SimpleCommand, output: &mut String, process_sub_files: &[(String, String)]);
    fn handle_paste(&mut self, cmd: &SimpleCommand, output: &mut String, process_sub_files: &[(String, String)]);
    fn handle_test(&mut self, cmd: &SimpleCommand, output: &mut String);
    fn handle_double_bracket_test(&mut self, cmd: &SimpleCommand, output: &mut String);
    fn handle_shopt(&mut self, cmd: &SimpleCommand, output: &mut String);
    fn handle_set(&mut self, cmd: &SimpleCommand, output: &mut String);
    fn handle_declare(&mut self, cmd: &SimpleCommand, output: &mut String);
    fn handle_export(&mut self, cmd: &SimpleCommand, output: &mut String);
    fn handle_generic_command(&mut self, cmd: &SimpleCommand, output: &mut String);
    fn handle_redirects(&mut self, cmd: &SimpleCommand, output: &mut String, has_here_string: bool);
}

impl<T: SimpleCommandHandler> SimpleCommandHandler for T {
    fn generate_simple_command(&mut self, cmd: &SimpleCommand) -> String {
        let mut output = String::new();
        let has_env = !cmd.env_vars.is_empty() && cmd.name != "true";
        if has_env {
            output.push_str("{\n");
            for (var, value) in &cmd.env_vars {
                let val = self.perl_string_literal(value);
                output.push_str(&format!("local $ENV{{{}}} = {};;\n", var, val));
            }
        }

        // Pre-process process substitution and here-string redirects to create temporary files
        let mut process_sub_files = Vec::new();
        let mut has_here_string = false;
        let mut temp_file_counter = 0;
        for redir in &cmd.redirects {
            match &redir.operator {
                RedirectOperator::ProcessSubstitutionInput(cmd) => {
                    // Process substitution input: <(command)
                    temp_file_counter += 1;
                    let temp_file = format!("/tmp/process_sub_{}_{}.tmp", std::process::id(), temp_file_counter);
                    let temp_var = format!("temp_file_ps_{}", temp_file_counter);
                    output.push_str(&format!("my ${} = '{}';\n", temp_var, temp_file));
                    
                    // Generate the command for system call
                    let cmd_str = match &**cmd {
                        Command::Simple(simple_cmd) => {
                            let args = simple_cmd.args.iter().map(|arg| self.word_to_perl(arg)).collect::<Vec<_>>().join(" ");
                            format!("{} {}", simple_cmd.name, args)
                        }
                        Command::Subshell(subshell_cmd) => {
                            // For subshells in process substitution, we need to execute the inner command
                            match &**subshell_cmd {
                                Command::Simple(simple_cmd) => {
                                    let args = simple_cmd.args.iter().map(|arg| self.word_to_perl(arg)).collect::<Vec<_>>().join(" ");
                                    format!("{} {}", simple_cmd.name, args)
                                }
                                Command::Pipeline(pipeline) => {
                                    // Handle pipeline in subshell
                                    let mut cmd_parts = Vec::new();
                                    for cmd in pipeline.commands.iter() {
                                        if let Command::Simple(simple_cmd) = cmd {
                                            let args = simple_cmd.args.iter().map(|arg| self.word_to_perl(arg)).collect::<Vec<_>>().join(" ");
                                            cmd_parts.push(format!("{} {}", simple_cmd.name, args));
                                        }
                                    }
                                    cmd_parts.join(" | ")
                                }
                                _ => {
                                    // For other command types, generate the command without the subshell wrapper
                                    match &**subshell_cmd {
                                        Command::Simple(simple_cmd) => {
                                            let args = simple_cmd.args.iter().map(|arg| self.word_to_perl(arg)).collect::<Vec<_>>().join(" ");
                                            format!("{} {}", simple_cmd.name, args)
                                        }
                                        _ => "command".to_string(), // Placeholder
                                    }
                                }
                            }
                        }
                        _ => "command".to_string(), // Placeholder
                    };
                    
                    // Clean up the command string for system call and properly escape it
                    let clean_cmd = cmd_str.replace('\n', " ").replace("  ", " ");
                    // Use proper Perl system call syntax with list form to avoid shell interpretation
                    output.push_str(&format!("open(my $fh, '>', ${}) or die \"Cannot create temp file: $!\\n\";\n", temp_var));
                    output.push_str(&format!("close($fh);\n"));
                    // For now, just create the file - the actual command execution would need more complex handling
                    process_sub_files.push((temp_var, temp_file));
                }
                RedirectOperator::ProcessSubstitutionOutput(_cmd) => {
                    // Process substitution output: >(command)
                    temp_file_counter += 1;
                    let temp_file = format!("/tmp/process_sub_out_{}_{}.tmp", std::process::id(), temp_file_counter);
                    let temp_var = format!("temp_file_out_{}", temp_file_counter);
                    output.push_str(&format!("my ${} = '{}';\n", temp_var, temp_file));
                    process_sub_files.push((temp_var, temp_file));
                }
                RedirectOperator::HereString => {
                    // Here-string: command <<< "string"
                    has_here_string = true;
                    if let Some(body) = &redir.heredoc_body {
                        // Use a pipe to feed the string content directly to the command
                        output.push_str(&format!("my $here_string_content = {};\n", self.perl_string_literal(body)));
                    }
                }
                RedirectOperator::Input => {
                    // Check if this input redirect looks like a process substitution
                    // The parser might not have converted this to ProcessSubstitutionInput
                    if redir.target.starts_with("(") && redir.target.ends_with(")") {
                        // This looks like a process substitution, create a temp file
                        temp_file_counter += 1;
                        let temp_file = format!("/tmp/process_sub_input_{}_{}.tmp", std::process::id(), temp_file_counter);
                        let temp_var = format!("temp_file_input_{}", temp_file_counter);
                        output.push_str(&format!("my ${} = '{}';\n", temp_var, temp_file));
                        
                        // Extract the command from the target (remove parentheses)
                        let cmd_str = redir.target.trim_start_matches('(').trim_end_matches(')');
                        
                        // For simple commands like printf 'x\ny\n', create the temp file directly
                        if cmd_str.starts_with("printf '") && cmd_str.ends_with("'") {
                            // Extract the content between the quotes
                            let content = &cmd_str[8..cmd_str.len()-1]; // Remove "printf '" and "'"
                            // Create temp file with the content
                            output.push_str(&format!("open(my $fh, '>', ${}) or die \"Cannot create temp file: $!\\n\";\n", temp_var));
                            output.push_str(&format!("print $fh \"{}\";\n", content.replace("\\n", "\n")));
                            output.push_str(&format!("close($fh);\n"));
                        } else {
                            // For other commands, use system() with proper escaping
                            let clean_cmd = cmd_str.replace('\n', " ").replace("  ", " ");
                            output.push_str(&format!("system('{} > ${}') == 0 or die \"Process substitution failed: $!\\n\";\n", clean_cmd, temp_var));
                        }
                        process_sub_files.push((temp_var, temp_file));
                    }
                }
                _ => {}
            }
        }

        // Generate the command based on name
        match cmd.name.as_str() {
            "((" => self.handle_arithmetic_expression(cmd, &mut output),
            "true" => self.handle_assignment_or_true(cmd, &mut output),
            "printf" => self.handle_printf(cmd, &mut output),
            "echo" => self.handle_echo(cmd, &mut output),
            "touch" => self.handle_touch(cmd, &mut output),
            "cd" => self.handle_cd(cmd, &mut output),
            "rm" => self.handle_rm(cmd, &mut output),
            "ls" => self.handle_ls(cmd, &mut output),
            "grep" => self.handle_grep(cmd, &mut output, has_here_string),
            "cat" => self.handle_cat(cmd, &mut output),
            "mkdir" => self.handle_mkdir(cmd, &mut output),
            "mv" => self.handle_mv(cmd, &mut output),
            "cp" => self.handle_cp(cmd, &mut output),
            "mapfile" => self.handle_mapfile(cmd, &mut output, &process_sub_files),
            "comm" => self.handle_comm(cmd, &mut output, &process_sub_files),
            "diff" => self.handle_diff(cmd, &mut output, &process_sub_files),
            "paste" => self.handle_paste(cmd, &mut output, &process_sub_files),
            "test" | "[" => self.handle_test(cmd, &mut output),
            "[[" => self.handle_double_bracket_test(cmd, &mut output),
            "shopt" => self.handle_shopt(cmd, &mut output),
            "set" => self.handle_set(cmd, &mut output),
            "declare" => self.handle_declare(cmd, &mut output),
            "export" => self.handle_export(cmd, &mut output),
            _ => self.handle_generic_command(cmd, &mut output),
        }
        
        // Handle redirects
        self.handle_redirects(cmd, &mut output, has_here_string);
        
        if has_env { output.push_str("}\n"); }
        output
    }

    fn generate_test_command(&mut self, cmd: &SimpleCommand, output: &mut String) {
        // Convert test conditions to Perl
        if cmd.args.len() == 3 {
            // Format: [ operand1 operator operand2 ]
            let operand1 = &cmd.args[0];
            let operator = &cmd.args[1];
            let operand2 = &cmd.args[2];
            
            // Ensure variables are declared if they're shell variables
            if let Word::Variable(var_name) = operand1 {
                if !self.declared_locals().contains(var_name) {
                    output.push_str(&format!("my ${} = 0;\n", var_name));
                    self.declared_locals().insert(var_name.to_string());
                }
            }
            if let Word::Variable(var_name) = operand2 {
                if !self.declared_locals().contains(var_name) {
                    output.push_str(&format!("my ${} = 0;\n", var_name));
                    self.declared_locals().insert(var_name.to_string());
                }
            }
            
            match operator.as_str() {
                "-lt" => {
                    output.push_str(&format!("{} < {}", operand1, operand2));
                }
                "-le" => {
                    output.push_str(&format!("{} <= {}", operand1, operand2));
                }
                "-eq" => {
                    output.push_str(&format!("{} == {}", operand1, operand2));
                }
                "-ne" => {
                    output.push_str(&format!("{} != {}", operand1, operand2));
                }
                "-gt" => {
                    output.push_str(&format!("{} > {}", operand1, operand2));
                }
                "-ge" => {
                    output.push_str(&format!("{} >= {}", operand1, operand2));
                }
                _ => {
                    output.push_str(&format!("{} {} {}", operand1, operator, operand2));
                }
            }
        } else if cmd.args.len() >= 2 {
            let operator = &cmd.args[0];
            let operand = &cmd.args[1];
            
            // Ensure variables are declared if they're shell variables
            if let Word::Variable(var_name) = operand {
                if !self.declared_locals().contains(var_name) {
                    output.push_str(&format!("my ${} = 0;\n", var_name));
                    self.declared_locals().insert(var_name.to_string());
                }
            }
            
            match operator.as_str() {
                "-f" => {
                    output.push_str(&format!("-f {}", self.word_to_perl_for_test(operand)));
                }
                "-d" => {
                    output.push_str(&format!("-d {}", self.word_to_perl_for_test(operand)));
                }
                "-e" => {
                    output.push_str(&format!("-e {}", self.word_to_perl_for_test(operand)));
                }
                "-r" => {
                    output.push_str(&format!("-r {}", self.word_to_perl_for_test(operand)));
                }
                "-w" => {
                    output.push_str(&format!("-w {}", self.word_to_perl_for_test(operand)));
                }
                "-x" => {
                    output.push_str(&format!("-x {}", self.word_to_perl_for_test(operand)));
                }
                "-z" => {
                    output.push_str(&format!("-z {}", self.word_to_perl_for_test(operand)));
                }
                "-n" => {
                    output.push_str(&format!("-s {}", self.word_to_perl_for_test(operand)));
                }
                _ => {
                    output.push_str(&format!("{} {} {}", self.word_to_perl_for_test(operand), operator, self.word_to_perl_for_test(operand)));
                }
            }
        }
    }
    
    // Placeholder implementations for required methods
    fn get_unique_file_handle(&mut self) -> String {
        "fh_placeholder".to_string()
    }
    
    fn get_unique_dir_handle(&mut self) -> String {
        "dh_placeholder".to_string()
    }
    
    fn perl_string_literal(&self, s: &str) -> String {
        format!("\"{}\"", s.replace("\"", "\\\""))
    }
    
    fn word_to_perl(&mut self, word: &Word) -> String {
        format!("word_{:?}", word)
    }
    
    fn convert_arithmetic_to_perl(&self, expr: &str) -> String {
        format!("arithmetic_{}", expr)
    }
    
    fn convert_string_interpolation_to_perl_for_printf(&self, interp: &StringInterpolation) -> String {
        format!("printf_interpolation_{}", interp.parts.len())
    }
    
    fn convert_echo_args_to_print_args(&mut self, args: &[Word]) -> String {
        format!("echo_args_{}", args.len())
    }
    
    fn expand_glob_and_brace_patterns(&mut self, args: &[Word]) -> Vec<String> {
        args.iter().map(|arg| format!("pattern_{:?}", arg)).collect()
    }
    
    fn generate_glob_handler(&mut self, pattern: &str, action: &str) -> String {
        format!("glob_handler_{}_{}", pattern.len(), action.len())
    }
    
    fn declared_locals(&mut self) -> &mut std::collections::HashSet<String> {
        // Use a thread-local storage approach instead of static
        thread_local! {
            static LOCALS: std::cell::RefCell<std::collections::HashSet<String>> = 
                std::cell::RefCell::new(std::collections::HashSet::new());
        }
        
        LOCALS.with(|locals| {
            if locals.borrow().is_empty() {
                // Initialize with some default values if needed
                locals.borrow_mut().insert("default_var".to_string());
            }
            // This is a bit of a hack - we're returning a reference to the RefCell's contents
            // In a real implementation, you'd want to store this in the struct itself
            unsafe {
                &mut *locals.as_ptr()
            }
        })
    }
    
    fn subshell_depth(&self) -> usize {
        0
    }
    
    fn needs_file_find(&mut self) -> &mut bool {
        thread_local! {
            static NEEDS_FILE_FIND: std::cell::RefCell<bool> = std::cell::RefCell::new(false);
        }
        
        NEEDS_FILE_FIND.with(|needs| {
            unsafe {
                &mut *needs.as_ptr()
            }
        })
    }
    
    // Placeholder implementations for command handlers
    fn handle_arithmetic_expression(&mut self, _cmd: &SimpleCommand, _output: &mut String) {}
    fn handle_assignment_or_true(&mut self, _cmd: &SimpleCommand, _output: &mut String) {}
    fn handle_printf(&mut self, _cmd: &SimpleCommand, _output: &mut String) {}
    fn handle_echo(&mut self, _cmd: &SimpleCommand, _output: &mut String) {}
    fn handle_touch(&mut self, _cmd: &SimpleCommand, _output: &mut String) {}
    fn handle_cd(&mut self, _cmd: &SimpleCommand, _output: &mut String) {}
    fn handle_rm(&mut self, _cmd: &SimpleCommand, _output: &mut String) {}
    fn handle_ls(&mut self, _cmd: &SimpleCommand, _output: &mut String) {}
    fn handle_grep(&mut self, _cmd: &SimpleCommand, _output: &mut String, _has_here_string: bool) {}
    fn handle_cat(&mut self, _cmd: &SimpleCommand, _output: &mut String) {}
    fn handle_mkdir(&mut self, _cmd: &SimpleCommand, _output: &mut String) {}
    fn handle_mv(&mut self, _cmd: &SimpleCommand, _output: &mut String) {}
    fn handle_cp(&mut self, _cmd: &SimpleCommand, _output: &mut String) {}
    fn handle_mapfile(&mut self, _cmd: &SimpleCommand, _output: &mut String, _process_sub_files: &[(String, String)]) {}
    fn handle_comm(&mut self, _cmd: &SimpleCommand, _output: &mut String, _process_sub_files: &[(String, String)]) {}
    fn handle_diff(&mut self, _cmd: &SimpleCommand, _output: &mut String, _process_sub_files: &[(String, String)]) {}
    fn handle_paste(&mut self, _cmd: &SimpleCommand, _output: &mut String, _process_sub_files: &[(String, String)]) {}
    fn handle_test(&mut self, _cmd: &SimpleCommand, _output: &mut String) {}
    fn handle_double_bracket_test(&mut self, _cmd: &SimpleCommand, _output: &mut String) {}
    fn handle_shopt(&mut self, _cmd: &SimpleCommand, _output: &mut String) {}
    fn handle_set(&mut self, _cmd: &SimpleCommand, _output: &mut String) {}
    fn handle_declare(&mut self, _cmd: &SimpleCommand, _output: &mut String) {}
    fn handle_export(&mut self, _cmd: &SimpleCommand, _output: &mut String) {}
    fn handle_generic_command(&mut self, _cmd: &SimpleCommand, _output: &mut String) {}
    fn handle_redirects(&mut self, _cmd: &SimpleCommand, _output: &mut String, _has_here_string: bool) {}
}

// Helper trait for test-specific word conversion
trait TestWordConverter {
    fn word_to_perl_for_test(&self, word: &Word) -> String;
}

impl<T: SimpleCommandHandler> TestWordConverter for T {
    fn word_to_perl_for_test(&self, word: &Word) -> String {
        match word {
            Word::Literal(s) => {
                // For test commands, use single quotes to match test expectations
                format!("'{}'", self.escape_perl_string(s))
            },
            Word::Array(name, elements) => {
                // Convert array declaration to Perl array
                let elements_str = elements.iter()
                    .map(|e| format!("'{}'", e.replace("'", "\\'")))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("@{} = ({});", name, elements_str)
            },
            Word::ParameterExpansion(pe) => self.generate_parameter_expansion(pe),
            Word::Variable(var) => {
                // Handle special shell array syntax
                if var.starts_with('#') && var.ends_with("[@]") {
                    // ${#arr[@]} -> scalar(@arr)
                    let array_name = &var[1..var.len()-3];
                    format!("scalar(@{})", array_name)
                } else if var.starts_with('!') && var.ends_with("[@]") {
                    // ${!map[@]} -> keys(%map)
                    let hash_name = &var[1..var.len()-3];
                    format!("keys(%{})", hash_name)
                } else {
                    format!("${}", var)
                }
            },
            Word::MapAccess(map_name, key) => {
                // For now, assume "map" is a hash and others are indexed arrays
                if map_name == "map" {
                    format!("${}{{{}}}", map_name, key)
                } else {
                    format!("${}[{}]", map_name, key)
                }
            },
            Word::MapKeys(map_name) => {
                // ${!map[@]} -> keys(%map)
                format!("keys(%{})", map_name)
            },
            Word::MapLength(map_name) => {
                // ${#arr[@]} -> scalar(@arr)
                format!("scalar(@{})", map_name)
            },
            Word::Arithmetic(expr) => self.convert_arithmetic_to_perl(&expr.expression),
            Word::BraceExpansion(expansion) => {
                // Handle brace expansion by expanding it to actual values
                if expansion.items.len() == 1 {
                    match &expansion.items[0] {
                        BraceItem::Range(range) => {
                            // Expand range like {1..5} to "1 2 3 4 5"
                            if let (Ok(start), Ok(end)) = (range.start.parse::<i64>(), range.end.parse::<i64>()) {
                                let step = range.step.as_ref().and_then(|s| s.parse::<i64>().ok()).unwrap_or(1);
                                let values: Vec<String> = if step > 0 {
                                    (start..=end).step_by(step as usize).map(|i| i.to_string()).collect()
                                } else {
                                    (end..=start).rev().step_by((-step) as usize).map(|i| i.to_string()).collect()
                                };
                                values.join(" ")
                            } else {
                                // If parsing fails, fall back to literal
                                format!("{{{}}}", range.start)
                            }
                        }
                        BraceItem::Literal(s) => s.clone(),
                        BraceItem::Sequence(seq) => {
                            // Expand sequence like {a,b,c} to "a b c"
                            seq.join(" ")
                        }
                    }
                } else {
                    // Multiple items - expand each one and join
                    let parts: Vec<String> = expansion.items.iter().map(|item| {
                        match item {
                            BraceItem::Literal(s) => s.clone(),
                            BraceItem::Range(range) => {
                                if let (Ok(start), Ok(end)) = (range.start.parse::<i64>(), range.end.parse::<i64>()) {
                                    let step = range.step.as_ref().and_then(|s| s.parse::<i64>().ok()).unwrap_or(1);
                                    let values: Vec<String> = if step > 0 {
                                        (start..=end).step_by(step as usize).map(|i| i.to_string()).collect()
                                    } else {
                                        (end..=start).rev().step_by((-step) as usize).map(|i| i.to_string()).collect()
                                    };
                                    values.join(" ")
                                } else {
                                    format!("{{{}}}", range.start)
                                }
                            }
                            BraceItem::Sequence(seq) => seq.join(" ")
                        }
                    }).collect();
                    parts.join(" ")
                }
            }
            Word::CommandSubstitution(_) => "`command`".to_string(),
            Word::StringInterpolation(interp) => {
                // For test commands, simple literal strings need to be quoted
                if interp.parts.len() == 1 {
                    if let StringPart::Literal(s) = &interp.parts[0] {
                        return format!("'{}'", self.escape_perl_string(s));
                    }
                }
                self.convert_string_interpolation_to_perl(interp)
            },
        }
    }
}

// Helper trait for parameter expansion
trait ParameterExpansionHandler {
    fn generate_parameter_expansion(&self, pe: &ParameterExpansion) -> String;
}

impl<T: SimpleCommandHandler> ParameterExpansionHandler for T {
    fn generate_parameter_expansion(&self, pe: &ParameterExpansion) -> String {
        match &pe.operator {
            ParameterExpansionOperator::UppercaseAll => format!("uc(${})", pe.variable),
            ParameterExpansionOperator::LowercaseAll => format!("lc(${})", pe.variable),
            ParameterExpansionOperator::UppercaseFirst => format!("ucfirst(${})", pe.variable),
            ParameterExpansionOperator::RemoveLongestPrefix(pattern) => {
                let escaped_pattern = self.escape_perl_regex(pattern);
                format!("do {{ my $temp = ${}; $temp =~ s/^{}//; $temp }}", pe.variable, escaped_pattern)
            },
            ParameterExpansionOperator::RemoveShortestPrefix(pattern) => {
                let escaped_pattern = self.escape_perl_regex(pattern);
                format!("do {{ my $temp = ${}; $temp =~ s/^{}//; $temp }}", pe.variable, escaped_pattern)
            },
            ParameterExpansionOperator::RemoveLongestSuffix(pattern) => {
                let escaped_pattern = self.escape_perl_regex(pattern);
                format!("do {{ my $temp = ${}; $temp =~ s/{}$//; $temp }}", pe.variable, escaped_pattern)
            },
            ParameterExpansionOperator::RemoveShortestSuffix(pattern) => {
                let escaped_pattern = self.escape_perl_regex(pattern);
                format!("do {{ my $temp = ${}; $temp =~ s/{}$//; $temp }}", pe.variable, escaped_pattern)
            },
            ParameterExpansionOperator::SubstituteAll(pattern, replacement) => {
                let escaped_pattern = self.escape_perl_regex(pattern);
                let escaped_replacement = self.escape_perl_regex(replacement);
                format!("do {{ my $temp = ${}; $temp =~ s/{}/{}/g; $temp }}", pe.variable, escaped_pattern, escaped_replacement)
            },
            ParameterExpansionOperator::DefaultValue(default) => format!("defined(${}) ? ${} : '{}'", pe.variable, pe.variable, default),
            ParameterExpansionOperator::AssignDefault(default) => format!("${} //= '{}'", pe.variable, default),
            ParameterExpansionOperator::ErrorIfUnset(error) => format!("defined(${}) ? ${} : die('{}')", pe.variable, pe.variable, error),
            ParameterExpansionOperator::Basename => format!("basename(${})", pe.variable),
            ParameterExpansionOperator::Dirname => format!("dirname(${})", pe.variable),
        }
    }
}

// Helper trait for string escaping
trait StringEscaper {
    fn escape_perl_string(&self, s: &str) -> String;
    fn escape_perl_regex(&self, s: &str) -> String;
}

impl<T: SimpleCommandHandler> StringEscaper for T {
    fn escape_perl_string(&self, s: &str) -> String {
        // Handle strings that already contain escape sequences
        let mut result = String::new();
        
        for ch in s.chars() {
            match ch {
                '\n' => result.push_str("\\n"),
                '\t' => result.push_str("\\t"),
                '\r' => result.push_str("\\r"),
                '\x07' => result.push_str("\\a"),  // bell
                '\x08' => result.push_str("\\b"),  // backspace
                '\x0c' => result.push_str("\\f"),  // formfeed
                '\x0b' => result.push_str("\\x0b"), // vertical tab - use hex escape for Perl compatibility
                '\\' => result.push_str("\\\\"),
                '"' => result.push_str("\\\""),
                '\'' => result.push_str("\\'"),
                _ => result.push(ch),
            }
        }
        
        result
    }

    fn escape_perl_regex(&self, s: &str) -> String {
        s.chars().map(|c| match c {
            '\\' => "\\\\".to_string(),
            '/' => "\\/".to_string(),
            '^' => "\\^".to_string(),
            '$' => "\\$".to_string(),
            '.' => "\\.".to_string(),
            '*' => "\\*".to_string(),
            '+' => "\\+".to_string(),
            '?' => "\\?".to_string(),
            '(' => "\\(".to_string(),
            ')' => "\\)".to_string(),
            '[' => "\\[".to_string(),
            ']' => "\\]".to_string(),
            '{' => "\\{".to_string(),
            '}' => "\\}".to_string(),
            '|' => "\\|".to_string(),
            _ => c.to_string()
        }).collect()
    }
}

// Helper trait for string interpolation
trait StringInterpolationHandler {
    fn convert_string_interpolation_to_perl(&self, interp: &StringInterpolation) -> String;
}

impl<T: SimpleCommandHandler> StringInterpolationHandler for T {
    fn convert_string_interpolation_to_perl(&self, interp: &StringInterpolation) -> String {
        // This is a placeholder - the full implementation will be in the string_interpolation module
        format!("interpolation_placeholder_{}", interp.parts.len())
    }
}
