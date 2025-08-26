use crate::ast::*;
use crate::generator::Generator;

pub fn generate_yes_command(generator: &mut Generator, cmd: &SimpleCommand) -> String {
    let mut output = String::new();
    
    // yes command syntax: yes [string]
    let string_to_repeat = if let Some(arg) = cmd.args.first() {
        generator.word_to_perl(arg)
    } else {
        "y".to_string()
    };
    
    output.push_str(&format!("my $string = {};\n", string_to_repeat));
    output.push_str("while (1) {\n");
    output.push_str("print \"$string\\n\";\n");
    output.push_str("}\n");
    
    output
}
