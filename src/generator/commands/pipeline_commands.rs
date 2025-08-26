use crate::ast::*;
use crate::generator::Generator;
use crate::generator::commands::{
    generate_cat_command,
    generate_find_command,
    generate_ls_command,
    generate_grep_command,
    generate_wc_command,
    generate_sort_command,
    generate_uniq_command,
    generate_awk_command,
    generate_sed_command,
    generate_comm_command,
    generate_xargs_command,
    generate_tr_command,
    generate_sleep_command,
    generate_cut_command,
    generate_basename_command,
    generate_dirname_command,
    generate_date_command,
    generate_time_command,
    generate_wget_command,
    generate_which_command,
    generate_yes_command,
    generate_zcat_command,
    generate_strings_command,
    generate_tee_command,
    generate_sha256sum_command,
    generate_sha512sum_command,
    generate_gzip_command,
    generate_kill_command,
    generate_nohup_command,
    generate_nice_command,
    generate_curl_command,
    generate_mkdir_command,
    generate_rm_command,
    generate_cp_command,
    generate_mv_command,
    generate_touch_command,
    generate_head_command,
    generate_tail_command,
};

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
            let cmd_name = match &cmd.name {
                Word::Literal(s) => s,
                _ => "unknown_command"
            };
            
            if cmd_name == "cat" {
                // Use the dedicated cat command function
                output.push_str(&generate_cat_command(generator, cmd));
            } else if cmd_name == "find" {
                // Use the dedicated find command function
                output.push_str(&generate_find_command(generator, cmd));
            } else if cmd_name == "ls" {
                // Use the dedicated ls command function
                output.push_str(&generate_ls_command(generator, cmd));
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
                let cmd_name = match &cmd.name {
                    Word::Literal(s) => s,
                    _ => "unknown_command"
                };
                
                if cmd_name == "grep" {
                    // Use the dedicated grep command function
                    output.push_str(&generate_grep_command(generator, cmd, "$output"));
                } else if cmd_name == "wc" {
                    // Use the dedicated wc command function
                    output.push_str(&generate_wc_command(generator, cmd, "$output"));
                } else if cmd_name == "sort" {
                    // Use the dedicated sort command function
                    output.push_str(&generate_sort_command(generator, cmd, "$output"));
                } else if cmd_name == "uniq" {
                    // Use the dedicated uniq command function
                    output.push_str(&generate_uniq_command(generator, cmd, "$output"));
                } else if cmd_name == "awk" {
                    // Use the dedicated awk command function
                    output.push_str(&generate_awk_command(generator, cmd, "$output"));
                } else if cmd_name == "sed" {
                    // Use the dedicated sed command function
                    output.push_str(&generate_sed_command(generator, cmd, "$output"));
                } else if cmd_name == "comm" {
                    // Use the dedicated comm command function
                    output.push_str(&generate_comm_command(generator, cmd, "$output"));
                } else if cmd_name == "tr" {
                    // Use the dedicated tr command function
                    output.push_str(&generate_tr_command(generator, cmd, "$output"));
                } else if cmd_name == "cut" {
                    // Use the dedicated cut command function
                    output.push_str(&generate_cut_command(generator, cmd, "$output"));
                } else if cmd_name == "basename" {
                    // Use the dedicated basename command function
                    output.push_str(&generate_basename_command(generator, cmd, "$output"));
                } else if cmd_name == "dirname" {
                    // Use the dedicated dirname command function
                    output.push_str(&generate_dirname_command(generator, cmd, "$output"));
                } else if cmd_name == "strings" {
                    // Use the dedicated strings command function
                    output.push_str(&generate_strings_command(generator, cmd, "$output"));
                } else if cmd_name == "tee" {
                    // Use the dedicated tee command function
                    output.push_str(&generate_tee_command(generator, cmd, "$output"));
                } else if cmd_name == "sha256sum" {
                    // Use the dedicated sha256sum command function
                    output.push_str(&generate_sha256sum_command(generator, cmd, "$output"));
                } else if cmd_name == "sha512sum" {
                    // Use the dedicated sha512sum command function
                    output.push_str(&generate_sha512sum_command(generator, cmd, "$output"));
                } else if cmd_name == "gzip" {
                    // Use the dedicated gzip command function
                    output.push_str(&generate_gzip_command(generator, cmd, "$output"));
                } else if cmd_name == "kill" {
                    // Use the dedicated kill command function
                    output.push_str(&generate_kill_command(generator, cmd));
                } else if cmd_name == "nohup" {
                    // Use the dedicated nohup command function
                    output.push_str(&generate_nohup_command(generator, cmd));
                } else if cmd_name == "nice" {
                    // Use the dedicated nice command function
                    output.push_str(&generate_nice_command(generator, cmd));
                } else if cmd_name == "curl" {
                    // Use the dedicated curl command function
                    output.push_str(&generate_curl_command(generator, cmd));
                } else if cmd_name == "mkdir" {
                    // Use the dedicated mkdir command function
                    output.push_str(&generate_mkdir_command(generator, cmd));
                } else if cmd_name == "rm" {
                    // Use the dedicated rm command function
                    output.push_str(&generate_rm_command(generator, cmd));
                } else if cmd_name == "cp" {
                    // Use the dedicated cp command function
                    output.push_str(&generate_cp_command(generator, cmd));
                } else if cmd_name == "mv" {
                    // Use the dedicated mv command function
                    output.push_str(&generate_mv_command(generator, cmd));
                } else if cmd_name == "touch" {
                    // Use the dedicated touch command function
                    output.push_str(&generate_touch_command(generator, cmd));
                } else if cmd_name == "head" {
                    // Use the dedicated head command function
                    output.push_str(&generate_head_command(generator, cmd, "$output"));
                } else if cmd_name == "tail" {
                    // Use the dedicated tail command function
                    output.push_str(&generate_tail_command(generator, cmd, "$output"));
                } else if cmd_name == "find" {
                    // Use the dedicated find command function
                    output.push_str(&generate_find_command(generator, cmd));
                } else if cmd_name == "xargs" {
                    // Use the dedicated xargs command function
                    output.push_str(&generate_xargs_command(generator, cmd, "$output"));
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
