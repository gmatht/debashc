use crate::ast::*;
use crate::generator::Generator;

pub fn generate_date_command(generator: &mut Generator, cmd: &SimpleCommand) -> String {
    let mut output = String::new();
    
    // date command syntax: date [format]
    if let Some(format) = cmd.args.first() {
        let format_str = generator.word_to_perl(format);
        
        output.push_str("use POSIX qw(strftime);\n");
        output.push_str(&format!("my $format = {};\n", format_str));
        output.push_str("my $date = strftime($format, localtime());\n");
        output.push_str("print $date;\n");
    } else {
        // Default format: RFC 2822 format
        output.push_str("use POSIX qw(strftime);\n");
        output.push_str("my $date = strftime('%a, %d %b %Y %H:%M:%S %z', localtime());\n");
        output.push_str("print $date;\n");
    }
    
    output
}
