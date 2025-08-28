use crate::ast::*;
use crate::generator::Generator;

fn escape_glob_pattern(pattern: &str) -> String {
    pattern.chars().map(|c| match c {
        '*' => ".*".to_string(),
        '?' => ".".to_string(),
        '.' => "\\.".to_string(),
        '[' => "\\[".to_string(),
        ']' => "\\]".to_string(),
        '(' => "\\(".to_string(),
        ')' => "\\)".to_string(),
        '+' => "\\+".to_string(),
        '^' => "\\^".to_string(),
        '$' => "\\$".to_string(),
        '|' => "\\|".to_string(),
        '{' => "\\{".to_string(),
        '}' => "\\}".to_string(),
        _ => c.to_string()
    }).collect()
}

pub fn generate_find_command(generator: &mut Generator, cmd: &SimpleCommand, generate_output: bool) -> String {
    let mut output = String::new();
    
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
    output.push_str("my $full_path = $dir eq '.' ? \"./$file\" : \"$dir/$file\";\n");
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
    output.push_str(&format!("find_files('{}', '{}');\n", path, escape_glob_pattern(&pattern)));
    if generate_output {
        output.push_str(&generator.indent());
        output.push_str("$output = join(\"\\n\", @find_files);\n");
    }
    output.push_str("\n");
    
    output
}
