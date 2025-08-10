mod lexer;
mod parser;
mod ast;
mod perl_generator;
mod rust_generator;
mod python_generator;
mod lua_generator;
mod c_generator;
mod js_generator;
mod english_generator;
mod french_generator;
mod batch_generator;
mod powershell_generator;

use lexer::*;
use parser::*;
// use ast::*; // not needed at top-level
use perl_generator::*;
use rust_generator::*;
use python_generator::*;
use lua_generator::*;
use c_generator::*;
use js_generator::*;
use english_generator::*;
use french_generator::*;
use batch_generator::*;
use powershell_generator::*;

use std::env;
use std::fs;
use std::io::{self, Write};
use std::process::{Command, Stdio};
use std::time::Duration;
use std::thread;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("Usage: {} <command> [input]", args[0]);
        println!("Commands:");
        println!("  lex <input>     - Tokenize shell script");
        println!("  parse <input>   - Parse shell script to AST");
        println!("  parse --perl <input> - Convert shell script to Perl");
        println!("  parse --rust <input> - Convert shell script to Rust");
        println!("  parse --python <input> - Convert shell script to Python");
        println!("  parse --lua <input> - Convert shell script to Lua");
        println!("  parse --c <input> - Convert shell script to C");
        println!("  parse --js <input> - Convert shell script to JavaScript (Node.js)");
        println!("  parse --english <input> - Generate English pseudocode");
        println!("  parse --french <input> - Générer du pseudo-code en français");
        println!("  parse --comment <input> - Output original SH with English pseudocode comments");
        println!("  parse --bat <input> - Convert shell script to Windows Batch");
        println!("  parse --ps <input> - Convert shell script to PowerShell");
        println!("  parse --run <perl|python|rust|lua|js|ps> <input> - Generate and run code");
        println!("  file <filename> - Parse shell script from file");
        println!("  file --perl <filename> - Convert shell script file to Perl");
        println!("  file --rust <filename> - Convert shell script file to Rust");
        println!("  file --python <filename> - Convert shell script file to Python");
        println!("  file --lua <filename> - Convert shell script file to Lua");
        println!("  file --c <filename> - Convert shell script file to C");
        println!("  file --js <filename> - Convert shell script file to JavaScript (Node.js)");
        println!("  file --english <filename> - Generate English pseudocode from file");
        println!("  file --french <filename> - Générer du pseudo-code en français (fichier)");
        println!("  file --comment <filename> - Output original SH with English pseudocode comments");
        println!("  file --bat <filename> - Convert shell script file to Windows Batch");
        println!("  file --ps <filename> - Convert shell script file to PowerShell");
        println!("  file --test-file <perl|python|rust|lua|js|ps> <filename> - Compare outputs of .sh vs translated");
        println!("  file --run <perl|python|rust|lua|js|ps> <filename> - Generate and run from file");
        println!("  --test-file <perl|python|rust|lua|js|ps> <filename> - Same as file --test-file (top-level)");
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
            } else if args.len() >= 3 && args[2] == "--lua" {
                if args.len() < 4 {
                    println!("Error: parse --lua command requires input");
                    return;
                }
                let input = &args[3];
                parse_to_lua(input);
            } else if args.len() >= 3 && args[2] == "--c" {
                if args.len() < 4 { println!("Error: parse --c command requires input"); return; }
                let input = &args[3];
                parse_to_c(input);
            } else if args.len() >= 3 && args[2] == "--js" {
                if args.len() < 4 { println!("Error: parse --js command requires input"); return; }
                let input = &args[3];
                parse_to_js(input);
            } else if args.len() >= 3 && args[2] == "--english" {
                if args.len() < 4 { println!("Error: parse --english command requires input"); return; }
                let input = &args[3];
                parse_to_english(input);
            } else if args.len() >= 3 && args[2] == "--french" {
                if args.len() < 4 { println!("Error: parse --french command requires input"); return; }
                let input = &args[3];
                parse_to_french(input);
            } else if args.len() >= 3 && args[2] == "--comment" {
                if args.len() < 4 { println!("Error: parse --comment command requires input"); return; }
                let input = &args[3];
                parse_to_commented_shell(input);
            } else if args.len() >= 3 && args[2] == "--bat" {
                if args.len() < 4 { println!("Error: parse --bat command requires input"); return; }
                let input = &args[3];
                parse_to_batch(input);
            } else if args.len() >= 3 && args[2] == "--ps" {
                if args.len() < 4 { println!("Error: parse --ps command requires input"); return; }
                let input = &args[3];
                parse_to_powershell(input);
            } else if args.len() >= 3 && args[2] == "--run" {
                // parse --run <lang> <input>
                if args.len() < 5 {
                    println!("Error: parse --run <perl|python|rust|lua|js|ps> <input>");
                    return;
                }
                let lang = &args[3];
                let input = &args[4];
                run_generated(lang, input);
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
            } else if args.len() >= 3 && args[2] == "--lua" {
                if args.len() < 4 {
                    println!("Error: file --lua command requires filename");
                    return;
                }
                let filename = &args[3];
                parse_file_to_lua(filename);
            } else if args.len() >= 3 && args[2] == "--c" {
                if args.len() < 4 { println!("Error: file --c command requires filename"); return; }
                let filename = &args[3];
                parse_file_to_c(filename);
            } else if args.len() >= 3 && args[2] == "--js" {
                if args.len() < 4 { println!("Error: file --js command requires filename"); return; }
                let filename = &args[3];
                parse_file_to_js(filename);
            } else if args.len() >= 3 && args[2] == "--english" {
                if args.len() < 4 { println!("Error: file --english command requires filename"); return; }
                let filename = &args[3];
                parse_file_to_english(filename);
            } else if args.len() >= 3 && args[2] == "--french" {
                if args.len() < 4 { println!("Error: file --french command requires filename"); return; }
                let filename = &args[3];
                parse_file_to_french(filename);
            } else if args.len() >= 3 && args[2] == "--comment" {
                if args.len() < 4 { println!("Error: file --comment command requires filename"); return; }
                let filename = &args[3];
                parse_file_to_commented_shell(filename);
            } else if args.len() >= 3 && args[2] == "--bat" {
                if args.len() < 4 { println!("Error: file --bat command requires filename"); return; }
                let filename = &args[3];
                parse_file_to_batch(filename);
            } else if args.len() >= 3 && args[2] == "--ps" {
                if args.len() < 4 { println!("Error: file --ps command requires filename"); return; }
                let filename = &args[3];
                parse_file_to_powershell(filename);
            } else if args.len() >= 3 && args[2] == "--test-file" {
                if args.len() < 5 {
                    println!("Error: file --test-file <perl|python|rust|lua|js|ps> <filename>");
                    return;
                }
                let lang = &args[3];
                let filename = &args[4];
                test_file_equivalence(lang, filename);
            } else if args.len() >= 3 && args[2] == "--run" {
                if args.len() < 5 {
                    println!("Error: file --run <perl|python|rust|lua|js|ps> <filename>");
                    return;
                }
                let lang = &args[3];
                let filename = &args[4];
                run_generated(lang, filename);
            } else {
                let filename = &args[2];
                parse_file(filename);
            }
        }
        "--test-file" | "test-file" => {
            if args.len() < 4 {
                println!("Error: --test-file <perl|python|rust|lua|js|ps> <filename>");
                return;
            }
            let lang = &args[2];
            let filename = &args[3];
            test_file_equivalence(lang, filename);
        }
        "interactive" => {
            interactive_mode();
        }
        _ => {
            println!("Unknown command: {}", command);
        }
    }
}
fn run_generated(lang: &str, input: &str) {
    let source = if input.ends_with(".sh") || std::path::Path::new(input).exists() {
        fs::read_to_string(input).unwrap_or_else(|_| input.to_string())
    } else { input.to_string() };
    let commands = match Parser::new(&source).parse() {
        Ok(c) => c,
        Err(e) => { println!("Parse error: {}", e); return; }
    };
    match lang {
        "perl" => {
            let mut gen = PerlGenerator::new();
            let code = gen.generate(&commands);
            let tmp = "__tmp_run.pl";
            if fs::write(tmp, &code).is_ok() {
                let _ = std::process::Command::new("perl").arg(tmp).status();
                let _ = fs::remove_file(tmp);
            }
        }
        "python" => {
            let mut gen = PythonGenerator::new();
            let code = gen.generate(&commands);
            let tmp = "__tmp_run.py";
            if fs::write(tmp, &code).is_ok() {
                let _ = std::process::Command::new("python3").arg(tmp).status();
                let _ = fs::remove_file(tmp);
            }
        }
        "rust" => {
            let mut gen = RustGenerator::new();
            let code = gen.generate(&commands);
            let tmp = "__tmp_run.rs";
            if fs::write(tmp, &code).is_ok() {
                let out = "__tmp_run_bin";
                let compile = std::process::Command::new("rustc")
                    .arg("--edition=2021").arg(tmp).arg("-o").arg(out)
                    .status();
                if compile.map(|s| s.success()).unwrap_or(false) {
                    let abs = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
                        .join(out);
                    match std::process::Command::new(&abs).output() {
                        Ok(child_out) => {
                            if !child_out.stdout.is_empty() { let _ = std::io::stdout().write_all(&child_out.stdout); }
                            if !child_out.stderr.is_empty() { let _ = std::io::stderr().write_all(&child_out.stderr); }
                        }
                        Err(e) => { eprintln!("Failed to run compiled Rust binary: {}", e); }
                    }
                    let _ = fs::remove_file(out);
                    if cfg!(windows) {
                        let _ = fs::remove_file(format!("{}.exe", out));
                        let _ = fs::remove_file(format!("{}.pdb", out));
                    }
                }
                let _ = fs::remove_file(tmp);
            }
        }
        "lua" => {
            let mut gen = LuaGenerator::new();
            let code = gen.generate(&commands);
            let tmp = "__tmp_run.lua";
            if fs::write(tmp, &code).is_ok() {
                let _ = std::process::Command::new("lua").arg(tmp).status();
                let _ = fs::remove_file(tmp);
            }
        }
        "js" => {
            let mut gen = JsGenerator::new();
            let code = gen.generate(&commands);
            let tmp = "__tmp_run.js";
            if fs::write(tmp, &code).is_ok() {
                let _ = std::process::Command::new("node").arg(tmp).status();
                let _ = fs::remove_file(tmp);
            }
        }
        "ps" => {
            let mut gen = PowerShellGenerator::new();
            let code = gen.generate(&commands);
            let tmp = "__tmp_run.ps1";
            if fs::write(tmp, &code).is_ok() {
                let shell = if cfg!(windows) { "powershell" } else { "pwsh" };
                let _ = std::process::Command::new(shell).arg("-File").arg(tmp).status();
                let _ = fs::remove_file(tmp);
            }
        }
        _ => println!("Unsupported language for --run: {}", lang),
    }
}

fn test_file_equivalence(lang: &str, filename: &str) {
    // Read shell script content
    let shell_content = match fs::read_to_string(filename) {
        Ok(c) => c,
        Err(e) => { eprintln!("Failed to read {}: {}", filename, e); return; }
    };

    // Parse and generate target language code
    let commands = match Parser::new(&shell_content).parse() {
        Ok(c) => c,
        Err(e) => { eprintln!("Failed to parse {}: {:?}", filename, e); return; }
    };

    let (tmp_file, run_cmd) = match lang {
        "perl" => {
            let mut gen = PerlGenerator::new();
            let code = gen.generate(&commands);
            let tmp = "__tmp_test_output.pl";
            if let Err(e) = fs::write(tmp, &code) { eprintln!("Failed to write Perl temp file: {}", e); return; }
            (tmp.to_string(), vec![if cfg!(windows) { "perl" } else { "perl" }, tmp])
        }
        "python" => {
            let mut gen = PythonGenerator::new();
            let code = gen.generate(&commands);
            let tmp = "__tmp_test_output.py";
            if let Err(e) = fs::write(tmp, &code) { eprintln!("Failed to write Python temp file: {}", e); return; }
            (tmp.to_string(), vec!["python3", tmp])
        }
        "lua" => {
            let mut gen = LuaGenerator::new();
            let code = gen.generate(&commands);
            let tmp = "__tmp_test_output.lua";
            if let Err(e) = fs::write(tmp, &code) { eprintln!("Failed to write Lua temp file: {}", e); return; }
            (tmp.to_string(), vec!["lua", tmp])
        }
        "js" => {
            let mut gen = JsGenerator::new();
            let code = gen.generate(&commands);
            let tmp = "__tmp_test_output.js";
            if let Err(e) = fs::write(tmp, &code) { eprintln!("Failed to write JS temp file: {}", e); return; }
            (tmp.to_string(), vec!["node", tmp])
        }
        "ps" => {
            let mut gen = PowerShellGenerator::new();
            let code = gen.generate(&commands);
            let tmp = "__tmp_test_output.ps1";
            if let Err(e) = fs::write(tmp, &code) { eprintln!("Failed to write PowerShell temp file: {}", e); return; }
            let shell = if cfg!(windows) { "powershell" } else { "pwsh" };
            (tmp.to_string(), vec![shell, "-File", tmp])
        }
        "rust" => {
            let mut gen = RustGenerator::new();
            let code = gen.generate(&commands);
            let tmp_src = "__tmp_test_output.rs";
            if let Err(e) = fs::write(tmp_src, &code) { eprintln!("Failed to write Rust temp file: {}", e); return; }
            // compile
            let out = "__tmp_test_bin";
            let compile_status = Command::new("rustc")
                .arg("--edition=2021").arg(tmp_src).arg("-o").arg(out)
                .status();
            match compile_status {
                Ok(s) if s.success() => {}
                Ok(_) => { eprintln!("Rust compilation failed"); let _ = fs::remove_file(tmp_src); return; }
                Err(e) => { eprintln!("Failed to run rustc: {}", e); let _ = fs::remove_file(tmp_src); return; }
            }
            // We'll run compiled binary; remember to cleanup later
            (tmp_src.to_string(), vec![if cfg!(windows) { "__tmp_test_bin.exe" } else { "__tmp_test_bin" }])
        }
        _ => { eprintln!("Unsupported language for --test-file: {}", lang); return; }
    };

    // Run shell script
    let shell_output = {
        let mut child = match Command::new("sh").arg(filename).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn() {
            Ok(c) => c,
            Err(e) => { eprintln!("Failed to spawn sh: {}", e); cleanup_tmp(lang, &tmp_file); return; }
        };
        let start = std::time::Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(_)) => break child.wait_with_output().unwrap(),
                Ok(None) => {
                    if start.elapsed() > Duration::from_millis(1000) { let _ = child.kill(); break child.wait_with_output().unwrap(); }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(_) => break child.wait_with_output().unwrap(),
            }
        }
    };

    // Run translated program
    let translated_output = {
        if lang == "rust" {
            // Run compiled binary directly (first arg of run_cmd)
            let bin = run_cmd[0];
            let mut child = match Command::new(bin).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn() {
                Ok(c) => c,
                Err(e) => { eprintln!("Failed to run compiled Rust: {}", e); cleanup_tmp(lang, &tmp_file); return; }
            };
            let start = std::time::Instant::now();
            let out = loop {
                match child.try_wait() {
                    Ok(Some(_)) => break child.wait_with_output().unwrap(),
                    Ok(None) => {
                        if start.elapsed() > Duration::from_millis(1000) { let _ = child.kill(); break child.wait_with_output().unwrap(); }
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(_) => break child.wait_with_output().unwrap(),
                }
            };
            out
        } else {
            let mut cmd = Command::new(run_cmd[0]);
            for a in &run_cmd[1..] { cmd.arg(a); }
            let mut child = match cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn() {
                Ok(c) => c,
                Err(e) => { eprintln!("Failed to run translated program: {}", e); cleanup_tmp(lang, &tmp_file); return; }
            };
            let start = std::time::Instant::now();
            let out = loop {
                match child.try_wait() {
                    Ok(Some(_)) => break child.wait_with_output().unwrap(),
                    Ok(None) => {
                        if start.elapsed() > Duration::from_millis(1000) { let _ = child.kill(); break child.wait_with_output().unwrap(); }
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(_) => break child.wait_with_output().unwrap(),
                }
            };
            out
        }
    };

    // Cleanup temp files
    cleanup_tmp(lang, &tmp_file);

    // Normalize and compare
    let shell_stdout = String::from_utf8_lossy(&shell_output.stdout).to_string().replace("\r\n", "\n").trim().to_string();
    let shell_stderr = String::from_utf8_lossy(&shell_output.stderr).to_string().replace("\r\n", "\n").trim().to_string();
    let trans_stdout = String::from_utf8_lossy(&translated_output.stdout).to_string().replace("\r\n", "\n").trim().to_string();
    let trans_stderr = String::from_utf8_lossy(&translated_output.stderr).to_string().replace("\r\n", "\n").trim().to_string();
    let shell_success = shell_output.status.success();
    let trans_success = translated_output.status.success();

    println!("Shell exit: {} | Translated exit: {}", shell_output.status, translated_output.status);
    println!("Shell stdout: {:?}", shell_stdout);
    println!("Translated stdout: {:?}", trans_stdout);
    println!("Shell stderr: {:?}", shell_stderr);
    println!("Translated stderr: {:?}", trans_stderr);

    if shell_success != trans_success || shell_stdout != trans_stdout || shell_stderr != trans_stderr {
        eprintln!("Mismatch detected (lang: {}, file: {})", lang, filename);
        std::process::exit(1);
    } else {
        println!("Outputs match (lang: {}, file: {})", lang, filename);
    }
}

fn cleanup_tmp(lang: &str, tmp_file: &str) {
    let _ = fs::remove_file(tmp_file);
    if lang == "rust" {
        let _ = fs::remove_file("__tmp_test_bin");
        if cfg!(windows) {
            let _ = fs::remove_file("__tmp_test_bin.exe");
            let _ = fs::remove_file("__tmp_test_bin.pdb");
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
            if let Some((line, col)) = extract_line_col(&e) {
                println!("Parse error at {}:{}: {:?}", line, col, e);
                if let Some(snippet) = caret_snippet(input, line, col) {
                    println!("{}", snippet);
                }
            } else {
                println!("Parse error: {:?}", e);
            }
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

fn parse_to_lua(input: &str) {
    println!("Converting to Lua: {}", input);
    println!("Lua Code:");
    println!("{}", "=".repeat(50));
    
    let mut parser = Parser::new(input);
    match parser.parse() {
        Ok(commands) => {
            let mut generator = LuaGenerator::new();
            let lua_code = generator.generate(&commands);
            println!("{}", lua_code);
        }
        Err(e) => {
            println!("Parse error: {:?}", e);
        }
    }
}

fn parse_file_to_lua(filename: &str) {
    match fs::read_to_string(filename) {
        Ok(content) => {
            println!("Converting file to Lua: {}", filename);
            println!("Lua Code:");
            println!("{}", "=".repeat(50));
            
            let mut parser = Parser::new(&content);
            match parser.parse() {
                Ok(commands) => {
                    let mut generator = LuaGenerator::new();
                    let lua_code = generator.generate(&commands);
                    println!("{}", lua_code);
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

fn parse_to_c(input: &str) {
    println!("Converting to C: {}", input);
    println!("C Code:");
    println!("{}", "=".repeat(50));
    match Parser::new(input).parse() {
        Ok(commands) => {
            let mut generator = CGenerator::new();
            let code = generator.generate(&commands);
            println!("{}", code);
        }
        Err(e) => {
            if let Some((line, col)) = extract_line_col(&e) {
                println!("Parse error at {}:{}: {:?}", line, col, e);
            } else {
                println!("Parse error: {:?}", e);
            }
        },
    }
}

fn parse_file_to_c(filename: &str) {
    match fs::read_to_string(filename) {
        Ok(content) => parse_to_c(&content),
        Err(e) => println!("Error reading file: {}", e),
    }
}

fn parse_to_js(input: &str) {
    println!("Converting to JavaScript: {}", input);
    println!("JavaScript Code:");
    println!("{}", "=".repeat(50));
    match Parser::new(input).parse() {
        Ok(commands) => {
            let mut generator = JsGenerator::new();
            let code = generator.generate(&commands);
            println!("{}", code);
        }
        Err(e) => println!("Parse error: {:?}", e),
    }
}

fn parse_file_to_js(filename: &str) {
    match fs::read_to_string(filename) {
        Ok(content) => parse_to_js(&content),
        Err(e) => println!("Error reading file: {}", e),
    }
}

fn parse_to_english(input: &str) {
    println!("English Pseudocode for: {}", input);
    println!("{}", "=".repeat(50));
    match Parser::new(input).parse() {
        Ok(commands) => {
            let mut generator = EnglishGenerator::new();
            let code = generator.generate(&commands);
            println!("{}", code);
        }
        Err(e) => println!("Parse error: {:?}", e),
    }
}

fn parse_file_to_english(filename: &str) {
    match fs::read_to_string(filename) {
        Ok(content) => parse_to_english(&content),
        Err(e) => println!("Error reading file: {}", e),
    }
}

fn parse_to_french(input: &str) {
    println!("Pseudo-code français pour: {}", input);
    println!("{}", "=".repeat(50));
    match Parser::new(input).parse() {
        Ok(commands) => {
            let mut generator = FrenchGenerator::new();
            let code = generator.generate(&commands);
            println!("{}", code);
        }
        Err(e) => println!("Parse error: {:?}", e),
    }
}

fn parse_file_to_french(filename: &str) {
    match fs::read_to_string(filename) {
        Ok(content) => parse_to_french(&content),
        Err(e) => println!("Error reading file: {}", e),
    }
}

fn parse_to_commented_shell(input: &str) {
    println!("Original shell with English pseudocode comments:");
    println!("{}", "=".repeat(50));
    let mut parser = Parser::new(input);
    match parser.parse() {
        Ok(commands) => {
            let mut generator = EnglishGenerator::new();
            let commentary = generator.generate(&commands);
            println!("# {}", commentary.replace('\n', "\n# "));
            println!("{}", input);
        }
        Err(e) => println!("Parse error: {:?}", e),
    }
}

fn parse_file_to_commented_shell(filename: &str) {
    match fs::read_to_string(filename) {
        Ok(content) => parse_to_commented_shell(&content),
        Err(e) => println!("Error reading file: {}", e),
    }
}

fn parse_to_batch(input: &str) {
    println!("Windows Batch for: {}", input);
    println!("{}", "=".repeat(50));
    match Parser::new(input).parse() {
        Ok(commands) => {
            let mut generator = BatchGenerator::new();
            let code = generator.generate(&commands);
            println!("{}", code);
        }
        Err(e) => println!("Parse error: {:?}", e),
    }
}

fn parse_file_to_batch(filename: &str) {
    match fs::read_to_string(filename) {
        Ok(content) => parse_to_batch(&content),
        Err(e) => println!("Error reading file: {}", e),
    }
}

fn parse_to_powershell(input: &str) {
    println!("PowerShell for: {}", input);
    println!("{}", "=".repeat(50));
    match Parser::new(input).parse() {
        Ok(commands) => {
            let mut generator = PowerShellGenerator::new();
            let code = generator.generate(&commands);
            println!("{}", code);
        }
        Err(e) => println!("Parse error: {:?}", e),
    }
}

fn extract_line_col(e: &dyn std::error::Error) -> Option<(usize, usize)> {
    let msg = e.to_string();
    // Try to find pattern " at line:col" we emit in our errors
    let parts: Vec<&str> = msg.split_whitespace().collect();
    for window in parts.windows(2) {
        if window[0] == "at" {
            if let Some((l, c)) = parse_line_col(window[1]) { return Some((l, c)); }
        }
    }
    None
}

fn parse_line_col(s: &str) -> Option<(usize, usize)> {
    let mut it = s.split(':');
    let line = it.next()?.trim_end_matches(',');
    let col = it.next()?.trim_end_matches(',');
    Some((line.parse().ok()?, col.parse().ok()?))
}

fn caret_snippet(input: &str, line: usize, col: usize) -> Option<String> {
    let lines: Vec<&str> = input.lines().collect();
    if line == 0 || line > lines.len() { return None; }
    let src_line = lines[line - 1];
    let mut caret = String::new();
    let prefix = format!("{:>4} | ", line);
    caret.push_str(&prefix);
    caret.push_str(src_line);
    caret.push('\n');
    caret.push_str(&" ".repeat(prefix.len().saturating_sub(0) + col.saturating_sub(1)));
    caret.push('^');
    Some(caret)
}

fn parse_file_to_powershell(filename: &str) {
    match fs::read_to_string(filename) {
        Ok(content) => parse_to_powershell(&content),
        Err(e) => println!("Error reading file: {}", e),
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
    use crate::lexer::Token;

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