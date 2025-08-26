use crate::ast::*;
use crate::generator::Generator;

pub fn generate_time_command(generator: &mut Generator, cmd: &SimpleCommand) -> String {
    let mut output = String::new();
    
    // time command syntax: time command
    // This is a simplified implementation that measures execution time
    output.push_str("use Time::HiRes qw(gettimeofday tv_interval);\n");
    output.push_str("my $start_time = [gettimeofday];\n");
    
    // Execute the command (if any arguments provided)
    if let Some(command) = cmd.args.first() {
        let command_str = generator.word_to_perl(command);
        output.push_str(&format!("system({});\n", command_str));
    }
    
    output.push_str("my $end_time = [gettimeofday];\n");
    output.push_str("my $elapsed = tv_interval($start_time, $end_time);\n");
    output.push_str("printf \"real\\t%.3fs\\n\", $elapsed;\n");
    
    output
}
