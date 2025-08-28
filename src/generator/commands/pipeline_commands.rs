use crate::generator::Generator;
use crate::ast::*;

pub fn generate_pipeline_impl(generator: &mut Generator, pipeline: &Pipeline) -> String {
    generate_pipeline_with_print_option(generator, pipeline, true)
}

pub fn generate_pipeline_with_print_option(generator: &mut Generator, pipeline: &Pipeline, should_print: bool) -> String {
    let mut output = String::new();
    
    if pipeline.commands.len() == 1 {
        // Single command, no pipeline needed
        output.push_str(&generator.generate_command(&pipeline.commands[0]));
    } else {
        // Multiple commands, implement proper Perl pipeline
        // Wrap in do block for proper scoping
        output.push_str("do {\n");
        generator.indent_level += 1;
        
        // Declare output variable in this scope
        output.push_str(&generator.indent());
        output.push_str("my $output;\n");
        
        for (i, command) in pipeline.commands.iter().enumerate() {
            if i > 0 {
                output.push_str("\n");
            }
            
            if i == 0 {
                // First command - generate output
                if let Command::Simple(cmd) = command {
                    let cmd_name = match &cmd.name {
                        Word::Literal(s) => s,
                        _ => "unknown_command"
                    };
                    
                    if cmd_name == "ls" {
                        output.push_str(&generator.indent());
                        output.push_str(&generate_ls_command(generator, cmd, true));
                        output.push_str(&generator.indent());
                        output.push_str("$output = join(\"\\n\", @ls_files);\n");
                    } else if cmd_name == "cat" {
                        output.push_str(&generator.indent());
                        output.push_str(&generate_cat_command(generator, cmd, &cmd.redirects));
                        // cat command already sets $output
                    } else if cmd_name == "find" {
                        output.push_str(&generator.indent());
                        output.push_str(&generate_find_command(generator, cmd, true));
                        // find command already sets $output when generate_output is true
                    } else {
                        // Generic first command
                        output.push_str(&generator.indent());
                        output.push_str("$output = `");
                        output.push_str(&generator.generate_command_string_for_system(command));
                        output.push_str("`;\n");
                    }
                } else {
                    // Non-simple first command
                    output.push_str(&generator.indent());
                    output.push_str("$output = `");
                    output.push_str(&generator.generate_command_string_for_system(command));
                    output.push_str("`;\n");
                }
            } else {
                // Subsequent commands - process the output from previous command
                if let Command::Simple(cmd) = command {
                    let cmd_name = match &cmd.name {
                        Word::Literal(s) => s,
                        _ => "unknown_command"
                    };
                    
                    if cmd_name == "grep" {
                        output.push_str(&generator.indent());
                        output.push_str(&generate_grep_command(generator, cmd, "$output", i));
                        // grep command modifies $output directly
                    } else if cmd_name == "wc" {
                        output.push_str(&generator.indent());
                        output.push_str(&generate_wc_command(generator, cmd, "$output", i));
                        // wc command modifies $output directly
                    } else if cmd_name == "sort" {
                        output.push_str(&generator.indent());
                        output.push_str(&generate_sort_command(generator, cmd, "$output", i));
                        // sort command modifies $output directly
                    } else if cmd_name == "uniq" {
                        output.push_str(&generator.indent());
                        output.push_str(&generate_uniq_command(generator, cmd, "$output", i));
                        // uniq command modifies $output directly
                    } else if cmd_name == "awk" {
                        output.push_str(&generator.indent());
                        output.push_str(&generate_awk_command(generator, cmd, "$output", i));
                        // awk command modifies $output directly
                    } else if cmd_name == "sed" {
                        output.push_str(&generator.indent());
                        output.push_str(&generate_sed_command(generator, cmd, "$output", i));
                        // sed command modifies $output directly
                    } else if cmd_name == "comm" {
                        output.push_str(&generator.indent());
                        output.push_str(&generate_comm_command(generator, cmd, "$output", i));
                        // comm command modifies $output directly
                    } else if cmd_name == "tr" {
                        output.push_str(&generator.indent());
                        output.push_str(&generate_tr_command(generator, cmd, "$output", i));
                        // tr command modifies $output directly
                    } else if cmd_name == "find" {
                        output.push_str(&generator.indent());
                        output.push_str(&generate_find_command(generator, cmd, false));
                        // find command modifies $output directly
                    } else if cmd_name == "cut" {
                        output.push_str(&generator.indent());
                        output.push_str(&generate_cut_command(generator, cmd, "$output", i));
                        // cut command modifies $output directly
                    } else if cmd_name == "basename" {
                        output.push_str(&generator.indent());
                        output.push_str(&generate_basename_command(generator, cmd, "$output"));
                        // basename command modifies $output directly
                    } else if cmd_name == "dirname" {
                        output.push_str(&generator.indent());
                        output.push_str(&generate_dirname_command(generator, cmd, "$output"));
                        // dirname command modifies $output directly
                    } else if cmd_name == "strings" {
                        output.push_str(&generator.indent());
                        output.push_str(&generate_strings_command(generator, cmd, "$output"));
                        // strings command modifies $output directly
                    } else if cmd_name == "tee" {
                        output.push_str(&generator.indent());
                        output.push_str(&generate_tee_command(generator, cmd, "$output"));
                        // tee command modifies $output directly
                    } else if cmd_name == "sha256sum" {
                        output.push_str(&generator.indent());
                        output.push_str(&generate_sha256sum_command(generator, cmd, "$output"));
                        // sha256sum command modifies $output directly
                    } else if cmd_name == "sha512sum" {
                        output.push_str(&generator.indent());
                        output.push_str(&generate_sha512sum_command(generator, cmd, "$output"));
                        // sha512sum command modifies $output directly
                    } else if cmd_name == "gzip" {
                        output.push_str(&generator.indent());
                        output.push_str(&generate_gzip_command(generator, cmd, "$output"));
                        // gzip command modifies $output directly
                    } else if cmd_name == "kill" {
                        output.push_str(&generator.indent());
                        output.push_str(&generate_kill_command(generator, cmd));
                        // kill doesn't produce output, so keep previous output
                    } else if cmd_name == "nohup" {
                        output.push_str(&generator.indent());
                        output.push_str(&generate_nohup_command(generator, cmd));
                        // nohup doesn't produce output, so keep previous output
                    } else if cmd_name == "nice" {
                        output.push_str(&generator.indent());
                        output.push_str(&generate_nice_command(generator, cmd));
                        // nice doesn't produce output, so keep previous output
                    } else if cmd_name == "curl" {
                        output.push_str(&generator.indent());
                        output.push_str(&generate_curl_command(generator, cmd));
                        // curl command modifies $output directly
                    } else if cmd_name == "mkdir" {
                        output.push_str(&generator.indent());
                        output.push_str(&generate_mkdir_command(generator, cmd));
                        // mkdir doesn't produce output, so keep previous output
                    } else if cmd_name == "rm" {
                        output.push_str(&generator.indent());
                        output.push_str(&generate_rm_command(generator, cmd));
                        // rm doesn't produce output, so keep previous output
                    } else if cmd_name == "cp" {
                        output.push_str(&generator.indent());
                        output.push_str(&generate_cp_command(generator, cmd));
                        // cp doesn't produce output, so keep previous output
                    } else if cmd_name == "mv" {
                        output.push_str(&generator.indent());
                        output.push_str(&generate_mv_command(generator, cmd));
                        // mv doesn't produce output, so keep previous output
                    } else if cmd_name == "touch" {
                        output.push_str(&generator.indent());
                        output.push_str(&generate_touch_command(generator, cmd));
                        // touch doesn't produce output, so keep previous output
                    } else if cmd_name == "head" {
                        output.push_str(&generator.indent());
                        output.push_str(&generate_head_command(generator, cmd, "$output", i));
                        // head command modifies $output directly
                    } else if cmd_name == "tail" {
                        output.push_str(&generator.indent());
                        output.push_str(&generate_tail_command(generator, cmd, "$output", i));
                        // tail command modifies $output directly
                    } else if cmd_name == "xargs" {
                        output.push_str(&generator.indent());
                        output.push_str(&generate_xargs_command(generator, cmd, "$output", i));
                        // xargs command modifies $output directly
                    } else {
                        // Generic command
                        output.push_str(&generator.indent());
                        output.push_str("my $new_output = `");
                        output.push_str(&generator.generate_command_string_for_system(command));
                        output.push_str("`;\n");
                        output.push_str(&generator.indent());
                        output.push_str("$output = $new_output;\n");
                    }
                } else {
                    // Non-simple command
                    output.push_str(&generator.indent());
                    output.push_str("my $new_output = `");
                    output.push_str(&generator.generate_command_string_for_system(command));
                    output.push_str("`;\n");
                    output.push_str(&generator.indent());
                    output.push_str("$output = $new_output;\n");
                }
            }
        }
        
        if should_print {
            // Output the final result
            output.push_str(&generator.indent());
            output.push_str("print $output;\n");
            output.push_str(&generator.indent());
            output.push_str("print \"\\n\";\n");
        } else {
            // Just return the output value for command substitution
            output.push_str(&generator.indent());
            output.push_str("$output;\n");
        }
        
        // Close the do block
        generator.indent_level -= 1;
        output.push_str(&generator.indent());
        output.push_str("};\n");
    }
    
    output
}

// Import all the command generation functions
use super::cat::generate_cat_command;
use super::find::generate_find_command;
use super::ls::generate_ls_command;
use super::grep::generate_grep_command;
use super::wc::generate_wc_command;
use super::sort::generate_sort_command;
use super::uniq::generate_uniq_command;
use super::awk::generate_awk_command;
use super::sed::generate_sed_command;
use super::comm::generate_comm_command;
use super::tr::generate_tr_command;
use super::cut::generate_cut_command;
use super::basename::generate_basename_command;
use super::dirname::generate_dirname_command;
use super::strings::generate_strings_command;
use super::tee::generate_tee_command;
use super::sha256sum::generate_sha256sum_command;
use super::sha512sum::generate_sha512sum_command;
use super::gzip::generate_gzip_command;
use super::kill::generate_kill_command;
use super::nohup::generate_nohup_command;
use super::nice::generate_nice_command;
use super::curl::generate_curl_command;
use super::mkdir::generate_mkdir_command;
use super::rm::generate_rm_command;
use super::cp::generate_cp_command;
use super::mv::generate_mv_command;
use super::touch::generate_touch_command;
use super::head::generate_head_command;
use super::tail::generate_tail_command;
use super::xargs::generate_xargs_command;
