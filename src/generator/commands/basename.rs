use crate::ast::*;
use crate::generator::Generator;

pub fn generate_basename_command(generator: &mut Generator, cmd: &SimpleCommand, input_var: &str) -> String {
    let mut output = String::new();
    
    // basename command syntax: basename path [suffix]
    if let Some(path) = cmd.args.first() {
        let path_str = generator.word_to_perl(path);
        let suffix = if cmd.args.len() > 1 {
            generator.word_to_perl(&cmd.args[1])
        } else {
            "".to_string()
        };
        
        output.push_str(&format!("my $path = {};\n", path_str));
        if !suffix.is_empty() {
            output.push_str(&format!("my $suffix = {};\n", suffix));
            output.push_str("$path =~ s/\\Q$suffix\\E$//;\n");
        }
        output.push_str("$path =~ s/.*\\///;\n"); // Remove directory part
        output.push_str(&format!("{} = $path;\n", input_var));
    } else {
        // Default to current directory
        output.push_str(&format!("{} = '.';\n", input_var));
    }
    
    output
}
