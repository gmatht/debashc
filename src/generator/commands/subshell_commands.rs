use crate::ast::*;
use crate::generator::Generator;

pub fn generate_subshell_impl(generator: &mut Generator, command: &Command) -> String {
    let mut output = String::new();
    
    // Generate subshell command
    output.push_str("(");
    output.push_str(&generator.generate_command(command));
    output.push_str(")");
    
    output
}

pub fn generate_background_impl(generator: &mut Generator, command: &Command) -> String {
    let mut output = String::new();
    
    // Generate background command
    output.push_str("(");
    output.push_str(&generator.generate_command(command));
    output.push_str(") &");
    
    output
}
