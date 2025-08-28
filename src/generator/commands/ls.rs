use crate::ast::*;
use crate::generator::Generator;

fn generate_ls_helper(generator: &mut Generator, dir: &str, array_name: &str) -> String {
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
    output.push_str(&generator.indent());
    output.push_str(&format!("@{} = sort {{ $a cmp $b }} @{};\n", array_name, array_name));
    generator.indent_level -= 1;
    output.push_str(&generator.indent());
    output.push_str("}\n");
    
    output
}

pub fn generate_ls_command(generator: &mut Generator, cmd: &SimpleCommand, pipeline_context: bool) -> String {
    let mut output = String::new();
    
    let dir = if cmd.args.is_empty() { "." } else { &generator.word_to_perl(&cmd.args[0]) };
    
    output.push_str(&generate_ls_helper(generator, dir, "ls_files"));
    
    // Only print files if not in pipeline context
    if !pipeline_context {
        output.push_str(&generator.indent());
        output.push_str("foreach my $file (@ls_files) {\n");
        generator.indent_level += 1;
        output.push_str(&generator.indent());
        output.push_str("print \"$file\\n\";\n");
        generator.indent_level -= 1;
        output.push_str(&generator.indent());
        output.push_str("}\n");
    }
    
    output
}

pub fn generate_ls_for_substitution(generator: &mut Generator, cmd: &SimpleCommand) -> String {
    let dir = if cmd.args.is_empty() { "." } else { &generator.word_to_perl(&cmd.args[0]) };
    
    let mut output = String::new();
    output.push_str("do {\n");
    generator.indent_level += 1;
    output.push_str(&generate_ls_helper(generator, dir, "ls_files_sub"));
    output.push_str(&generator.indent());
    output.push_str("join(\"\\n\", @ls_files_sub);\n");
    generator.indent_level -= 1;
    output.push_str(&generator.indent());
    output.push_str("}");
    
    output
}
