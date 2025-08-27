use crate::ast::*;
use crate::generator::Generator;

pub fn generate_subshell_impl(generator: &mut Generator, command: &Command) -> String {
    let mut output = String::new();
    
    // Generate subshell command using Perl's do block
    output.push_str("do {\n");
    generator.indent_level += 1;
    output.push_str(&generator.indent());
    output.push_str(&generator.generate_command(command));
    generator.indent_level -= 1;
    output.push_str("};\n");
    
    output
}

pub fn generate_background_impl(generator: &mut Generator, command: &Command) -> String {
    let mut output = String::new();
    
    // Generate background command using Perl's do block
    output.push_str("do {\n");
    generator.indent_level += 1;
    output.push_str(&generator.indent());
    output.push_str(&generator.generate_command(command));
    generator.indent_level -= 1;
    output.push_str("};\n");
    
    output
}
