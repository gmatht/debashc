use crate::ast::*;
use crate::generator::Generator;

fn generate_ls_helper(generator: &mut Generator, dir: &str, array_name: &str, sort_files: bool) -> String {
    let mut output = String::new();
    
    // Always declare the array first
    output.push_str(&generator.indent());
    output.push_str(&format!("my @{};\n", array_name));
    
    if sort_files {
        // For sorting, we still need to collect files first
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
        output.push_str(&generator.indent());
        output.push_str(&format!("@{} = sort {{ $a cmp $b }} @{};\n", array_name, array_name));
        generator.indent_level -= 1;
        output.push_str(&generator.indent());
        output.push_str("}\n");
    } else {
        // For non-sorting, collect to array instead of printing directly
        // This is needed for pipeline context where we need the array
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
        generator.indent_level -= 1;
        output.push_str(&generator.indent());
        output.push_str("}\n");
    }
    
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
    
    // Only print files if not in pipeline context
    if !pipeline_context {
        if single_column {
            // -1 flag: one file per line, preserve directory order (no sorting)
            output.push_str(&generate_ls_helper(generator, dir, "ls_files", false));
        } else {
            // Default: space-separated on one line, with sorting
            output.push_str(&generate_ls_helper(generator, dir, "ls_files", true));
            output.push_str(&generator.indent());
            output.push_str("print join(\" \", @ls_files) . \"\\n\";\n");
        }
    } else {
        // In pipeline context, always collect to array for output
        // For -1 flag, we want newline-separated output, so use single_column=true
        // For other flags, we want space-separated output, so use single_column=false
        output.push_str(&generate_ls_helper(generator, dir, "ls_files", single_column));
    }
    
    output
}

pub fn generate_ls_for_substitution(generator: &mut Generator, cmd: &SimpleCommand) -> String {
    // Parse ls arguments to determine directory and flags
    let mut dir = ".";
    let mut single_column = false; // Default to multi-column (space-separated) like shell ls
    
    for arg in &cmd.args {
        if let Word::Literal(s) = arg {
            if s.starts_with('-') {
                // Parse flags
                for flag in s.chars().skip(1) {
                    match flag {
                        '1' => single_column = true,  // -1 flag: explicit single column (newline-separated)
                        'C' => single_column = false, // -C flag: multi-column (space-separated)
                        'x' => single_column = false, // -x flag: multi-column (space-separated)
                        _ => {} // Ignore other flags for now
                    }
                }
            } else {
                // This is a directory argument (not a flag)
                dir = s;
            }
        }
    }
    
    let mut output = String::new();
    output.push_str("do {\n");
    generator.indent_level += 1;
    output.push_str(&generate_ls_helper(generator, dir, "ls_files_sub", !single_column));
    output.push_str(&generator.indent());
    if single_column {
        // -1 flag: join with newlines to preserve one file per line
        output.push_str("join(\"\\n\", @ls_files_sub);\n");
    } else {
        // Default behavior and -C or -x flags: join with spaces for multi-column output
        output.push_str("join(\" \", @ls_files_sub);\n");
    }
    generator.indent_level -= 1;
    output.push_str(&generator.indent());
    output.push_str("}");
    
    output
}
