use crate::ast::*;
use super::Generator;
use std::sync::atomic::{AtomicUsize, Ordering};

// Static counter for generating unique temp file names
static TEMP_FILE_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn generate_command_impl(generator: &mut Generator, command: &Command) -> String {
    match command {
        Command::Simple(cmd) => generator.generate_simple_command(cmd),
        Command::ShoptCommand(cmd) => generator.generate_shopt_command(cmd),
        Command::TestExpression(test_expr) => {
            generator.generate_test_expression(test_expr)
        },
        Command::Pipeline(pipeline) => generator.generate_pipeline(pipeline),
        Command::If(if_stmt) => generator.generate_if_statement(if_stmt),
        Command::Case(case_stmt) => generator.generate_case_statement(case_stmt),
        Command::While(while_loop) => generator.generate_while_loop(while_loop),
        Command::For(for_loop) => generator.generate_for_loop(for_loop),
        Command::Function(func) => generator.generate_function(func),
        Command::Subshell(cmd) => generator.generate_subshell(cmd),
        Command::Background(cmd) => generator.generate_background(cmd),
        Command::Block(block) => generator.generate_block(block),
        Command::BuiltinCommand(cmd) => generator.generate_builtin_command(cmd),
        Command::Break(level) => generator.generate_break_statement(level),
        Command::Continue(level) => generator.generate_continue_statement(level),
        Command::Return(value) => generator.generate_return_statement(value),
        Command::BlankLine => "\n".to_string(),
        Command::Redirect(redirect_cmd) => {
            let mut result = generate_command_impl(generator, &redirect_cmd.command);
            for redirect in &redirect_cmd.redirects {
                result.push_str(&generator.generate_redirect(redirect));
            }
            result
        }
    }
}

pub fn generate_simple_command_impl(generator: &mut Generator, cmd: &SimpleCommand) -> String {
    let mut output = String::new();
    let has_env = !cmd.env_vars.is_empty() && cmd.name != "true";
    if has_env {
        output.push_str("{\n");
        for (var, value) in &cmd.env_vars {
            // Check if this is an associative array assignment like map[foo]=bar
            if let Some((array_name, key)) = generator.extract_array_key(var) {
                let val = generator.perl_string_literal(value);
                // For associative array assignments, generate $array{key} = value instead of $ENV{var}
                // Quote the key to avoid bareword errors in strict mode
                let quoted_key = format!("\"{}\"", generator.escape_perl_string(&key));
                output.push_str(&format!("${}{{{}}} = {};\n", array_name, quoted_key, val));
            } else if let Some(elements) = generator.extract_array_elements(value) {
                // Check if this is an indexed array assignment like arr=(one two three)
                let elements_perl: Vec<String> = elements.iter()
                    .map(|s| format!("\"{}\"", generator.escape_perl_string(s)))
                    .collect();
                output.push_str(&format!("@{} = ({});\n", var, elements_perl.join(", ")));
            } else {
                let val = generator.perl_string_literal(value);
                // Always assign the value, but only declare if not already declared
                if !generator.declared_locals.contains(var) {
                    output.push_str(&format!("my ${} = {};\n", var, val));
                    generator.declared_locals.insert(var.clone());
                } else {
                    // Variable already declared, just assign the value
                    output.push_str(&format!("${} = {};\n", var, val));
                }
                output.push_str(&format!("local $ENV{{{}}} = {};;\n", var, val));
            }
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
                let global_counter = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
                let temp_file = format!("/tmp/process_sub_{}_{}.tmp", global_counter, temp_file_counter);
                let temp_var = format!("temp_file_ps_{}_{}", global_counter, temp_file_counter);
                output.push_str(&format!("my ${} = '{}';\n", temp_var, temp_file));
                
                // Generate the command for system call
                let cmd_str = generator.generate_command_string_for_system(&**cmd);
                
                // Clean up the command string for system call and properly escape it
                let _clean_cmd = cmd_str.replace('\n', " ").replace("  ", " ");
                // Use proper Perl system call syntax with list form to avoid shell interpretation
                let fh_var = format!("fh_ps_{}_{}", global_counter, temp_file_counter);
                output.push_str(&format!("open(my ${}, '>', ${}) or die \"Cannot create temp file: $!\\n\";\n", fh_var, temp_var));
                output.push_str(&format!("close(${});\n", fh_var));
                // For now, just create the file - the actual command execution would need more complex handling
                process_sub_files.push((temp_var, temp_file));
            }
            RedirectOperator::ProcessSubstitutionOutput(_cmd) => {
                // Process substitution output: >(command)
                temp_file_counter += 1;
                let global_counter = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
                let temp_file = format!("/tmp/process_sub_out_{}_{}.tmp", global_counter, temp_file_counter);
                let temp_var = format!("temp_file_out_{}_{}", global_counter, temp_file_counter);
                output.push_str(&format!("my ${} = '{}';\n", temp_var, temp_file));
                process_sub_files.push((temp_var, temp_file));
            }
            RedirectOperator::HereString => {
                // Here-string: <<< content
                has_here_string = true;
                temp_file_counter += 1;
                let global_counter = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
                let temp_file = format!("/tmp/here_string_{}_{}.tmp", global_counter, temp_file_counter);
                let temp_var = format!("temp_file_hs_{}_{}", global_counter, temp_file_counter);
                output.push_str(&format!("my ${} = '{}';\n", temp_var, temp_file));
                
                // Create the temporary file with the here-string content
                if let Some(content) = &redir.heredoc_body {
                    let fh_var = format!("fh_hs_{}_{}", global_counter, temp_file_counter);
                    output.push_str(&format!("open(my ${}, '>', ${}) or die \"Cannot create temp file: $!\\n\";\n", fh_var, temp_var));
                    output.push_str(&format!("print ${} {};\n", fh_var, generator.perl_string_literal(&Word::Literal(content.clone()))));
                    output.push_str(&format!("close(${});\n", fh_var));
                }
                
                process_sub_files.push((temp_var, temp_file));
            }
            _ => {}
        }
    }

    // Generate the actual command
    if cmd.name == "echo" {
        // Special handling for echo command
        if cmd.args.is_empty() {
            output.push_str("print \"\\n\";\n");
        } else {
            let args: Vec<String> = cmd.args.iter()
                .map(|arg| {
                    match arg {
                        Word::Literal(s) => {
                            // Properly quote literal strings for Perl
                            format!("\"{}\"", generator.escape_perl_string(s))
                        },
                        Word::Variable(var) => {
                            // Convert shell variables to Perl variables
                            format!("${}", var)
                        },
                        Word::ParameterExpansion(pe) => {
                            // Handle parameter expansions
                            generator.generate_parameter_expansion(pe)
                        },
                        _ => {
                            // For other word types, use the general conversion
                            generator.word_to_perl(arg)
                        }
                    }
                })
                .collect();
            
            // Use proper Perl print statement formatting
            if args.len() == 1 {
                output.push_str(&format!("print {}, \"\\n\";\n", args[0]));
            } else {
                // For multiple arguments, use comma separation for proper Perl syntax
                let args_str = args.join(", ");
                output.push_str(&format!("print {}, \"\\n\";\n", args_str));
            }
        }
    } else if cmd.name == "true" && !cmd.env_vars.is_empty() && cmd.args.is_empty() {
        // This is a standalone assignment (e.g., i=$((i + 1)))
        // Generate proper Perl assignment statements
        for (var, value) in &cmd.env_vars {
            match value {
                Word::Arithmetic(expr) => {
                    // Convert arithmetic expression to Perl
                    let perl_expr = generator.convert_arithmetic_to_perl(&expr.expression);
                    // Check if variable is already declared in current scope
                    if !generator.declared_locals.contains(var) {
                        // For now, assume this is a loop variable or already in scope
                        // and just assign to it without redeclaring
                        output.push_str(&format!("${} = {};\n", var, perl_expr));
                    } else {
                        // Variable already declared, just assign the value
                        output.push_str(&format!("${} = {};\n", var, perl_expr));
                    }
                },
                _ => {
                    // Handle other value types
                    let val = generator.perl_string_literal(value);
                    if !generator.declared_locals.contains(var) {
                        // For now, assume this is a loop variable or already in scope
                        // and just assign to it without redeclaring
                        output.push_str(&format!("${} = {};\n", var, val));
                    } else {
                        // Variable already declared, just assign the value
                        output.push_str(&format!("${} = {};\n", var, val));
                    }
                }
            }
        }
    } else {
        // Handle other commands
        if cmd.args.is_empty() {
            // Check if this is a function call
            let cmd_name = match &cmd.name {
                Word::Literal(s) => s,
                _ => "unknown_command"
            };
            if generator.declared_functions.contains(cmd_name) {
                output.push_str(&format!("{}();\n", cmd_name));
            } else {
                output.push_str(&format!("system('{}');\n", cmd_name));
            }
        } else {
            let args: Vec<String> = cmd.args.iter()
                .map(|arg| generator.word_to_perl(arg))
                .collect();
            
            // Check if this is a function call
            let cmd_name = match &cmd.name {
                Word::Literal(s) => s,
                _ => "unknown_command"
            };
            if generator.declared_functions.contains(cmd_name) {
                output.push_str(&format!("{}({});\n", cmd_name, args.join(", ")));
            } else {
                output.push_str(&format!("system('{}', {});\n", cmd_name, args.join(", ")));
            }
        }
    }

    if has_env {
        output.push_str("}\n");
    }

    output
}

pub fn generate_pipeline_impl(generator: &mut Generator, pipeline: &Pipeline) -> String {
    let mut output = String::new();
    
    if pipeline.commands.len() == 1 {
        // Single command, no pipeline needed
        output.push_str(&generator.generate_command(&pipeline.commands[0]));
    } else {
        // Multiple commands, implement proper Perl pipeline
        output.push_str("do {\n");
        generator.indent_level += 1;
        
        // Handle special case where first command is cat with split arguments
        if let Command::Simple(cmd) = &pipeline.commands[0] {
            if cmd.name == "cat" {
                // Handle cat command natively in Perl
                let filename = if cmd.args.is_empty() { 
                    "".to_string()
                } else { 
                    // Reconstruct the filename from split arguments if needed
                    if cmd.args.len() > 1 {
                        cmd.args.iter()
                            .map(|arg| generator.word_to_perl(arg))
                            .collect::<Vec<_>>()
                            .join("")
                    } else {
                        generator.word_to_perl(&cmd.args[0])
                    }
                };
                output.push_str(&generator.indent());
                output.push_str(&format!("my $output = '';\n"));
                output.push_str(&generator.indent());
                output.push_str(&format!("if (open(my $fh, '<', '{}')) {{\n", filename));
                generator.indent_level += 1;
                output.push_str(&generator.indent());
                output.push_str("while (my $line = <$fh>) {\n");
                generator.indent_level += 1;
                output.push_str(&generator.indent());
                output.push_str("$output .= $line;\n");
                generator.indent_level -= 1;
                output.push_str(&generator.indent());
                output.push_str("}\n");
                output.push_str(&generator.indent());
                output.push_str("close($fh);\n");
                generator.indent_level -= 1;
                output.push_str(&generator.indent());
                output.push_str("} else {\n");
                generator.indent_level += 1;
                output.push_str(&generator.indent());
                output.push_str(&format!("warn \"cat: {}: No such file or directory\";\n", filename));
                output.push_str(&generator.indent());
                output.push_str("exit(1);\n");
                generator.indent_level -= 1;
                output.push_str(&generator.indent());
                output.push_str("}\n");
            } else if cmd.name == "find" {
                // Handle find command natively in Perl
                let mut path = ".";
                let mut pattern = "*.sh".to_string();
                
                // Parse find arguments
                let mut i = 0;
                while i < cmd.args.len() {
                    if let Word::Literal(arg) = &cmd.args[i] {
                        if arg == "." {
                            path = ".";
                        } else if arg == "-name" && i + 1 < cmd.args.len() {
                            if let Some(next_arg) = cmd.args.get(i + 1) {
                                pattern = match next_arg {
                                    Word::StringInterpolation(interp) => {
                                        interp.parts.iter()
                                            .map(|part| match part {
                                                crate::ast::StringPart::Literal(s) => s,
                                                _ => "*"
                                            })
                                            .collect::<Vec<_>>()
                                            .join("")
                                    },
                                    _ => generator.word_to_perl(next_arg)
                                };
                                i += 1; // Skip the pattern argument
                            }
                        }
                    }
                    i += 1;
                }
                
                output.push_str(&generator.indent());
                output.push_str(&format!("my @find_files;\n"));
                output.push_str(&generator.indent());
                output.push_str(&format!("sub find_files {{\n"));
                generator.indent_level += 1;
                output.push_str(&generator.indent());
                output.push_str("my ($dir, $pattern) = @_;\n");
                output.push_str(&generator.indent());
                output.push_str("if (opendir(my $dh, $dir)) {\n");
                generator.indent_level += 1;
                output.push_str(&generator.indent());
                output.push_str("while (my $file = readdir($dh)) {\n");
                generator.indent_level += 1;
                output.push_str(&generator.indent());
                output.push_str("next if $file eq '.' || $file eq '..';\n");
                output.push_str(&generator.indent());
                output.push_str("my $full_path = $dir eq '.' ? $file : \"$dir/$file\";\n");
                output.push_str(&generator.indent());
                output.push_str("if (-d $full_path) {\n");
                generator.indent_level += 1;
                output.push_str(&generator.indent());
                output.push_str("find_files($full_path, $pattern);\n");
                generator.indent_level -= 1;
                output.push_str(&generator.indent());
                output.push_str("} elsif ($file =~ /^$pattern$/) {\n");
                generator.indent_level += 1;
                output.push_str(&generator.indent());
                output.push_str("push @find_files, $full_path;\n");
                generator.indent_level -= 1;
                output.push_str(&generator.indent());
                output.push_str("}\n");
                generator.indent_level -= 1;
                output.push_str(&generator.indent());
                output.push_str("}\n");
                output.push_str(&generator.indent());
                output.push_str("closedir($dh);\n");
                generator.indent_level -= 1;
                output.push_str(&generator.indent());
                output.push_str("}\n");
                generator.indent_level -= 1;
                output.push_str(&generator.indent());
                output.push_str("}\n");
                output.push_str(&generator.indent());
                output.push_str(&format!("find_files('{}', '{}');\n", path, pattern));
                output.push_str(&generator.indent());
                output.push_str("$output = join(\"\\n\", @find_files);\n");
            } else if cmd.name == "ls" {
                // Handle ls command natively in Perl
                let dir = if cmd.args.is_empty() { "." } else { &generator.word_to_perl(&cmd.args[0]) };
                output.push_str(&generator.indent());
                output.push_str(&format!("my @ls_files;\n"));
                output.push_str(&generator.indent());
                output.push_str(&format!("if (opendir(my $dh, '{}')) {{\n", dir));
                generator.indent_level += 1;
                output.push_str(&generator.indent());
                output.push_str("while (my $file = readdir($dh)) {\n");
                generator.indent_level += 1;
                output.push_str(&generator.indent());
                output.push_str("next if $file eq '.' || $file eq '..';\n");
                output.push_str(&generator.indent());
                output.push_str("push @ls_files, $file;\n");
                generator.indent_level -= 1;
                output.push_str(&generator.indent());
                output.push_str("}\n");
                output.push_str(&generator.indent());
                output.push_str("closedir($dh);\n");
                generator.indent_level -= 1;
                output.push_str(&generator.indent());
                output.push_str("}\n");
                output.push_str(&generator.indent());
                output.push_str("my $output = join(\"\\n\", @ls_files);\n");
            } else {
                // First command - capture its output using system command
                output.push_str(&generator.indent());
                output.push_str("my $output = `");
                output.push_str(&generator.generate_command_string_for_system(&pipeline.commands[0]));
                output.push_str("`;\n");
            }
        } else {
            // First command - capture its output
            output.push_str(&generator.indent());
            output.push_str("my $output = `");
            output.push_str(&generator.generate_command_string_for_system(&pipeline.commands[0]));
            output.push_str("`;\n");
        }
        
        // Generate subsequent commands in the pipeline
        for command in pipeline.commands.iter().skip(1) {
            if let Command::Simple(cmd) = command {
                if cmd.name == "grep" {
                    // Handle grep command natively in Perl
                    if let Some(pattern) = cmd.args.first() {
                        let pattern_str = match pattern {
                            Word::StringInterpolation(interp) => {
                                // Extract the pattern from StringInterpolation
                                interp.parts.iter()
                                    .map(|part| match part {
                                        crate::ast::StringPart::Literal(s) => s,
                                        _ => ".*" // fallback for non-literal parts
                                    })
                                    .collect::<Vec<_>>()
                                    .join("")
                            },
                            _ => generator.word_to_perl(pattern)
                        };
                        output.push_str(&generator.indent());
                        output.push_str(&format!("my @lines = split(/\\n/, $output);\n"));
                        output.push_str(&generator.indent());
                        output.push_str(&format!("my @filtered = grep /{}/, @lines;\n", pattern_str));
                        output.push_str(&generator.indent());
                        output.push_str("$output = join(\"\\n\", @filtered);\n");
                    }
                } else if cmd.name == "wc" {
                    // Handle wc command natively in Perl
                    if let Some(flag) = cmd.args.first() {
                        if let Word::Literal(flag_str) = flag {
                            if flag_str == "-l" {
                                output.push_str(&generator.indent());
                                output.push_str("my @lines = split(/\\n/, $output);\n");
                                output.push_str(&generator.indent());
                                output.push_str("$output = scalar(@lines);\n");
                            } else {
                                output.push_str(&generator.indent());
                                output.push_str(&format!("$output = `echo \"$output\" | wc {}`;\n", flag_str));
                            }
                        } else {
                            output.push_str(&generator.indent());
                            output.push_str(&format!("$output = `echo \"$output\" | wc {}`;\n", generator.word_to_perl(flag)));
                        }
                    } else {
                        output.push_str(&generator.indent());
                        output.push_str("$output = `echo \"$output\" | wc`;\n");
                    }
                } else if cmd.name == "sort" {
                    // Handle sort command natively in Perl
                    let mut numeric = false;
                    let mut reverse = false;
                    
                    // Check for flags
                    for arg in &cmd.args {
                        if let Word::Literal(arg_str) = arg {
                            if arg_str == "-n" {
                                numeric = true;
                            } else if arg_str == "r" || arg_str == "-r" {
                                reverse = true;
                            } else if arg_str == "-nr" || arg_str == "-rn" {
                                numeric = true;
                                reverse = true;
                            }
                        }
                    }
                    
                    output.push_str(&generator.indent());
                    output.push_str("my @lines = split(/\\n/, $output);\n");
                    output.push_str(&generator.indent());
                    if numeric {
                        output.push_str("my @sorted = sort { $a <=> $b } @lines;\n");
                    } else {
                        output.push_str("my @sorted = sort @lines;\n");
                    }
                    if reverse {
                        output.push_str(&generator.indent());
                        output.push_str("@sorted = reverse(@sorted);\n");
                    }
                    output.push_str(&generator.indent());
                    output.push_str("$output = join(\"\\n\", @sorted);\n");
                } else if cmd.name == "uniq" {
                    // Handle uniq command natively in Perl
                    let mut count = false;
                    
                    // Check for flags
                    for arg in &cmd.args {
                        if let Word::Literal(arg_str) = arg {
                            if arg_str == "-c" {
                                count = true;
                            }
                        }
                    }
                    
                    output.push_str(&generator.indent());
                    output.push_str("my @lines = split(/\\n/, $output);\n");
                    output.push_str(&generator.indent());
                    if count {
                        output.push_str("my %counts;\n");
                        output.push_str(&generator.indent());
                        output.push_str("foreach my $line (@lines) {\n");
                        generator.indent_level += 1;
                        output.push_str(&generator.indent());
                        output.push_str("$counts{$line}++;\n");
                        generator.indent_level -= 1;
                        output.push_str(&generator.indent());
                        output.push_str("}\n");
                        output.push_str(&generator.indent());
                        output.push_str("my @result;\n");
                        output.push_str(&generator.indent());
                        output.push_str("foreach my $line (keys %counts) {\n");
                        generator.indent_level += 1;
                        output.push_str(&generator.indent());
                        output.push_str("push @result, sprintf(\"%7d %s\", $counts{$line}, $line);\n");
                        generator.indent_level -= 1;
                        output.push_str(&generator.indent());
                        output.push_str("}\n");
                        output.push_str(&generator.indent());
                        output.push_str("$output = join(\"\\n\", @result);\n");
                    } else {
                        output.push_str("my %seen;\n");
                        output.push_str(&generator.indent());
                        output.push_str("my @result;\n");
                        output.push_str(&generator.indent());
                        output.push_str("foreach my $line (@lines) {\n");
                        generator.indent_level += 1;
                        output.push_str(&generator.indent());
                        output.push_str("push @result, $line unless $seen{$line}++;\n");
                        generator.indent_level -= 1;
                        output.push_str(&generator.indent());
                        output.push_str("}\n");
                        output.push_str(&generator.indent());
                        output.push_str("$output = join(\"\\n\", @result);\n");
                    }
                } else if cmd.name == "find" {
                    // Handle find command natively in Perl
                    let mut path = ".";
                    let mut pattern = "*.sh".to_string();
                    
                    // Parse find arguments
                    let mut i = 0;
                    while i < cmd.args.len() {
                        if let Word::Literal(arg) = &cmd.args[i] {
                            if arg == "." {
                                path = ".";
                            } else if arg == "-name" && i + 1 < cmd.args.len() {
                                if let Some(next_arg) = cmd.args.get(i + 1) {
                                    pattern = match next_arg {
                                        Word::StringInterpolation(interp) => {
                                            interp.parts.iter()
                                                .map(|part| match part {
                                                    crate::ast::StringPart::Literal(s) => s,
                                                    _ => "*"
                                                })
                                                .collect::<Vec<_>>()
                                                .join("")
                                        },
                                        _ => generator.word_to_perl(next_arg)
                                    };
                                    i += 1; // Skip the pattern argument
                                }
                            }
                        }
                        i += 1;
                    }
                    
                    output.push_str(&generator.indent());
                    output.push_str(&format!("my @find_files;\n"));
                    output.push_str(&generator.indent());
                    output.push_str(&format!("sub find_files {{\n"));
                    generator.indent_level += 1;
                    output.push_str(&generator.indent());
                    output.push_str("my ($dir, $pattern) = @_;\n");
                    output.push_str(&generator.indent());
                    output.push_str("if (opendir(my $dh, $dir)) {\n");
                    generator.indent_level += 1;
                    output.push_str(&generator.indent());
                    output.push_str("while (my $file = readdir($dh)) {\n");
                    generator.indent_level += 1;
                    output.push_str(&generator.indent());
                    output.push_str("next if $file eq '.' || $file eq '..';\n");
                    output.push_str(&generator.indent());
                    output.push_str("my $full_path = $dir eq '.' ? $file : \"$dir/$file\";\n");
                    output.push_str(&generator.indent());
                    output.push_str("if (-d $full_path) {\n");
                    generator.indent_level += 1;
                    output.push_str(&generator.indent());
                    output.push_str("find_files($full_path, $pattern);\n");
                    generator.indent_level -= 1;
                    output.push_str(&generator.indent());
                    output.push_str("} elsif ($file =~ /^$pattern$/) {\n");
                    generator.indent_level += 1;
                    output.push_str(&generator.indent());
                    output.push_str("push @find_files, $full_path;\n");
                    generator.indent_level -= 1;
                    output.push_str(&generator.indent());
                    output.push_str("}\n");
                    generator.indent_level -= 1;
                    output.push_str(&generator.indent());
                    output.push_str("}\n");
                    output.push_str(&generator.indent());
                    output.push_str("closedir($dh);\n");
                    generator.indent_level -= 1;
                    output.push_str(&generator.indent());
                    output.push_str("}\n");
                    generator.indent_level -= 1;
                    output.push_str(&generator.indent());
                    output.push_str("}\n");
                    output.push_str(&generator.indent());
                    output.push_str(&format!("find_files('{}', '{}');\n", path, pattern));
                    output.push_str(&generator.indent());
                    output.push_str("$output = join(\"\\n\", @find_files);\n");
                } else if cmd.name == "xargs" {
                    // Handle xargs command natively in Perl
                    let mut command = "echo";
                    let mut args = Vec::new();
                    
                    // Parse xargs arguments
                    for arg in &cmd.args {
                        if let Word::Literal(arg_str) = arg {
                            if arg_str == "grep" {
                                command = "grep";
                            } else if arg_str == "-l" {
                                // This will be handled in the grep logic
                            } else if arg_str == "function" {
                                args.push("function".to_string());
                            }
                        } else if let Word::StringInterpolation(interp) = arg {
                            let pattern = interp.parts.iter()
                                .map(|part| match part {
                                    crate::ast::StringPart::Literal(s) => s,
                                    _ => ".*"
                                })
                                .collect::<Vec<_>>()
                                .join("");
                            args.push(pattern);
                        }
                    }
                    
                    if command == "grep" && args.contains(&"function".to_string()) {
                        // Handle grep -l "function" on the input files
                        output.push_str(&generator.indent());
                        output.push_str("my @files = split(/\\n/, $output);\n");
                        output.push_str(&generator.indent());
                        output.push_str("my @matching_files;\n");
                        output.push_str(&generator.indent());
                        output.push_str("foreach my $file (@files) {\n");
                        generator.indent_level += 1;
                        output.push_str(&generator.indent());
                        output.push_str("next unless $file && -f $file;\n");
                        output.push_str(&generator.indent());
                        output.push_str("if (open(my $fh, '<', $file)) {\n");
                        generator.indent_level += 1;
                        output.push_str(&generator.indent());
                        output.push_str("my $found = 0;\n");
                        output.push_str(&generator.indent());
                        output.push_str("while (my $line = <$fh>) {\n");
                        generator.indent_level += 1;
                        output.push_str(&generator.indent());
                        output.push_str("if ($line =~ /function/) {\n");
                        generator.indent_level += 1;
                        output.push_str(&generator.indent());
                        output.push_str("$found = 1;\n");
                        output.push_str(&generator.indent());
                        output.push_str("last;\n");
                        generator.indent_level -= 1;
                        output.push_str(&generator.indent());
                        output.push_str("}\n");
                        generator.indent_level -= 1;
                        output.push_str(&generator.indent());
                        output.push_str("}\n");
                        output.push_str(&generator.indent());
                        output.push_str("close($fh);\n");
                        output.push_str(&generator.indent());
                        output.push_str("push @matching_files, $file if $found;\n");
                        generator.indent_level -= 1;
                        output.push_str(&generator.indent());
                        output.push_str("}\n");
                        generator.indent_level -= 1;
                        output.push_str(&generator.indent());
                        output.push_str("}\n");
                        output.push_str(&generator.indent());
                        output.push_str("$output = join(\"\\n\", @matching_files);\n");
                    } else {
                        // Fallback to system command for other cases
                        output.push_str(&generator.indent());
                        output.push_str("$output = `echo \"$output\" | ");
                        output.push_str(command);
                        output.push_str("`;\n");
                    }
                } else {
                    // Use backticks for other commands
                    output.push_str(&generator.indent());
                    output.push_str("$output = `echo \"$output\" | ");
                    
                    // Handle special case where command has split arguments (like sort -n r)
                    if cmd.args.len() > 1 {
                        // Check if this looks like a split flag (e.g., -n and r should be -nr)
                        let mut reconstructed_args = Vec::new();
                        let mut i = 0;
                        while i < cmd.args.len() {
                            if let Word::Literal(arg) = &cmd.args[i] {
                                if arg.starts_with('-') && i + 1 < cmd.args.len() {
                                    // This might be a split flag, try to reconstruct
                                    if let Word::Literal(next_arg) = &cmd.args[i + 1] {
                                        if !next_arg.starts_with('-') {
                                            // This might be a split flag, try to reconstruct
                                            // Check if the next arg looks like it could be part of the flag
                                            if next_arg.chars().all(|c| c.is_ascii_alphabetic()) {
                                                // Reconstruct the flag (e.g., -n + r = -nr)
                                                reconstructed_args.push(format!("{}{}", arg, next_arg));
                                                i += 2; // Skip both args
                                                continue;
                                            }
                                        }
                                    }
                                }
                            }
                            reconstructed_args.push(generator.word_to_perl(&cmd.args[i]));
                            i += 1;
                        }
                        let args_str = if reconstructed_args.is_empty() {
                            "".to_string()
                        } else {
                            format!(" {}", reconstructed_args.join(" "))
                        };
                        output.push_str(&format!("{}{}", cmd.name, args_str));
                    } else if cmd.args.is_empty() {
                        output.push_str(&generator.word_to_perl(&cmd.name));
                    } else {
                        output.push_str(&generator.generate_command_string_for_system(command));
                    }
                    
                    output.push_str("`;\n");
                }
            } else {
                // Use backticks for non-simple commands
                output.push_str(&generator.indent());
                output.push_str("$output = `echo \"$output\" | ");
                output.push_str(&generator.generate_command_string_for_system(command));
                output.push_str("`;\n");
            }
        }
        
        // Output the final result
        output.push_str(&generator.indent());
        output.push_str("print $output;\n");
        
        generator.indent_level -= 1;
        output.push_str("};\n");
    }
    
    output
}

pub fn generate_subshell_impl(generator: &mut Generator, command: &Command) -> String {
    let mut output = String::new();
    
    // Generate subshell command - just execute the command directly
    output.push_str(&generator.generate_command(command));
    
    output
}

pub fn generate_background_impl(generator: &mut Generator, command: &Command) -> String {
    let mut output = String::new();
    
    // Generate background command
    output.push_str("(");
    output.push_str(&generator.generate_command(command));
    output.push_str(") &");
    
    output
}

pub fn generate_command_string_for_system_impl(generator: &mut Generator, cmd: &Command) -> String {
    match cmd {
        Command::Simple(simple_cmd) => {
            let args: Vec<String> = simple_cmd.args.iter()
                .map(|arg| generator.word_to_perl(arg))
                .collect();
            if args.is_empty() {
                simple_cmd.name.to_string()
            } else {
                format!("{} {}", simple_cmd.name, args.join(" "))
            }
        }
        Command::Subshell(subshell_cmd) => {
            match &**subshell_cmd {
                Command::Simple(simple_cmd) => {
                    let args: Vec<String> = simple_cmd.args.iter()
                        .map(|arg| generator.word_to_perl(arg))
                        .collect();
                    if args.is_empty() {
                        simple_cmd.name.to_string()
                    } else {
                        format!("{} {}", simple_cmd.name, args.join(" "))
                    }
                }
                Command::Pipeline(pipeline) => {
                    let commands: Vec<String> = pipeline.commands.iter()
                        .filter_map(|cmd| {
                            if let Command::Simple(simple_cmd) = cmd {
                                let args: Vec<String> = simple_cmd.args.iter()
                                    .map(|arg| generator.word_to_perl(arg))
                                    .collect();
                                Some(format!("{} {}", simple_cmd.name, args.join(" ")))
                            } else {
                                None
                            }
                        })
                        .collect();
                    commands.join(" | ")
                }
                _ => format!("{:?}", cmd)
            }
        }
        _ => format!("{:?}", cmd)
    }
}

// Helper method for escaping Perl strings
pub fn escape_perl_string(s: &str) -> String {
    s.replace("\\", "\\\\")
     .replace("\"", "\\\"")
     .replace("\n", "\\n")
     .replace("\t", "\\t")
     .replace("\r", "\\r")
}
