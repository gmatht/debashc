use crate::ast::*;
use crate::generator::Generator;

pub fn generate_nice_command(generator: &mut Generator, cmd: &SimpleCommand) -> String {
    let mut output = String::new();
    
    // nice command syntax: nice [options] command [args...]
    let mut nice_value = 10; // Default nice value
    let mut command_args = Vec::new();
    let mut i = 0;
    
    // Parse nice options
    while i < cmd.args.len() {
        if let Word::Literal(arg_str) = &cmd.args[i] {
            if arg_str == "-n" && i + 1 < cmd.args.len() {
                if let Some(next_arg) = cmd.args.get(i + 1) {
                    if let Word::Literal(nice_str) = next_arg {
                        if let Ok(nice_num) = nice_str.parse::<i32>() {
                            nice_value = nice_num;
                        }
                    }
                    i += 1; // Skip the next argument
                }
            } else if arg_str.starts_with("-") && arg_str.len() > 1 {
                // Handle -10, -15 style nice values
                if let Ok(nice_num) = arg_str[1..].parse::<i32>() {
                    nice_value = nice_num;
                }
            } else if !arg_str.starts_with("-") {
                // Non-option argument, treat as command
                command_args = cmd.args[i..].to_vec();
                break;
            }
        } else {
            // Non-literal argument, treat as command
            command_args = cmd.args[i..].to_vec();
            break;
        }
        i += 1;
    }
    
    if command_args.is_empty() {
        output.push_str("die \"nice: missing command\\n\";\n");
    } else {
        let command = generator.word_to_perl(&command_args[0]);
        let args: Vec<String> = command_args[1..].iter()
            .map(|arg| generator.word_to_perl(arg))
            .collect();
        
        output.push_str("use POSIX qw(setpriority PRIO_PROCESS);\n");
        output.push_str(&format!("my $nice_value = {};\n", nice_value));
        output.push_str("my $pid = getpid();\n");
        
        // Set the nice value for current process
        output.push_str("my $old_priority = getpriority(PRIO_PROCESS, $pid);\n");
        output.push_str("setpriority(PRIO_PROCESS, $pid, $old_priority + $nice_value);\n");
        
        output.push_str("print \"Running command with nice value: $nice_value\\n\";\n");
        
        // Execute the command
        if args.is_empty() {
            output.push_str(&format!("system({});\n", command));
        } else {
            output.push_str(&format!("system({}, @args);\n", command));
        }
        
        // Restore original priority
        output.push_str("setpriority(PRIO_PROCESS, $pid, $old_priority);\n");
    }
    
    output
}
