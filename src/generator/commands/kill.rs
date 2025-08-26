use crate::ast::*;
use crate::generator::Generator;

pub fn generate_kill_command(generator: &mut Generator, cmd: &SimpleCommand) -> String {
    let mut output = String::new();
    
    // kill command syntax: kill [options] pid
    let mut signal = "TERM".to_string(); // Default signal
    let mut pids = Vec::new();
    
    // Parse kill options
    for arg in &cmd.args {
        if let Word::Literal(arg_str) = arg {
            if arg_str.starts_with("-") {
                if arg_str.len() > 1 {
                    // Handle numeric signals like -9, -15
                    if let Ok(sig_num) = arg_str[1..].parse::<i32>() {
                        signal = sig_num.to_string();
                    } else {
                        // Handle named signals like -TERM, -HUP
                        signal = arg_str[1..].to_string();
                    }
                }
            } else {
                // Non-option argument is a PID
                pids.push(generator.word_to_perl(arg));
            }
        } else {
            pids.push(generator.word_to_perl(arg));
        }
    }
    
    if pids.is_empty() {
        output.push_str("die \"kill: missing operand\\n\";\n");
    } else {
        output.push_str(&format!("my $signal = '{}';\n", signal));
        output.push_str("my @pids = (");
        for (i, pid) in pids.iter().enumerate() {
            if i > 0 { output.push_str(", "); }
            output.push_str(pid);
        }
        output.push_str(");\n");
        
        output.push_str("foreach my $pid (@pids) {\n");
        output.push_str("if ($pid =~ /^\\d+$/) {\n"); // Check if it's numeric
        output.push_str("my $result = kill $signal, $pid;\n");
        output.push_str("if ($result) {\n");
        output.push_str("print \"Sent signal $signal to process $pid\\n\";\n");
        output.push_str("} else {\n");
        output.push_str("print STDERR \"kill: ($pid) - No such process\\n\";\n");
        output.push_str("}\n");
        output.push_str("} else {\n");
        output.push_str("print STDERR \"kill: invalid process id: $pid\\n\";\n");
        output.push_str("}\n");
        output.push_str("}\n");
    }
    
    output
}
