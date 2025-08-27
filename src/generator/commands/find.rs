use crate::ast::*;
use crate::generator::Generator;

pub fn generate_find_command(generator: &mut Generator, cmd: &SimpleCommand) -> String {
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
    
    output.push_str(&format!("my @find_files;\n"));
    output.push_str(&format!("sub find_files {{\n"));
    output.push_str("my ($dir, $pattern) = @_;\n");
    output.push_str("if (opendir(my $dh, $dir)) {\n");
    output.push_str("while (my $file = readdir($dh)) {\n");
    output.push_str("next if $file eq '.' || $file eq '..';\n");
    output.push_str("my $full_path = $dir eq '.' ? \"./$file\" : \"$dir/$file\";\n");
    output.push_str("if (-d $full_path) {\n");
    output.push_str("find_files($full_path, $pattern);\n");
    output.push_str("} elsif ($file =~ /^$pattern$/) {\n");
    output.push_str("push @find_files, $full_path;\n");
    output.push_str("}\n");
    output.push_str("}\n");
    output.push_str("closedir($dh);\n");
    output.push_str("}\n");
    output.push_str("}\n");
    output.push_str(&format!("find_files('{}', '{}');\n", path, pattern));
    output.push_str("my $output = join(\"\\n\", @find_files);\n");
    output.push_str("\n");
    
    output
}
