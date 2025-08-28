use crate::ast::*;
use crate::generator::Generator;

pub fn generate_cat_command(generator: &mut Generator, cmd: &SimpleCommand, redirects: &[Redirect]) -> String {
    let mut output = String::new();
    
    // Check if this cat command has heredoc redirects
    let mut has_heredoc = false;
    for redir in redirects {
        if matches!(redir.operator, RedirectOperator::Heredoc | RedirectOperator::HeredocTabs) {
            has_heredoc = true;
            if let Some(body) = &redir.heredoc_body {
                // Print the heredoc content directly
                output.push_str(&format!("print {};\n", generator.perl_string_literal(&Word::Literal(body.clone()))));
            }
        }
    }
    
    // If no heredocs, handle file reading as before
    if !has_heredoc {
        let filename = if cmd.args.is_empty() { 
            "".to_string()
        } else { 
            // Reconstruct the filename from split arguments if needed
            if cmd.args.len() > 1 {
                cmd.args.iter()
                    .map(|arg| generator.word_to_perl(arg))
                    .collect::<Vec<_>>()
                    .join("")
            } else {
                generator.word_to_perl(&cmd.args[0])
            }
        };
        
        output.push_str(&format!("$output = '';\n"));
        output.push_str(&format!("if (open(my $fh, '<', '{}')) {{\n", filename));
        output.push_str("while (my $line = <$fh>) {\n");
        output.push_str("$line =~ s/\\r\\n?/\\n/g; # Normalize line endings\n");
        output.push_str("$output .= $line;\n");
        output.push_str("}\n");
        output.push_str("close($fh);\n");
        output.push_str("} else {\n");
        output.push_str(&format!("warn \"cat: {}: No such file or directory\";\n", filename));
        output.push_str("exit(1);\n");
        output.push_str("}\n");
        output.push_str("\n");
    }
    
    output
}
