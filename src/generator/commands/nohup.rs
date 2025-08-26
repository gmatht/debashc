use crate::ast::*;
use crate::generator::Generator;

pub fn generate_nohup_command(generator: &mut Generator, cmd: &SimpleCommand) -> String {
    let mut output = String::new();
    
    // nohup command syntax: nohup command [args...]
    if cmd.args.is_empty() {
        output.push_str("die \"nohup: missing command\\n\";\n");
    } else {
        // Get the command to run
        let command = generator.word_to_perl(&cmd.args[0]);
        let args: Vec<String> = cmd.args[1..].iter()
            .map(|arg| generator.word_to_perl(arg))
            .collect();
        
        output.push_str("use POSIX qw(setsid);\n");
        output.push_str("use POSIX qw(dup2);\n");
        output.push_str("use POSIX qw(open);\n");
        
        // Redirect output to nohup.out if not already redirected
        output.push_str("my $nohup_out = 'nohup.out';\n");
        output.push_str("if (!defined $ENV{NOHUP_OUT}) {\n");
        output.push_str("$ENV{NOHUP_OUT} = $nohup_out;\n");
        output.push_str("}\n");
        
        // Create new session and detach from terminal
        output.push_str("my $pid = fork();\n");
        output.push_str("if ($pid == 0) {\n"); // Child process
        output.push_str("setsid();\n"); // Create new session
        
        // Redirect output
        output.push_str("if (open(my $fh, '>', $ENV{NOHUP_OUT})) {\n");
        output.push_str("dup2(fileno($fh), STDOUT->fileno());\n");
        output.push_str("dup2(fileno($fh), STDERR->fileno());\n");
        output.push_str("close($fh);\n");
        output.push_str("}\n");
        
        // Execute the command
        if args.is_empty() {
            output.push_str(&format!("exec({});\n", command));
        } else {
            output.push_str(&format!("exec({}, @args);\n", command));
        }
        output.push_str("exit(1);\n"); // Should not reach here
        output.push_str("} elsif ($pid > 0) {\n"); // Parent process
        output.push_str("print \"nohup: ignoring input and appending output to '$ENV{NOHUP_OUT}'\\n\";\n");
        output.push_str("print \"nohup: process $pid started\\n\";\n");
        output.push_str("} else {\n");
        output.push_str("die \"nohup: fork failed\\n\";\n");
        output.push_str("}\n");
    }
    
    output
}
