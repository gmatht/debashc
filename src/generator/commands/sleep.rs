use crate::ast::*;
use crate::generator::Generator;

pub fn generate_sleep_command(generator: &mut Generator, cmd: &SimpleCommand) -> String {
    let mut output = String::new();
    
    // sleep command syntax: sleep [number][suffix]
    // Supports seconds (default), minutes (m), hours (h), days (d)
    if let Some(duration) = cmd.args.first() {
        let duration_str = generator.word_to_perl(duration);
        
        output.push_str("use Time::HiRes qw(sleep);\n");
        output.push_str(&format!("sleep({});\n", duration_str));
    } else {
        // Default to 1 second if no argument provided
        output.push_str("use Time::HiRes qw(sleep);\n");
        output.push_str("sleep(1);\n");
    }
    
    output
}
