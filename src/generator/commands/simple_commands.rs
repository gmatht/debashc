use crate::ast::*;
use crate::generator::Generator;
use std::sync::atomic::{AtomicUsize, Ordering};

// Static counter for generating unique temp file names
static TEMP_FILE_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Dispatch to command-specific generators
fn generate_command_specific(generator: &mut Generator, cmd: &SimpleCommand, input_var: &str) -> Option<String> {
    let cmd_name = match &cmd.name {
        Word::Literal(s) => s,
        _ => return None
    };
    
    // Use default input variable for commands that need it
    let default_input = if input_var.is_empty() { "input_data" } else { input_var };
    
    match cmd_name.as_str() {
        "grep" => Some(super::grep::generate_grep_command(generator, cmd, default_input)),
        "cat" => Some(super::cat::generate_cat_command(generator, cmd)),
        "find" => Some(super::find::generate_find_command(generator, cmd)),
        "ls" => Some(super::ls::generate_ls_command(generator, cmd)),
        "wc" => Some(super::wc::generate_wc_command(generator, cmd, default_input)),
        "sort" => Some(super::sort::generate_sort_command(generator, cmd, default_input)),
        "uniq" => Some(super::uniq::generate_uniq_command(generator, cmd, default_input)),
        "xargs" => Some(super::xargs::generate_xargs_command(generator, cmd, default_input)),
        "awk" => Some(super::awk::generate_awk_command(generator, cmd, default_input)),
        "sed" => Some(super::sed::generate_sed_command(generator, cmd, default_input)),
        "comm" => Some(super::comm::generate_comm_command(generator, cmd, default_input)),
        "tr" => Some(super::tr::generate_tr_command(generator, cmd, default_input)),
        "sleep" => Some(super::sleep::generate_sleep_command(generator, cmd)),
        "cut" => Some(super::cut::generate_cut_command(generator, cmd, default_input)),
        "basename" => Some(super::basename::generate_basename_command(generator, cmd, default_input)),
        "dirname" => Some(super::dirname::generate_dirname_command(generator, cmd, default_input)),
        "date" => Some(super::date::generate_date_command(generator, cmd)),
        "time" => Some(super::time::generate_time_command(generator, cmd)),
        "wget" => Some(super::wget::generate_wget_command(generator, cmd)),
        "which" => Some(super::which::generate_which_command(generator, cmd)),
        "yes" => Some(super::yes::generate_yes_command(generator, cmd)),
        "zcat" => Some(super::zcat::generate_zcat_command(generator, cmd)),
        "strings" => Some(super::strings::generate_strings_command(generator, cmd, default_input)),
        "tee" => Some(super::tee::generate_tee_command(generator, cmd, default_input)),
        "sha256sum" => Some(super::sha256sum::generate_sha256sum_command(generator, cmd, default_input)),
        "sha512sum" => Some(super::sha512sum::generate_sha512sum_command(generator, cmd, default_input)),
        "gzip" => Some(super::gzip::generate_gzip_command(generator, cmd, default_input)),
        "kill" => Some(super::kill::generate_kill_command(generator, cmd)),
        "nohup" => Some(super::nohup::generate_nohup_command(generator, cmd)),
        "nice" => Some(super::nice::generate_nice_command(generator, cmd)),
        "curl" => Some(super::curl::generate_curl_command(generator, cmd)),
        "mkdir" => Some(super::mkdir::generate_mkdir_command(generator, cmd)),
        "rm" => Some(super::rm::generate_rm_command(generator, cmd)),
        "cp" => Some(super::cp::generate_cp_command(generator, cmd)),
        "mv" => Some(super::mv::generate_mv_command(generator, cmd)),
        "touch" => Some(super::touch::generate_touch_command(generator, cmd)),
        "head" => Some(super::head::generate_head_command(generator, cmd, default_input)),
        "tail" => Some(super::tail::generate_tail_command(generator, cmd, default_input)),
        _ => None
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
    let mut _has_here_string = false;
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
                _has_here_string = true;
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
                            // Check if the string is already quoted (starts and ends with same quote)
                            let trimmed = s.trim();
                            if (trimmed.starts_with("'") && trimmed.ends_with("'")) || 
                               (trimmed.starts_with("\"") && trimmed.ends_with("\"")) {
                                // Strip the surrounding quotes and escape for Perl
                                let content = &trimmed[1..trimmed.len()-1];
                                format!("\"{}\"", generator.escape_perl_string(content))
                            } else {
                                // Not quoted, but check if it contains already escaped quotes
                                // If the string contains \" or \', we need to handle it specially
                                if s.contains("\\\"") || s.contains("\\'") {
                                    // The string already has escaped quotes, don't double-escape
                                    // Just escape newlines and tabs, but preserve the existing quote escaping
                                    let escaped = s.replace("\n", "\\n")
                                                  .replace("\t", "\\t")
                                                  .replace("\r", "\\r");
                                    format!("\"{}\"", escaped)
                                } else {
                                    // Normal case, escape as-is
                                    format!("\"{}\"", generator.escape_perl_string(s))
                                }
                            }
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
        let cmd_name = match &cmd.name {
            Word::Literal(s) => s,
            _ => "unknown_command"
        };
        
        // First try to use command-specific generators
        if let Some(specific_output) = generate_command_specific(generator, cmd, "") {
            output.push_str(&specific_output);
        } else if generator.declared_functions.contains(cmd_name) {
            // Check if this is a function call
            if cmd.args.is_empty() {
                output.push_str(&format!("{}();\n", cmd_name));
            } else {
                let args: Vec<String> = cmd.args.iter()
                    .map(|arg| generator.word_to_perl(arg))
                    .collect();
                output.push_str(&format!("{}({});\n", cmd_name, args.join(", ")));
            }
        } else {
            // Fallback to system call
            if cmd.args.is_empty() {
                output.push_str(&format!("system('{}');\n", cmd_name));
            } else {
                let args: Vec<String> = cmd.args.iter()
                    .map(|arg| generator.word_to_perl(arg))
                    .collect();
                output.push_str(&format!("system('{}', {});\n", cmd_name, args.join(", ")));
            }
        }
    }

    if has_env {
        output.push_str("}\n");
    }

    output
}
