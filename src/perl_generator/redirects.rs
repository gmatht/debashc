use crate::ast::*;
use super::PerlGenerator;

impl PerlGenerator {
    pub fn generate_redirect(&mut self, redirect: &Redirect) -> String {
        let mut output = String::new();
        
        match redirect.operator {
            RedirectOperator::Input => {
                // Input redirection: command < file
                output.push_str(&format!("open(STDIN, '<', '{}') or die \"Cannot open file: $!\\n\";\n", redirect.target));
            }
            RedirectOperator::Output => {
                // Output redirection: command > file
                // Note: This function doesn't have access to the command name, so it can't handle echo specially
                // The special handling is done in generate_simple_command
                output.push_str(&format!("open(STDOUT, '>', '{}') or die \"Cannot open file: $!\\n\";\n", redirect.target));
            }
            RedirectOperator::Append => {
                // Append redirection: command >> file
                output.push_str(&format!("open(STDOUT, '>>', '{}') or die \"Cannot open file: $!\\n\";\n", redirect.target));
            }
            RedirectOperator::Heredoc | RedirectOperator::HeredocTabs => {
                // Heredoc: command << delimiter
                if let Some(body) = &redirect.heredoc_body {
                    // Create a temporary file with the heredoc content
                    output.push_str(&format!("my $temp_content = {};\n", self.perl_string_literal(&Word::Literal(body.clone()))));
                    let fh = self.get_unique_file_handle();
                    output.push_str(&format!("open(my {}, '>', '/tmp/heredoc_temp') or die \"Cannot create temp file: $!\\n\";\n", fh));
                    output.push_str(&format!("print {} $temp_content;\n", fh));
                    output.push_str(&format!("close({});\n", fh));
                    output.push_str("open(STDIN, '<', '/tmp/heredoc_temp') or die \"Cannot open temp file: $!\\n\";\n");
                }
            }
            _ => {
                // Other redirects not yet implemented
                output.push_str(&format!("# Redirect {:?} not yet implemented\n", redirect.operator));
            }
        }
        
        output
    }

    pub fn generate_shopt_command(&mut self, cmd: &ShoptCommand) -> String {
        let mut output = String::new();
        
        // Handle shopt command for shell options
        let action = if cmd.enable { "enabled" } else { "disabled" };
        let comment = match cmd.option.as_str() {
            "extglob" | "nocasematch" => format!("# {} option {}", cmd.option, action),
            _ => format!("# shopt -{} {} not implemented", if cmd.enable { "s" } else { "u" }, cmd.option),
        };
        output.push_str(&format!("{}\n", comment));
        
        // shopt commands always succeed (return true)
        output
    }
    
    pub fn generate_builtin_command(&mut self, cmd: &BuiltinCommand) -> String {
        let mut output = String::new();
        
        // Handle environment variables if any
        let has_env = !cmd.env_vars.is_empty();
        if has_env {
            output.push_str("{\n");
            for (var, value) in &cmd.env_vars {
                // Check if this is an associative array assignment like map[foo]=bar
                if let Some((array_name, key)) = self.extract_array_key(var) {
                    let val = self.perl_string_literal(value);
                    // For associative array assignments, generate $array{key} = value instead of $ENV{var}
                    output.push_str(&format!("${}{{{}}} = {};\n", array_name, key, val));
                } else if let Some(elements) = self.extract_array_elements(value) {
                    // Check if this is an indexed array assignment like arr=(one two three)
                    let elements_perl: Vec<String> = elements.iter()
                        .map(|s| format!("\"{}\"", self.escape_perl_string(s)))
                        .collect();
                    output.push_str(&format!("@{} = ({});\n", var, elements_perl.join(", ")));
                } else {
                    let val = self.perl_string_literal(value);
                    // Declare the variable if it's not already declared
                    if !self.declared_locals.contains(var) {
                        output.push_str(&format!("my ${} = {};\n", var, val));
                        self.declared_locals.insert(var.clone());
                    } else {
                        // Variable already declared, just assign the value
                        output.push_str(&format!("${} = {};\n", var, val));
                    }
                    output.push_str(&format!("local $ENV{{{}}} = {};;\n", var, val));
                }
            }
        }
        
        // Generate the builtin command
        match cmd.name.as_str() {
            "set" => {
                // Convert shell set options to Perl equivalents
                for arg in &cmd.args {
                    if let Word::Literal(opt) = arg {
                        match opt.as_str() {
                            "-e" => output.push_str("$SIG{__DIE__} = sub { exit 1 };\n"),
                            "-u" => output.push_str("use strict;\n"),
                            "-o" => {
                                // Handle pipefail and other options
                                if let Some(next_arg) = cmd.args.get(cmd.args.iter().position(|a| a == arg).unwrap() + 1) {
                                    if let Word::Literal(opt_name) = next_arg {
                                        match opt_name.as_str() {
                                            "pipefail" => output.push_str("# set -o pipefail not implemented in Perl\n"),
                                            _ => output.push_str(&format!("# set -o {} not implemented\n", opt_name)),
                                        }
                                    }
                                }
                            }
                            _ => output.push_str(&format!("# set {} not implemented\n", opt)),
                        }
                    }
                }
            }
            "unset" => {
                // Handle unset command
                for arg in &cmd.args {
                    if let Word::Literal(var_name) = arg {
                        if let Some((array_name, key)) = self.extract_array_key(var_name) {
                            // Unset array element
                            output.push_str(&format!("delete ${}{{{}}};\n", array_name, key));
                        } else {
                            // Unset variable
                            output.push_str(&format!("undef ${};\n", var_name));
                            output.push_str(&format!("delete $ENV{{{}}};\n", var_name));
                        }
                    }
                }
            }
            "export" => {
                // Handle export command
                for arg in &cmd.args {
                    if let Word::Literal(var_name) = arg {
                        if let Some((array_name, key)) = self.extract_array_key(var_name) {
                            // Export array element
                            output.push_str(&format!("$ENV{{{}}} = ${}{{{}}};\n", var_name, array_name, key));
                        } else {
                            // Export variable
                            output.push_str(&format!("$ENV{{{}}} = ${};\n", var_name, var_name));
                        }
                    }
                }
            }
            "readonly" => {
                // Handle readonly command (not directly supported in Perl)
                for arg in &cmd.args {
                    if let Word::Literal(var_name) = arg {
                        output.push_str(&format!("# readonly {} not implemented in Perl\n", var_name));
                    }
                }
            }
            "declare" => {
                // Handle declare command
                for arg in &cmd.args {
                    if let Word::Literal(opt) = arg {
                        match opt.as_str() {
                            "-a" => {
                                // Declare array
                                if let Some(next_arg) = cmd.args.get(cmd.args.iter().position(|a| a == arg).unwrap() + 1) {
                                    if let Word::Literal(var_name) = next_arg {
                                        if !self.declared_locals.contains(var_name) {
                                            output.push_str(&format!("my @{} = ();\n", var_name));
                                            self.declared_locals.insert(var_name.clone());
                                        }
                                    }
                                }
                            }
                            "-A" => {
                                // Declare associative array
                                if let Some(next_arg) = cmd.args.get(cmd.args.iter().position(|a| a == arg).unwrap() + 1) {
                                    if let Word::Literal(var_name) = next_arg {
                                        if !self.declared_locals.contains(var_name) {
                                            output.push_str(&format!("my %{} = ();\n", var_name));
                                            self.declared_locals.insert(var_name.clone());
                                        }
                                    }
                                }
                            }
                            _ => output.push_str(&format!("# declare {} not implemented\n", opt)),
                        }
                    }
                }
            }
            _ => {
                // Other builtin commands
                output.push_str(&format!("# Builtin command '{}' not implemented\n", cmd.name));
            }
        }
        
        if has_env {
            output.push_str("}\n");
        }
        
        output
    }


}
