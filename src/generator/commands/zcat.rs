use crate::ast::*;
use crate::generator::Generator;

pub fn generate_zcat_command(generator: &mut Generator, cmd: &SimpleCommand) -> String {
    let mut output = String::new();
    
    // zcat command syntax: zcat file
    if let Some(filename) = cmd.args.first() {
        let filename_str = generator.word_to_perl(filename);
        
        output.push_str(&format!("my $filename = {};\n", filename_str));
        output.push_str("if (!-f $filename) {\n");
        output.push_str(&format!("die \"zcat: {}: No such file or directory\\n\";\n", filename_str));
        output.push_str("}\n");
        output.push_str("if (open(my $fh, '-|', \"gunzip -c $filename\")) {\n");
        output.push_str("while (my $line = <$fh>) {\n");
        output.push_str("print $line;\n");
        output.push_str("}\n");
        output.push_str("close($fh);\n");
        output.push_str("} else {\n");
        output.push_str(&format!("die \"zcat: {}: Cannot open file\\n\";\n", filename_str));
        output.push_str("}\n");
    } else {
        // Read from stdin if no file specified
        output.push_str("while (my $line = <STDIN>) {\n");
        output.push_str("print $line;\n");
        output.push_str("}\n");
    }
    
    output
}
