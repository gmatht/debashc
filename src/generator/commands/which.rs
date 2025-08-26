use crate::ast::*;
use crate::generator::Generator;

pub fn generate_which_command(generator: &mut Generator, cmd: &SimpleCommand) -> String {
    let mut output = String::new();
    
    // which command syntax: which command
    if let Some(command) = cmd.args.first() {
        let command_str = generator.word_to_perl(command);
        
        output.push_str(&format!("my $command = {};\n", command_str));
        output.push_str("my $found = 0;\n");
        output.push_str("foreach my $dir (split(/:/, $ENV{PATH})) {\n");
        output.push_str("my $full_path = \"$dir/$command\";\n");
        output.push_str("if (-x $full_path) {\n");
        output.push_str("print \"$full_path\\n\";\n");
        output.push_str("$found = 1;\n");
        output.push_str("last;\n");
        output.push_str("}\n");
        output.push_str("}\n");
        output.push_str("if (!$found) {\n");
        output.push_str("print STDERR \"$command: command not found\\n\";\n");
        output.push_str("exit(1);\n");
        output.push_str("}\n");
    } else {
        output.push_str("print STDERR \"which: missing command name\\n\";\n");
        output.push_str("exit(1);\n");
    }
    
    output
}
