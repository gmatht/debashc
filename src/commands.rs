use std::fs;
use std::process::Command;
use std::io::Write;
use crate::shared_utils;
use crate::utils::{extract_line_col, caret_snippet};
use debashl::{Lexer, Parser, Generator};

pub fn run_generated(lang: &str, input: &str) {
    let source = if input.ends_with(".sh") || std::path::Path::new(input).exists() {
        fs::read_to_string(input).unwrap_or_else(|_| input.to_string())
    } else { input.to_string() };
    let commands = match Parser::new(&source).parse() {
        Ok(c) => c,
        Err(e) => { println!("Parse error: {}", e); return; }
    };
    match lang {
        "perl" => {
            let mut gen = Generator::new();
            let code = gen.generate(&commands);
            let tmp = "__tmp_run.pl";
            if shared_utils::SharedUtils::write_utf8_file(tmp, &code).is_ok() {
                let _ = std::process::Command::new("perl").arg(tmp).status();
                let _ = fs::remove_file(tmp);
            }
        }
        _ => println!("Unsupported language for --run: {}", lang),
    }
}

pub fn lex_input(input: &str) {
    println!("Tokenizing: {}", input);
    println!("Tokens:");
    println!("{}", "=".repeat(50));
    
    let mut lexer = Lexer::new(input);
    let mut token_count = 0;
    
    while let Some(token) = lexer.peek() {
        token_count += 1;
        let token_text = lexer.get_current_text().unwrap_or_else(|| "".to_string());
        println!("{:3}: {:?}('{}')", token_count, token, token_text);
        lexer.next(); // Advance to next token
    }
    
    println!("{}", "=".repeat(50));
    println!("Total tokens: {}", token_count);
}

pub fn parse_input(input: &str) {
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
            if let Some((line, col)) = extract_line_col(&e) {
                println!("Parse error at {}:{}: {}", line, col, e);
                if let Some(snippet) = caret_snippet(input, line, col) {
                    println!("{}", snippet);
                }
            } else {
                println!("Parse error: {}", e);
            }
        }
    }
    
    println!("{}", "=".repeat(50));
}

pub fn parse_file(filename: &str) {
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

pub fn parse_to_perl(input: &str) {
    // Check if input looks like a filename and read it if so
    let content = if input.ends_with(".sh") || std::path::Path::new(input).exists() {
        match fs::read_to_string(input) {
            Ok(content) => content,
            Err(_) => input.to_string(),
        }
    } else {
        input.to_string()
    };
    
    println!("Converting to Perl: {}", content);
    println!("Perl Code:");
    println!("{}", "=".repeat(50));
    
    match Parser::new(&content).parse() {
        Ok(commands) => {
            let mut generator = Generator::new();
            let perl_code = generator.generate(&commands);
            println!("{}", perl_code);
        }
        Err(e) => {
            println!("Parse error: {}", e);
        }
    }
    
    println!("{}", "=".repeat(50));
}

pub fn parse_file_to_perl(filename: &str) {
    println!("Converting file to Perl: {}", filename);
    
    match fs::read_to_string(filename) {
        Ok(content) => {
            println!("Converting to Perl: {}", content);
            println!("Perl Code:");
            println!("{}", "=".repeat(50));
            
            match Parser::new(&content).parse() {
                Ok(commands) => {
                    let mut generator = Generator::new();
                    let perl_code = generator.generate(&commands);
                    println!("{}", perl_code);
                }
                Err(e) => {
                    println!("Parse error: {}", e);
                }
            }
            
            println!("{}", "=".repeat(50));
        }
        Err(e) => {
            println!("Error reading file: {}", e);
        }
    }
}

pub fn interactive_mode() {
    println!("Interactive Shell Script Parser");
    println!("Type 'quit' to exit, 'help' for commands");
    println!("{}", "=".repeat(50));
    
    loop {
        print!("sh2perl> ");
        std::io::stdout().flush().unwrap();
        
        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_err() {
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
                println!();
                println!("Type 'quit' to exit interactive mode");
                println!("Use --help from command line for full program help");
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
