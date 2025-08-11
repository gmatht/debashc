mod lexer;
mod parser;
mod ast;
mod perl_generator;

use lexer::*;
use parser::*;
use perl_generator::*;

use std::env;
use std::fs;
use std::io::{self, Write};

fn main() {
    let args: Vec<String> = env::args().collect();
    let program_name = &args[0];

    if args.len() < 2 {
        print_help(program_name);
        return;
    }

    let command = &args[1];
    match command.as_str() {
        "parse" => {
            if args.len() < 3 {
                eprintln!("Error: parse command requires input");
                return;
            }
            let input = &args[2];
            match parse_and_generate_perl(input) {
                Ok(code) => println!("{}", code),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        "file" => {
            if args.len() < 3 {
                eprintln!("Error: file command requires filename");
                return;
            }
            let filename = &args[2];
            match process_file_perl(filename) {
                Ok(code) => println!("{}", code),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        "lex" => {
            if args.len() < 3 {
                eprintln!("Error: lex command requires input");
                return;
            }
            let input = &args[2];
            let mut lexer = Lexer::new(input);
            while let Some(token) = lexer.next() {
                println!("{:?}", token);
            }
        }
        "help" | "--help" | "-h" => {
            print_help(program_name);
        }
        _ => {
            // Treat as direct input
            match parse_and_generate_perl(command) {
                Ok(code) => println!("{}", code),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
    }
}

fn parse_and_generate_perl(input: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut parser = Parser::new(input);
    let commands = parser.parse()?;
    
    let mut generator = PerlGenerator::new();
    let code = generator.generate(&commands)?;
    
    Ok(code)
}

fn process_file_perl(filename: &str) -> Result<String, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(filename)?;
    parse_and_generate_perl(&content)
}

fn print_help(program_name: &str) {
    println!("sh2perl - Shell to Perl translator");
    println!();
    println!("USAGE:");
    println!("  {} <command> [options]", program_name);
    println!();
    println!("COMMANDS:");
    println!("  parse <input>                  - Convert shell script to Perl");
    println!("  file <filename>                - Convert shell script file to Perl");
    println!("  lex <input>                    - Show lexer tokens");
    println!("  help                           - Show this help message");
    println!();
    println!("EXAMPLES:");
    println!("  {} parse 'echo hello world'", program_name);
    println!("  {} file examples/simple.sh", program_name);
    println!("  {} lex 'echo hello world'", program_name);
    println!("  {} 'echo hello world'           - Direct conversion", program_name);
    println!();
    println!("DESCRIPTION:");
    println!("  sh2perl is a specialized tool that translates shell scripts to Perl.");
    println!("  It can parse shell syntax and generate equivalent Perl code.");
}
