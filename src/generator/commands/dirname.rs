use crate::ast::*;
use crate::generator::Generator;

pub fn generate_dirname_command(generator: &mut Generator, cmd: &SimpleCommand, input_var: &str) -> String {
    let mut output = String::new();
    
    // dirname command syntax: dirname path
    if let Some(path) = cmd.args.first() {
        let path_str = generator.word_to_perl(path);
        
        output.push_str(&format!("my $path = {};\n", path_str));
        output.push_str("if ($path =~ /\\//) {\n");
        output.push_str("$path =~ s/\\/[^\\/]*$//;\n"); // Remove basename part
        output.push_str("$path = '.' if $path eq '';\n"); // Handle root case
        output.push_str("} else {\n");
        output.push_str("$path = '.';\n"); // No slashes, current directory
        output.push_str("}\n");
        output.push_str(&format!("{} = $path;\n", input_var));
    } else {
        // Default to current directory
        output.push_str(&format!("{} = '.';\n", input_var));
    }
    
    output
}
