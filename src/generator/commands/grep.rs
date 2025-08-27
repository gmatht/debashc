use crate::ast::*;
use crate::generator::Generator;

pub fn generate_grep_command(generator: &mut Generator, cmd: &SimpleCommand, input_var: &str) -> String {
    let mut output = String::new();
    
    if let Some(pattern) = cmd.args.first() {
        let pattern_str = match pattern {
            Word::StringInterpolation(interp) => {
                // Extract the pattern from StringInterpolation
                interp.parts.iter()
                    .map(|part| match part {
                        crate::ast::StringPart::Literal(s) => s,
                        _ => ".*" // fallback for non-literal parts
                    })
                    .collect::<Vec<_>>()
                    .join("")
            },
            _ => generator.word_to_perl(pattern)
        };
        
        output.push_str(&format!("my @lines = split(/\\n/, {});\n", input_var));
        // Escape the pattern for Perl regex
        let escaped_pattern = pattern_str.replace("\\", "\\\\").replace("$", "\\$");
        output.push_str(&format!("my @filtered = grep /{}/, @lines;\n", escaped_pattern));
        output.push_str(&format!("{} = join(\"\\n\", @filtered);\n", input_var));
    }
    output.push_str("\n");
    
    output
}
