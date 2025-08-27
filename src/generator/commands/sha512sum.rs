use crate::ast::*;
use crate::generator::Generator;

pub fn generate_sha512sum_command(generator: &mut Generator, cmd: &SimpleCommand, input_var: &str) -> String {
    let mut output = String::new();
    
    // sha512sum command syntax: sha512sum [options] [file]
    let mut check_mode = false;
    let mut files = Vec::new();
    
    // Parse sha512sum options
    for arg in &cmd.args {
        if let Word::Literal(arg_str) = arg {
            if arg_str == "-c" {
                check_mode = true;
            } else if !arg_str.starts_with('-') {
                files.push(generator.word_to_perl(arg));
            }
        } else {
            files.push(generator.word_to_perl(arg));
        }
    }
    
    if check_mode {
        // Check mode: verify checksums from input
        output.push_str(&format!("my @lines = split(/\\n/, {});\n", input_var));
        output.push_str("my @results;\n");
        output.push_str("foreach my $line (@lines) {\n");
        output.push_str("chomp($line);\n");
        output.push_str("if ($line =~ /^([a-f0-9]{128})\\s+(.+)$/) {\n");
        output.push_str("my ($expected_hash, $filename) = ($1, $2);\n");
        output.push_str("if (-f $filename) {\n");
        output.push_str("my $actual_hash = `sha512sum \"$filename\" | cut -d' ' -f1`;\n");
        output.push_str("chomp($actual_hash);\n");
        output.push_str("if ($expected_hash eq $actual_hash) {\n");
        output.push_str("push @results, \"$filename: OK\";\n");
        output.push_str("} else {\n");
        output.push_str("push @results, \"$filename: FAILED\";\n");
        output.push_str("}\n");
        output.push_str("} else {\n");
        output.push_str("push @results, \"$filename: No such file\";\n");
        output.push_str("}\n");
        output.push_str("}\n");
        output.push_str("}\n");
        output.push_str(&format!("{} = join(\"\\n\", @results);\n", input_var));
    } else if files.is_empty() {
        // No files specified, calculate hash of input
        output.push_str(&format!("my $hash = `echo -n \"${}\" | sha512sum | cut -d' ' -f1`;\n", input_var));
        output.push_str("chomp($hash);\n");
        output.push_str(&format!("{} = $hash;\n", input_var));
    } else {
        // Calculate hashes of specified files
        output.push_str("my @results;\n");
        for file in &files {
            output.push_str(&format!("if (-f {}) {{\n", file));
            output.push_str(&format!("my $hash = `sha512sum {} | cut -d' ' -f1`;\n", file));
            output.push_str("chomp($hash);\n");
            output.push_str(&format!("push @results, \"$hash  {}\";\n", file));
            output.push_str("} else {\n");
            output.push_str(&format!("push @results, \"00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000  {}  FAILED open or read\";\n", file));
            output.push_str("}\n");
        }
        output.push_str(&format!("{} = join(\"\\n\", @results);\n", input_var));
    }
    output.push_str("\n");
    
    output
}
