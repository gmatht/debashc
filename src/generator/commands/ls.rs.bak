use crate::ast::*;
use crate::generator::Generator;

pub fn generate_ls_command(generator: &mut Generator, cmd: &SimpleCommand) -> String {
    let mut output = String::new();
    
    let dir = if cmd.args.is_empty() { "." } else { &generator.word_to_perl(&cmd.args[0]) };
    
    output.push_str(&format!("my @ls_files;\n"));
    output.push_str(&format!("if (opendir(my $dh, '{}')) {{\n", dir));
    output.push_str("while (my $file = readdir($dh)) {\n");
    output.push_str("next if $file eq '.' || $file eq '..';\n");
    output.push_str("push @ls_files, $file;\n");
    output.push_str("}\n");
    output.push_str("closedir($dh);\n");
    output.push_str("}\n");
    output.push_str("my $output = join(\"\\n\", @ls_files);\n");
    
    output
}
