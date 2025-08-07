use debashc::{Lexer, Parser, PerlGenerator, RustGenerator, PythonGenerator};
use std::env;
use std::fs;
use std::io::{self, Write};

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("Usage: {} <command> [input]", args[0]);
        println!("Commands:");
        println!("  lex <input>     - Tokenize shell script");
        println!("  parse <input>   - Parse shell script to AST");
        println!("  parse --perl <input> - Convert shell script to Perl");
        println!("  parse --rust <input> - Convert shell script to Rust");
        println!("  file <filename> - Parse shell script from file");
        println!("  file --perl <filename> - Convert shell script file to Perl");
        println!("  file --rust <filename> - Convert shell script file to Rust");
        return;
    }
    
    let command = &args[1];
    
    match command.as_str() {
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
            } else if args.len() >= 3 && args[2] == "--rust" {
                if args.len() < 4 {
                    println!("Error: parse --rust command requires input");
                    return;
                }
                let input = &args[3];
                parse_to_rust(input);
            } else if args.len() >= 3 && args[2] == "--python" {
                if args.len() < 4 {
                    println!("Error: parse --python command requires input");
                    return;
                }
                let input = &args[3];
                parse_to_python(input);
            } else {
                let input = &args[2];
                parse_input(input);
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
            } else if args.len() >= 3 && args[2] == "--rust" {
                if args.len() < 4 {
                    println!("Error: file --rust command requires filename");
                    return;
                }
                let filename = &args[3];
                parse_file_to_rust(filename);
            } else if args.len() >= 3 && args[2] == "--python" {
                if args.len() < 4 {
                    println!("Error: file --python command requires filename");
                    return;
                }
                let filename = &args[3];
                parse_file_to_python(filename);
            } else {
                let filename = &args[2];
                parse_file(filename);
            }
        }
        "interactive" => {
            interactive_mode();
        }
        _ => {
            println!("Unknown command: {}", command);
        }
    }
}

fn lex_input(input: &str) {
    println!("Tokenizing: {}", input);
    println!("Tokens:");
    println!("{}", "=".repeat(50));
    
    let mut lexer = Lexer::new(input);
    let mut token_count = 0;
    
    while let Some(token) = lexer.next() {
        token_count += 1;
        println!("{:3}: {:?}", token_count, token);
    }
    
    println!("{}", "=".repeat(50));
    println!("Total tokens: {}", token_count);
}

fn parse_input(input: &str) {
    println!("Parsing: {}", input);
    println!("AST:");
    println!("{}", "=".repeat(50));
    
    match Parser::new(input).parse() {
        Ok(commands) => {
            for (i, command) in commands.iter().enumerate() {
                println!("Command {}: {:?}", i + 1, command);
            }
        }
        Err(e) => {
            println!("Parse error: {}", e);
        }
    }
    
    println!("{}", "=".repeat(50));
}

fn parse_file(filename: &str) {
    println!("Parsing file: {}", filename);
    
    match fs::read_to_string(filename) {
        Ok(content) => {
            parse_input(&content);
        }
        Err(e) => {
            println!("Error reading file: {}", e);
        }
    }
}

fn parse_to_perl(input: &str) {
    println!("Converting to Perl: {}", input);
    println!("Perl Code:");
    println!("{}", "=".repeat(50));
    
    match Parser::new(input).parse() {
        Ok(commands) => {
            let mut generator = PerlGenerator::new();
            let perl_code = generator.generate(&commands);
            println!("{}", perl_code);
        }
        Err(e) => {
            println!("Parse error: {}", e);
        }
    }
    
    println!("{}", "=".repeat(50));
}

fn parse_file_to_perl(filename: &str) {
    println!("Converting file to Perl: {}", filename);
    
    match fs::read_to_string(filename) {
        Ok(content) => {
            parse_to_perl(&content);
        }
        Err(e) => {
            println!("Error reading file: {}", e);
        }
    }
}

fn parse_to_rust(input: &str) {
    println!("Converting to Rust: {}", input);
    println!("Rust Code:");
    println!("{}", "=".repeat(50));
    
    match Parser::new(input).parse() {
        Ok(commands) => {
            let mut generator = RustGenerator::new();
            let rust_code = generator.generate(&commands);
            println!("{}", rust_code);
        }
        Err(e) => {
            println!("Parse error: {}", e);
        }
    }
    
    println!("{}", "=".repeat(50));
}

fn parse_file_to_rust(filename: &str) {
    println!("Converting file to Rust: {}", filename);
    
    match fs::read_to_string(filename) {
        Ok(content) => {
            parse_to_rust(&content);
        }
        Err(e) => {
            println!("Error reading file: {}", e);
        }
    }
}

fn parse_to_python(input: &str) {
    println!("Converting to Python: {}", input);
    println!("Python Code:");
    println!("{}", "=".repeat(50));
    
    let mut parser = Parser::new(input);
    match parser.parse() {
        Ok(commands) => {
            let mut generator = PythonGenerator::new();
            let python_code = generator.generate(&commands);
            println!("{}", python_code);
        }
        Err(e) => {
            println!("Parse error: {:?}", e);
        }
    }
}

fn parse_file_to_python(filename: &str) {
    match fs::read_to_string(filename) {
        Ok(content) => {
            println!("Converting file to Python: {}", filename);
            println!("Python Code:");
            println!("{}", "=".repeat(50));
            
            let mut parser = Parser::new(&content);
            match parser.parse() {
                Ok(commands) => {
                    let mut generator = PythonGenerator::new();
                    let python_code = generator.generate(&commands);
                    println!("{}", python_code);
                }
                Err(e) => {
                    println!("Parse error: {:?}", e);
                }
            }
        }
        Err(e) => {
            println!("Error reading file: {}", e);
        }
    }
}

fn interactive_mode() {
    println!("Interactive Shell Script Parser");
    println!("Type 'quit' to exit, 'help' for commands");
    println!("{}", "=".repeat(50));
    
    loop {
        print!("sh2perl> ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }
        
        let input = input.trim();
        
        match input {
            "quit" | "exit" => break,
            "help" => {
                println!("Commands:");
                println!("  lex <input>     - Tokenize shell script");
                println!("  parse <input>   - Parse shell script to AST");
                println!("  quit/exit       - Exit interactive mode");
                println!("  help            - Show this help");
            }
            _ => {
                if input.starts_with("lex ") {
                    let script = &input[4..];
                    lex_input(script);
                } else if input.starts_with("parse ") {
                    let script = &input[6..];
                    parse_input(script);
                } else if !input.is_empty() {
                    // Default to parsing
                    parse_input(input);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sh2perl::Token;

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