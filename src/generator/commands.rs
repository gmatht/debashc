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
        
        // Generate each command in the pipeline
        for (i, command) in pipeline.commands.iter().enumerate() {
            if i == 0 {
                // First command - capture its output
                output.push_str(&generator.indent());
                output.push_str("my $output = `");
                output.push_str(&generator.generate_command_string_for_system(command));
                output.push_str("`;\n");
            } else {
                // Subsequent commands - pipe previous output to them
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
            format!("{} {}", simple_cmd.name, args.join(" "))
        }
        Command::Subshell(subshell_cmd) => {
            match &**subshell_cmd {
                Command::Simple(simple_cmd) => {
                    let args: Vec<String> = simple_cmd.args.iter()
                        .map(|arg| generator.word_to_perl(arg))
                        .collect();
                    format!("{} {}", simple_cmd.name, args.join(" "))
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
