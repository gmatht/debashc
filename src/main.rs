mod lexer;
mod parser;
mod ast;
mod shared_utils;
mod generator;
mod cache;
mod execution;
mod utils;
mod testing;
mod commands;
mod help;

use std::env;
use std::fs;

// Use the debug module for controlling DEBUG output
use debashl::debug::set_debug_enabled;
use debashl::{Parser, Generator};

// Import from our new modules
use crate::utils::generate_unified_diff;
use crate::testing::{test_all_examples, test_all_examples_next_fail, find_uses_of_system,
                    test_file_equivalence, AstFormatOptions};
use crate::commands::{run_generated, lex_input, parse_input, parse_file, parse_to_perl, 
                     parse_file_to_perl, interactive_mode};
use crate::help::show_help;

fn main() {
    let args: Vec<String> = env::args().collect();
    let program_name = &args[0];
    
    if args.len() < 2 {
        show_help(program_name);
        return;
    }
    
    let command = &args[1];
    
    if command == "--help" || command == "-h" {
        show_help(&args[0]);
        return;
    }
    
    // Check if this is --next-fail and disable DEBUG output early
    if command == "--next-fail" {
        set_debug_enabled(false);
    }
    
    // Parse AST formatting options and input/output options
    let mut ast_options = AstFormatOptions::default();
    let mut input_file: Option<String> = None;
    let mut output_file: Option<String> = None;
    let mut i = 2;
    
    // Special case: if the first argument is -i or -o, start parsing from index 1
    if command == "-i" || command == "-o" {
        i = 1;
    }
    
    while i < args.len() {
        match args[i].as_str() {
            "--ast-pretty" => {
                ast_options.compact = false;
                ast_options.indent = true;
                ast_options.newlines = true;
            }
            "--ast-compact" => {
                ast_options.compact = true;
                ast_options.indent = false;
                ast_options.newlines = false;
            }
            "--ast-indent" => {
                ast_options.indent = true;
            }
            "--ast-no-indent" => {
                ast_options.indent = false;
            }
            "--ast-newlines" => {
                ast_options.newlines = true;
            }
            "--ast-no-newlines" => {
                ast_options.newlines = false;
            }
            "-i" => {
                if i + 1 < args.len() {
                    input_file = Some(args[i + 1].to_string());
                    i += 1; // Skip the next argument since it's the filename
                } else {
                    println!("Error: -i requires a filename");
                    return;
                }
            }
            "-o" => {
                if i + 1 < args.len() {
                    output_file = Some(args[i + 1].to_string());
                    i += 1; // Skip the next argument since it's the filename
                } else {
                    println!("Error: -o requires a filename");
                    return;
                }
            }
            _ => {
                // This might be a filename or other argument
                break;
            }
        }
        i += 1;
    }
    
    let command = &args[1];
    
    // Special case: if the first argument is -i or -o, treat it as input/output processing
    if command == "-i" || command == "-o" {
        if let Some(input_filename) = &input_file {
            // Always treat as input file when -i is specified
            match fs::read_to_string(input_filename) {
                Ok(content) => {
                    println!("Processing input file: {}", input_filename);
                    // Parse the shell script
                    let commands = match Parser::new(&content).parse() {
                        Ok(c) => c,
                        Err(e) => { 
                            println!("Parse error: {}", e); 
                            return; 
                        }
                    };
                    
                    // Generate Perl code
                    let mut gen = Generator::new();
                    let code = gen.generate(&commands);
                    
                    // Handle output file option
                    if let Some(output_filename) = &output_file {
                        // Write to output file with UTF-8 encoding
                        match shared_utils::SharedUtils::write_utf8_file(output_filename, &code) {
                            Ok(_) => println!("Generated Perl code written to: {} (UTF-8 encoded)", output_filename),
                            Err(e) => println!("Error writing to output file {}: {}", output_filename, e),
                        }
                    } else {
                        // Show generated code and run it
                        println!("Generated Perl code:");
                        println!("{}", code);
                        println!("\n--- Running generated Perl code ---");
                        let tmp = "__tmp_run.pl";
                        if shared_utils::SharedUtils::write_utf8_file(tmp, &code).is_ok() {
                            let _ = std::process::Command::new("perl").arg(tmp).status();
                            let _ = fs::remove_file(tmp);
                        }
                    }
                }
                Err(e) => {
                    println!("Error reading input file {}: {}", input_filename, e);
                }
            }
        } else {
            println!("Error: -i option requires an input filename");
            return;
        }
        return;
    }
    
    match command.as_str() {
        "--test-eq" => {
            test_all_examples();
        }
        "--uses-of-system" => {
            find_uses_of_system();
        }
        "--next-fail" => {
            // Disable DEBUG output for --next-fail mode
            set_debug_enabled(false);
            
            // Parse optional test number, generator list, and AST options after --next-fail
            let mut test_number: Option<usize> = None;
            let mut generators = Vec::new();
            let mut i = 2;
            
            // Check if first argument is a number (test number)
            if i < args.len() {
                if let Ok(num) = args[i].parse::<usize>() {
                    test_number = Some(num);
                    i += 1;
                }
            }
            
            // Collect generators until we hit an AST option or run out of args
            while i < args.len() {
                match args[i].as_str() {
                    "--ast-pretty" | "--ast-compact" | "--ast-indent" | "--ast-no-indent" | 
                    "--ast-newlines" | "--ast-no-newlines" => {
                        // Stop parsing generators, let the AST options parsing continue
                        break;
                    }
                    generator => {
                        // Only perl generator is supported
                        if generator == "perl" {
                            generators.push(generator.to_string());
                        } else {
                            println!("Warning: Only 'perl' generator is supported, skipping '{}'", generator);
                        }
                    }
                }
                i += 1;
            }
            
            // If no generators specified, default to perl
            if generators.is_empty() {
                generators = vec!["perl".to_string()];
            }
            
            test_all_examples_next_fail(&generators, test_number);
        }
        "--clear-cache" => {
            // Clear the unified command cache
            let cache_file = "command_cache.json";
            if let Err(e) = fs::remove_file(cache_file) {
                if e.kind() != std::io::ErrorKind::NotFound {
                    println!("Error removing cache file: {}", e);
                } else {
                    println!("Cache file not found, nothing to clear.");
                }
            } else {
                println!("Command cache cleared successfully.");
            }
        }
        "--diff" => {
            if args.len() < 4 {
                println!("Error: --diff requires two filenames");
                println!("Usage: {} --diff <file1> <file2>", program_name);
                return;
            }
            let file1 = &args[2];
            let file2 = &args[3];
            
            // Read both files
            let content1 = match fs::read_to_string(file1) {
                Ok(c) => c,
                Err(e) => {
                    println!("Error reading {}: {}", file1, e);
                    return;
                }
            };
            
            let content2 = match fs::read_to_string(file2) {
                Ok(c) => c,
                Err(e) => {
                    println!("Error reading {}: {}", file2, e);
                    return;
                }
            };
            
            // Generate and display the diff
            println!("Diffing {} and {}:", file1, file2);
            println!("{}", generate_unified_diff(&content1, &content2, file1, file2));
        }
        "lex" => {
            if args.len() < 3 {
                println!("Error: lex command requires input");
                return;
            }
            let input = &args[2];
            // Check if input looks like a filename (contains .sh or doesn't contain spaces)
            if input.contains(".sh") || !input.contains(' ') {
                // Try to read as file first
                match fs::read_to_string(input) {
                    Ok(content) => {
                        lex_input(&content);
                    }
                    Err(_) => {
                        // If file read fails, treat as direct input
                        lex_input(input);
                    }
                }
            } else {
                lex_input(input);
            }
        }
        "parse" => {
            if args.len() < 3 {
                println!("Error: parse command requires input");
                return;
            }
            if args.len() >= 3 && args[2] == "--perl" {
                if args.len() < 4 {
                    println!("Error: parse --perl command requires input");
                    return;
                }
                let input = &args[3];
                parse_to_perl(input);
            } else if args.len() >= 3 && args[2] == "--run" {
                // parse --run <lang> <input>
                if args.len() < 5 {
                    println!("Error: parse --run <perl> <input>");
                    return;
                }
                let lang = &args[3];
                let input = &args[4];
                if lang == "perl" {
                    run_generated(lang, input);
                } else {
                    println!("Error: Only 'perl' language is supported");
                    return;
                }
            } else {
                let input = &args[2];
                // If looks like a filename or the path exists, treat as file
                if input.ends_with(".sh") || std::path::Path::new(input).exists() {
                    match fs::read_to_string(input) {
                        Ok(content) => parse_input(&content),
                        Err(_) => parse_input(input),
                    }
                } else {
                    parse_input(input);
                }
            }
        }
        "file" => {
            if args.len() < 3 {
                println!("Error: file command requires filename");
                return;
            }
            if args.len() >= 3 && args[2] == "--perl" {
                if args.len() < 4 {
                    println!("Error: file --perl command requires filename");
                    return;
                }
                let filename = &args[3];
                parse_file_to_perl(filename);
            } else if args.len() >= 3 && args[2] == "--test-file" {
                if args.len() < 5 {
                    println!("Error: file --test-file <perl> <filename>");
                    return;
                }
                let lang = &args[3];
                let filename = &args[4];
                if lang == "perl" {
                    let _ = test_file_equivalence(lang, filename);
                } else {
                    println!("Error: Only 'perl' language is supported");
                    return;
                }
            } else if args.len() >= 3 && args[2] == "--run" {
                if args.len() < 5 {
                    println!("Error: file --run <perl> <filename>");
                    return;
                }
                let lang = &args[3];
                let filename = &args[4];
                if lang == "perl" {
                    run_generated(lang, filename);
                } else {
                    println!("Error: Only 'perl' language is supported");
                    return;
                }
            } else {
                let filename = &args[2];
                parse_file(filename);
            }
        }
        "--test-file" | "test-file" => {
            if args.len() < 4 {
                println!("Error: --test-file <perl> <filename>");
                return;
            }
            let lang = &args[2];
            let filename = &args[3];
            if lang == "perl" {
                let _ = test_file_equivalence(lang, filename);
            } else {
                println!("Error: Only 'perl' language is supported");
                return;
            }
        }
        "interactive" => {
            interactive_mode();
        }
        "fail" => {
            // Shorthand for --next-fail
            // Disable DEBUG output for fail mode
            set_debug_enabled(false);
            
            // Parse optional test number, generator list, and AST options after fail
            let mut test_number: Option<usize> = None;
            let mut generators = Vec::new();
            let mut i = 2;
            
            // Check if first argument is a number (test number)
            if i < args.len() {
                if let Ok(num) = args[i].parse::<usize>() {
                    test_number = Some(num);
                    i += 1;
                }
            }
            
            // Collect generators until we hit an AST option or run out of args
            while i < args.len() {
                match args[i].as_str() {
                    "--ast-pretty" | "--ast-compact" | "--ast-indent" | "--ast-no-indent" | 
                    "--ast-newlines" | "--ast-no-newlines" => {
                        // Stop parsing generators, let the AST options parsing continue
                        break;
                    }
                    generator => {
                        // Only perl generator is supported
                        if generator == "perl" {
                            generators.push(generator.to_string());
                        } else {
                            println!("Warning: Only 'perl' generator is supported, skipping '{}'", generator);
                        }
                    }
                }
                i += 1;
            }
            
            // If no generators specified, default to perl
            if generators.is_empty() {
                generators = vec!["perl".to_string()];
            }
            
            test_all_examples_next_fail(&generators, test_number);
        }
        _ => {
            // Handle input file option
            if let Some(input_filename) = &input_file {
                // Always treat as input file when -i is specified
                match fs::read_to_string(input_filename) {
                    Ok(content) => {
                        println!("Processing input file: {}", input_filename);
                        // Parse the shell script
                        let commands = match Parser::new(&content).parse() {
                            Ok(c) => c,
                            Err(e) => { 
                                println!("Parse error: {}", e); 
                                return; 
                            }
                        };
                        
                        // Generate Perl code
                        let mut gen = Generator::new();
                        let code = gen.generate(&commands);
                        
                        // Handle output file option
                        if let Some(output_filename) = &output_file {
                            // Write to output file with UTF-8 encoding
                            match shared_utils::SharedUtils::write_utf8_file(output_filename, &code) {
                                Ok(_) => println!("Generated Perl code written to: {} (UTF-8 encoded)", output_filename),
                                Err(e) => println!("Error writing to output file {}: {}", output_filename, e),
                            }
                        } else {
                            // Show generated code and run it
                            println!("Generated Perl code:");
                            println!("{}", code);
                            println!("\n--- Running generated Perl code ---");
                            let tmp = "__tmp_run.pl";
                            if shared_utils::SharedUtils::write_utf8_file(tmp, &code).is_ok() {
                                let _ = std::process::Command::new("perl").arg(tmp).status();
                                let _ = fs::remove_file(tmp);
                            }
                        }
                    }
                    Err(e) => {
                        println!("Error reading input file {}: {}", input_filename, e);
                    }
                }
            } else if command.ends_with(".sh") {
                // Run the shell script directly
                match fs::read_to_string(command) {
                    Ok(content) => {
                        println!("Running shell script: {}", command);
                        // Parse and run the shell script
                        let commands = match Parser::new(&content).parse() {
                            Ok(c) => c,
                            Err(e) => { 
                                println!("Parse error: {}", e); 
                                return; 
                            }
                        };
                        
                        // Generate Perl code
                        let mut gen = Generator::new();
                        let code = gen.generate(&commands);
                        
                        // Handle output file option
                        if let Some(output_filename) = &output_file {
                            // Write to output file with UTF-8 encoding
                            match shared_utils::SharedUtils::write_utf8_file(output_filename, &code) {
                                Ok(_) => println!("Generated Perl code written to: {} (UTF-8 encoded)", output_filename),
                                Err(e) => println!("Error writing to output file {}: {}", output_filename, e),
                            }
                        } else {
                            // Show generated code and run it
                            println!("Generated Perl code:");
                            println!("{}", code);
                            println!("\n--- Running generated Perl code ---");
                            let tmp = "__tmp_run.pl";
                            if shared_utils::SharedUtils::write_utf8_file(tmp, &code).is_ok() {
                                let _ = std::process::Command::new("perl").arg(tmp).status();
                                let _ = fs::remove_file(tmp);
                            }
                        }
                    }
                    Err(e) => {
                        println!("Error reading file {}: {}", command, e);
                    }
                }
            } else {
                // Treat unknown commands as shell commands to be executed
                println!("Executing shell command: {}", command);
                println!("{}", "=".repeat(50));
                
                // Parse the command as shell input
                match Parser::new(command).parse() {
                    Ok(commands) => {
                        // Generate Perl code
                        let mut generator = Generator::new();
                        let perl_code = generator.generate(&commands);
                        
                        // Write to temporary file and execute
                        let tmp_file = "__tmp_direct_exec.pl";
                        if shared_utils::SharedUtils::write_utf8_file(tmp_file, &perl_code).is_ok() {
                            println!("Generated Perl code:");
                            println!("{}", perl_code);
                            println!("\n--- Running generated Perl code ---");
                            
                            match std::process::Command::new("perl").arg(tmp_file).output() {
                                Ok(output) => {
                                    if !output.stdout.is_empty() {
                                        print!("{}", String::from_utf8_lossy(&output.stdout));
                                    }
                                    if !output.stderr.is_empty() {
                                        eprint!("{}", String::from_utf8_lossy(&output.stderr));
                                    }
                                    println!("Exit code: {}", output.status);
                                }
                                Err(e) => {
                                    println!("Error running Perl: {}", e);
                                }
                            }
                            
                            // Clean up temporary file
                            let _ = fs::remove_file(tmp_file);
                        } else {
                            println!("Error writing temporary Perl file");
                        }
                    }
                    Err(e) => {
                        println!("Parse error: {}", e);
                        println!("Use '{} --help' for usage information", args[0]);
                    }
                }
                
                println!("{}", "=".repeat(50));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::{Lexer, Token};

    #[test]
    fn test_lexer_basic() {
        let input = "echo hello world";
        let mut lexer = Lexer::new(input);
        
        assert_eq!(lexer.next(), Some(&Token::Identifier));
        assert_eq!(lexer.next(), Some(&Token::Space));
        assert_eq!(lexer.next(), Some(&Token::Identifier));
        assert_eq!(lexer.next(), Some(&Token::Space));
        assert_eq!(lexer.next(), Some(&Token::Identifier));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn test_parser_basic() {
        let input = "echo hello world";
        let result = Parser::new(input).parse();
        assert!(result.is_ok());
    }
}
