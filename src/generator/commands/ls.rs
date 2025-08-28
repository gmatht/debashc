use crate::ast::*;
use crate::generator::Generator;

fn generate_ls_helper(generator: &mut Generator, dir: &str, array_name: &str, sort_files: bool) -> String {
    let mut output = String::new();
    
    output.push_str(&generator.indent());
    output.push_str(&format!("my @{};\n", array_name));
    output.push_str(&generator.indent());
    output.push_str(&format!("if (opendir(my $dh, '{}')) {{\n", dir));
    generator.indent_level += 1;
    output.push_str(&generator.indent());
    output.push_str("while (my $file = readdir($dh)) {\n");
    generator.indent_level += 1;
    output.push_str(&generator.indent());
    output.push_str("next if $file eq '.' || $file eq '..';\n");
    output.push_str(&generator.indent());
    output.push_str(&format!("push @{}, $file;\n", array_name));
    generator.indent_level -= 1;
    output.push_str(&generator.indent());
    output.push_str("}\n");
    output.push_str(&generator.indent());
    output.push_str("closedir($dh);\n");
    if sort_files {
        output.push_str(&generator.indent());
        output.push_str(&format!("@{} = sort {{ $a cmp $b }} @{};\n", array_name, array_name));
    }
    generator.indent_level -= 1;
    output.push_str(&generator.indent());
    output.push_str("}\n");
    
    output
}

pub fn generate_ls_command(generator: &mut Generator, cmd: &SimpleCommand, pipeline_context: bool) -> String {
    let mut output = String::new();
    
    // Parse ls arguments to determine directory and flags
    let mut dir = ".";
    let mut single_column = false;
    
    for arg in &cmd.args {
        if let Word::Literal(s) = arg {
            if s.starts_with('-') {
                // Parse flags
                for flag in s.chars().skip(1) {
                    match flag {
                        '1' => single_column = true,
                        _ => {} // Ignore other flags for now
                    }
                }
            } else {
                // This is a directory argument
                dir = s;
            }
        }
    }
    
    output.push_str(&generate_ls_helper(generator, dir, "ls_files", !single_column));
    
    // Only print files if not in pipeline context
    if !pipeline_context {
        output.push_str(&generator.indent());
        if single_column {
            // -1 flag: one file per line, preserve directory order (no sorting)
            output.push_str("foreach my $file (@ls_files) {\n");
            generator.indent_level += 1;
            output.push_str(&generator.indent());
            output.push_str("print \"$file\\n\";\n");
            generator.indent_level -= 1;
            output.push_str(&generator.indent());
            output.push_str("}\n");
        } else {
            // Default: space-separated on one line, with sorting
            output.push_str("print join(\" \", @ls_files) . \"\\n\";\n");
        }
    }
    
    output
}

pub fn generate_ls_for_substitution(generator: &mut Generator, cmd: &SimpleCommand) -> String {
    // Parse ls arguments to determine directory
    let mut dir = ".";
    
    for arg in &cmd.args {
        if let Word::Literal(s) = arg {
            if !s.starts_with('-') {
                // This is a directory argument (not a flag)
                dir = s;
            }
        }
    }
    
    let mut output = String::new();
    output.push_str("do {\n");
    generator.indent_level += 1;
    output.push_str(&generate_ls_helper(generator, dir, "ls_files_sub", true));
    output.push_str(&generator.indent());
    output.push_str("join(\" \", @ls_files_sub);\n");
    generator.indent_level -= 1;
    output.push_str(&generator.indent());
    output.push_str("}");
    
    output
}
