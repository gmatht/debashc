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

#[derive(Debug)]
struct TestResult {
    success: bool,
    shell_stdout: String,
    shell_stderr: String,
    translated_stdout: String,
    translated_stderr: String,
    shell_exit: i32,
    translated_exit: i32,
}

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
        println!("  --test-eq - Test all generators against all examples");
        println!("  --next-fail - Test all generators against all examples, exit after first failure with diff");
        return;
    }
    
    let command = &args[1];
    
    match command.as_str() {
        "--test-eq" => {
            test_all_examples();
        }
        "--next-fail" => {
            test_all_examples_next_fail();
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
                let _ = test_file_equivalence(lang, filename);
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
            let _ = test_file_equivalence(lang, filename);
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

fn test_file_equivalence(lang: &str, filename: &str) -> Result<(), String> {
    // Read shell script content
    let shell_content = match fs::read_to_string(filename) {
        Ok(c) => c,
        Err(e) => { return Err(format!("Failed to read {}: {}", filename, e)); }
    };

    // Parse and generate target language code
    let commands = match Parser::new(&shell_content).parse() {
        Ok(c) => c,
        Err(e) => { return Err(format!("Failed to parse {}: {:?}", filename, e)); }
    };

    let (tmp_file, run_cmd) = match lang {
        "perl" => {
            let mut gen = PerlGenerator::new();
            let code = gen.generate(&commands);
            let tmp = "__tmp_test_output.pl";
            if let Err(e) = fs::write(tmp, &code) { return Err(format!("Failed to write Perl temp file: {}", e)); }
            (tmp.to_string(), vec![if cfg!(windows) { "perl" } else { "perl" }, tmp])
        }
        "python" => {
            let mut gen = PythonGenerator::new();
            let code = gen.generate(&commands);
            let tmp = "__tmp_test_output.py";
            if let Err(e) = fs::write(tmp, &code) { return Err(format!("Failed to write Python temp file: {}", e)); }
            (tmp.to_string(), vec!["python3", tmp])
        }
        "lua" => {
            let mut gen = LuaGenerator::new();
            let code = gen.generate(&commands);
            let tmp = "__tmp_test_output.lua";
            if let Err(e) = fs::write(tmp, &code) { return Err(format!("Failed to write Lua temp file: {}", e)); }
            (tmp.to_string(), vec!["lua", tmp])
        }
        "js" => {
            let mut gen = JsGenerator::new();
            let code = gen.generate(&commands);
            let tmp = "__tmp_test_output.js";
            if let Err(e) = fs::write(tmp, &code) { return Err(format!("Failed to write JS temp file: {}", e)); }
            (tmp.to_string(), vec!["node", tmp])
        }
        "ps" => {
            let mut gen = PowerShellGenerator::new();
            let code = gen.generate(&commands);
            let tmp = "__tmp_test_output.ps1";
            if let Err(e) = fs::write(tmp, &code) { return Err(format!("Failed to write PowerShell temp file: {}", e)); }
            let shell = if cfg!(windows) { "powershell" } else { "pwsh" };
            // Add -ExecutionPolicy Bypass to allow running unsigned scripts
            (tmp.to_string(), vec![shell, "-ExecutionPolicy", "Bypass", "-File", tmp])
        }
        "rust" => {
            let mut gen = RustGenerator::new();
            let code = gen.generate(&commands);
            let tmp_src = "__tmp_test_output.rs";
            if let Err(e) = fs::write(tmp_src, &code) { return Err(format!("Failed to write Rust temp file: {}", e)); }
            // compile
            let out = if cfg!(windows) { "__tmp_test_bin.exe" } else { "__tmp_test_bin" };
            let out_path = std::env::current_dir().unwrap_or_default().join(out);
            let compile_status = Command::new("rustc")
                .arg("--edition=2021").arg(tmp_src).arg("-o").arg(&out_path)
                .status();
            match compile_status {
                Ok(s) if s.success() => {}
                Ok(_) => { return Err("Rust compilation failed".to_string()); }
                Err(e) => { return Err(format!("Failed to run rustc: {}", e)); }
            }
            // We'll run compiled binary; remember to cleanup later
            (tmp_src.to_string(), vec![out])
        }
        _ => { return Err(format!("Unsupported language for --test-file: {}", lang)); }
    };

    // Run shell script
    let shell_output = {
        let mut child = match Command::new("sh").arg(filename).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn() {
            Ok(c) => c,
            Err(e) => { cleanup_tmp(lang, &tmp_file); return Err(format!("Failed to spawn sh: {}", e)); }
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
            let bin = if cfg!(windows) { "__tmp_test_bin.exe" } else { "__tmp_test_bin" };
            let abs_bin = std::env::current_dir().unwrap_or_default().join(bin);
            let mut child = match Command::new(&abs_bin).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn() {
                Ok(c) => c,
                Err(e) => { cleanup_tmp(lang, &tmp_file); return Err(format!("Failed to run compiled Rust: {} ({})", e, abs_bin.display())); }
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
                Err(e) => { cleanup_tmp(lang, &tmp_file); return Err(format!("Failed to run translated program: {}", e)); }
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

    // Limit output to first 200 characters for readability
    let truncate_output = |s: &str| -> String {
        if s.len() > 200 {
            format!("{}...", &s[..200])
        } else {
            s.to_string()
        }
    };

    let truncated_shell_stdout = truncate_output(&shell_stdout);
    let truncated_trans_stdout = truncate_output(&trans_stdout);
    let truncated_shell_stderr = truncate_output(&shell_stderr);
    let truncated_trans_stderr = truncate_output(&trans_stderr);

    println!("Shell exit: {} | Translated exit: {}", shell_output.status, translated_output.status);
    println!("Shell stdout: {:?}", truncated_shell_stdout);
    println!("Translated stdout: {:?}", truncated_trans_stdout);
    println!("Shell stderr: {:?}", truncated_shell_stderr);
    println!("Translated stderr: {:?}", truncated_trans_stderr);

    if shell_success != trans_success || shell_stdout != trans_stdout || shell_stderr != trans_stderr {
        return Err(format!("Mismatch detected (lang: {}, file: {})", lang, filename));
    } else {
        println!("Outputs match (lang: {}, file: {})", lang, filename);
    }
    
    Ok(())
}

fn test_file_equivalence_detailed(lang: &str, filename: &str) -> Result<TestResult, String> {
    // Read shell script content
    let shell_content = match fs::read_to_string(filename) {
        Ok(c) => c,
        Err(e) => { return Err(format!("Failed to read {}: {}", filename, e)); }
    };

    // Parse and generate target language code
    let commands = match Parser::new(&shell_content).parse() {
        Ok(c) => c,
        Err(e) => { return Err(format!("Failed to parse {}: {:?}", filename, e)); }
    };

    let (tmp_file, run_cmd) = match lang {
        "perl" => {
            let mut gen = PerlGenerator::new();
            let code = gen.generate(&commands);
            let tmp = "__tmp_test_output.pl";
            if let Err(e) = fs::write(tmp, &code) { return Err(format!("Failed to write Perl temp file: {}", e)); }
            (tmp.to_string(), vec![if cfg!(windows) { "perl" } else { "perl" }, tmp])
        }
        "python" => {
            let mut gen = PythonGenerator::new();
            let code = gen.generate(&commands);
            let tmp = "__tmp_test_output.py";
            if let Err(e) = fs::write(tmp, &code) { return Err(format!("Failed to write Python temp file: {}", e)); }
            (tmp.to_string(), vec!["python3", tmp])
        }
        "lua" => {
            let mut gen = LuaGenerator::new();
            let code = gen.generate(&commands);
            let tmp = "__tmp_test_output.lua";
            if let Err(e) = fs::write(tmp, &code) { return Err(format!("Failed to write Lua temp file: {}", e)); }
            (tmp.to_string(), vec!["lua", tmp])
        }
        "js" => {
            let mut gen = JsGenerator::new();
            let code = gen.generate(&commands);
            let tmp = "__tmp_test_output.js";
            if let Err(e) = fs::write(tmp, &code) { return Err(format!("Failed to write JS temp file: {}", e)); }
            (tmp.to_string(), vec!["node", tmp])
        }
        "ps" => {
            let mut gen = PowerShellGenerator::new();
            let code = gen.generate(&commands);
            let tmp = "__tmp_test_output.ps1";
            if let Err(e) = fs::write(tmp, &code) { return Err(format!("Failed to write PowerShell temp file: {}", e)); }
            let shell = if cfg!(windows) { "powershell" } else { "pwsh" };
            // Add -ExecutionPolicy Bypass to allow running unsigned scripts
            (tmp.to_string(), vec![shell, "-ExecutionPolicy", "Bypass", "-File", tmp])
        }
        "rust" => {
            let mut gen = RustGenerator::new();
            let code = gen.generate(&commands);
            let tmp_src = "__tmp_test_output.rs";
            if let Err(e) = fs::write(tmp_src, &code) { return Err(format!("Failed to write Rust temp file: {}", e)); }
            // compile
            let out = if cfg!(windows) { "__tmp_test_bin.exe" } else { "__tmp_test_bin" };
            let out_path = std::env::current_dir().unwrap_or_default().join(out);
            let compile_status = Command::new("rustc")
                .arg("--edition=2021").arg(tmp_src).arg("-o").arg(&out_path)
                .status();
            match compile_status {
                Ok(s) if s.success() => {}
                Ok(_) => { return Err("Rust compilation failed".to_string()); }
                Err(e) => { return Err(format!("Failed to run rustc: {}", e)); }
            }
            // We'll run compiled binary; remember to cleanup later
            (tmp_src.to_string(), vec![out])
        }
        _ => { return Err(format!("Unsupported language for --test-file: {}", lang)); }
    };

    // Run shell script
    let shell_output = {
        let mut child = match Command::new("sh").arg(filename).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn() {
            Ok(c) => c,
            Err(e) => { cleanup_tmp(lang, &tmp_file); return Err(format!("Failed to spawn sh: {}", e)); }
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
            let bin = if cfg!(windows) { "__tmp_test_bin.exe" } else { "__tmp_test_bin" };
            let abs_bin = std::env::current_dir().unwrap_or_default().join(bin);
            let mut child = match Command::new(&abs_bin).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn() {
                Ok(c) => c,
                Err(e) => { cleanup_tmp(lang, &tmp_file); return Err(format!("Failed to run compiled Rust: {} ({})", e, abs_bin.display())); }
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
                Err(e) => { cleanup_tmp(lang, &tmp_file); return Err(format!("Failed to run translated program: {}", e)); }
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

    let success = shell_success == trans_success && shell_stdout == trans_stdout && shell_stderr == trans_stderr;

    Ok(TestResult {
        success,
        shell_stdout,
        shell_stderr,
        translated_stdout: trans_stdout,
        translated_stderr: trans_stderr,
        shell_exit: shell_output.status.code().unwrap_or(-1),
        translated_exit: translated_output.status.code().unwrap_or(-1),
    })
}

fn cleanup_tmp(lang: &str, tmp_file: &str) {
    let _ = fs::remove_file(tmp_file);
    if lang == "rust" {
        let _ = fs::remove_file("__tmp_test_bin");
        if cfg!(windows) {
            let _ = fs::remove_file(format!("{}.exe", "__tmp_test_bin"));
            let _ = fs::remove_file(format!("{}.pdb", "__tmp_test_bin"));
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

fn check_generator_available(generator: &str) -> bool {
    match generator {
        "perl" => Command::new("perl").arg("--version").output().is_ok(),
        "python" => Command::new("python3").arg("--version").output().is_ok() || Command::new("python").arg("--version").output().is_ok(),
        "rust" => Command::new("rustc").arg("--version").output().is_ok(),
        "lua" => Command::new("lua").arg("-v").output().is_ok(),
        "js" => Command::new("node").arg("--version").output().is_ok(),
        "ps" => {
            let shell = if cfg!(windows) { "powershell" } else { "pwsh" };
            Command::new(shell).arg("-Command").arg("$PSVersionTable").output().is_ok()
        },
        _ => false
    }
}

fn test_all_examples() {
    let all_generators = vec!["perl", "python", "rust", "lua", "js", "ps"];
    
    // Filter to only available generators
    let generators: Vec<_> = all_generators.into_iter()
        .filter(|g| {
            let available = check_generator_available(g);
            if !available {
                println!("Skipping {} tests - {} not found in PATH", g, g);
            }
            available
        })
        .collect();
    
    if generators.is_empty() {
        println!("No supported generators found. Please install at least one of: perl, python3, rustc, lua, node, powershell/pwsh");
        std::process::exit(1);
    }
    
    let mut examples = Vec::new();
    
    // Read examples directory
    if let Ok(entries) = fs::read_dir("examples") {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("sh") {
                    if let Some(path_str) = path.to_str() {
                        examples.push(path_str.to_string());
                    }
                }
            }
        }
    }
    
    // Sort examples for consistent output
    examples.sort();
    
    // Test each combination
    let mut results = Vec::new();
    let total_tests = examples.len() * generators.len();
    let mut passed_tests = 0;
    let mut current_test = 0;
    
    println!("\nRunning {} tests across {} examples and {} generators", 
             total_tests, examples.len(), generators.len());
    println!("{}", "=".repeat(50));
    
    for example in &examples {
        for generator in &generators {
            current_test += 1;
            print!("\rTest {}/{}: {} with {:<8} ", 
                  current_test, total_tests, 
                  example.replace("examples/", "").replace("examples\\", ""), 
                  generator);
            io::stdout().flush().unwrap();
            
            let mut success = true;
            let mut error_msg = String::new();
            
            // Run the actual test
            match test_file_equivalence(generator, example) {
                Ok(_) => {
                    passed_tests += 1;
                    print!("✓");
                }
                Err(e) => {
                    success = false;
                    error_msg = format!("Test failed for {} with {}: {}", example, generator, e);
                    print!("✗");
                }
            }
            
            results.push((example.to_string(), generator.to_string(), success, error_msg));
            io::stdout().flush().unwrap();
        }
    }
    
    // Clear screen and show prominent summary
    println!("\n\n");
    println!("{}", "=".repeat(80));
    println!("                                    TEST SUMMARY");
    println!("{}", "=".repeat(80));
    println!("Total tests: {}", total_tests);
    println!("Passed: {} ({:.1}%)", passed_tests, (passed_tests as f64 / total_tests as f64) * 100.0);
    println!("Failed: {}", total_tests - passed_tests);
    println!("{}", "=".repeat(80));
    
    // Per-generator summary
    println!("\n{}", "=".repeat(80));
    println!("                                PER-GENERATOR RESULTS");
    println!("{}", "=".repeat(80));
    for generator in &generators {
        let gen_results: Vec<_> = results.iter()
            .filter(|(_, g, _, _)| g == generator)
            .collect();
        let gen_passed = gen_results.iter().filter(|(_, _, success, _)| *success).count();
        let gen_total = gen_results.len();
        let percentage = (gen_passed as f64 / gen_total as f64) * 100.0;
        let status = if gen_passed == gen_total { "✓" } else if gen_passed > 0 { "⚠" } else { "✗" };
        println!("{:<8}: {}/{} passed ({:.1}%) {}", 
            generator, 
            gen_passed, 
            gen_total,
            percentage,
            status
        );
    }
    println!("{}", "=".repeat(80));

    // Per-example summary
    println!("\nResults by Example:");
    println!("{}", "=".repeat(50));
    for example in &examples {
        let example_results: Vec<_> = results.iter()
            .filter(|(e, _, _, _)| e == example)
            .collect();
        let example_passed = example_results.iter().filter(|(_, _, success, _)| *success).count();
        let example_total = example_results.len();
        
        // Only show examples with failures
        if example_passed < example_total {
            println!("{}: {}/{} passed", 
                example.replace("examples/", "").replace("examples\\", ""),
                example_passed,
                example_total
            );
            // Show which generators failed
            for (_, generator, success, error_msg) in example_results {
                if !success {
                    println!("  ✗ {}: {}", generator, error_msg);
                }
            }
        }
    }
    
    // Final summary line
    println!("\n{}", "=".repeat(80));
    if passed_tests < total_tests {
        println!("FINAL RESULT: {} out of {} tests PASSED ({:.1}%)", passed_tests, total_tests, (passed_tests as f64 / total_tests as f64) * 100.0);
        std::process::exit(1);
    } else {
        println!("FINAL RESULT: ALL {} tests PASSED! 🎉", total_tests);
    }
    println!("{}", "=".repeat(80));
}

/// Truncate output to specified number of lines, adding ellipsis if truncated
fn truncate_output(output: &str, max_lines: usize) -> String {
    let lines: Vec<&str> = output.lines().collect();
    if lines.len() <= max_lines {
        output.to_string()
    } else {
        let mut result = lines[..max_lines].join("\n");
        result.push_str("\n... (truncated, showing first ");
        result.push_str(&max_lines.to_string());
        result.push_str(" lines)");
        result
    }
}

fn test_all_examples_next_fail() {
    let all_generators = vec!["perl", "python", "rust", "lua", "js", "ps"];
    
    // Filter to only available generators
    let generators: Vec<_> = all_generators.into_iter()
        .filter(|g| {
            let available = check_generator_available(g);
            if !available {
                println!("Skipping {} tests - {} not found in PATH", g, g);
            }
            available
        })
        .collect();
    
    if generators.is_empty() {
        println!("No supported generators found. Please install at least one of: perl, python3, rustc, lua, node, powershell/pwsh");
        std::process::exit(1);
    }
    
    let mut examples = Vec::new();
    
    // Read examples directory
    if let Ok(entries) = fs::read_dir("examples") {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("sh") {
                    if let Some(path_str) = path.to_str() {
                        examples.push(path_str.to_string());
                    }
                }
            }
        }
    }
    
    // Sort examples for consistent output
    examples.sort();
    
    // Test each combination
    let mut passed_tests = 0;
    let mut current_test = 0;
    let total_tests = examples.len() * generators.len();
    
    println!("\nRunning {} tests across {} examples and {} generators", 
             total_tests, examples.len(), generators.len());
    println!("{}", "=".repeat(50));
    
    for example in &examples {
        for generator in &generators {
            current_test += 1;
            print!("\rTest {}/{}: {} with {:<8} ", 
                  current_test, total_tests, 
                  example.replace("examples/", "").replace("examples\\", ""), 
                  generator);
            io::stdout().flush().unwrap();
            
            // Run the actual test
            match test_file_equivalence_detailed(generator, example) {
                Ok(result) => {
                    if result.success {
                        passed_tests += 1;
                        print!("✓");
                    } else {
                        // Test failed - show diff and exit
                        println!("\n\n");
                        println!("{}", "=".repeat(80));
                        println!("                                    TEST FAILED");
                        println!("{}", "=".repeat(80));
                        println!("File: {}", example);
                        println!("Generator: {}", generator);
                        println!("Test: {}/{}", current_test, total_tests);
                        println!("Tests passed before failure: {}", passed_tests);
                        println!("{}", "=".repeat(80));
                        
                        // Show exit code comparison
                        println!("\nExit Code Comparison:");
                        println!("Shell script exit code: {}", result.shell_exit);
                        println!("Translated code exit code: {}", result.translated_exit);
                        
                        // Show stdout diff
                        println!("\nSTDOUT Comparison:");
                        println!("Shell script stdout:");
                        println!("{}", truncate_output(&result.shell_stdout, 10));
                        println!("\nTranslated code stdout:");
                        println!("{}", truncate_output(&result.translated_stdout, 10));
                        
                        // Show stderr diff
                        println!("\nSTDERR Comparison:");
                        println!("Shell script stderr:");
                        println!("{}", truncate_output(&result.shell_stderr, 10));
                        println!("\nTranslated code stderr:");
                        println!("{}", truncate_output(&result.translated_stderr, 10));
                        
                        // Show summary
                        println!("\n{}", "=".repeat(80));
                        println!("SUMMARY: {} out of {} tests passed before first failure", passed_tests, total_tests);
                        println!("{}", "=".repeat(80));
                        
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    // Test error - show error and exit
                    println!("\n\n");
                    println!("{}", "=".repeat(80));
                    println!("                                    TEST ERROR");
                    println!("{}", "=".repeat(80));
                    println!("File: {}", example);
                    println!("Generator: {}", generator);
                    println!("Test: {}/{}", current_test, total_tests);
                    println!("Tests passed before error: {}", passed_tests);
                    println!("Error: {}", e);
                    println!("{}", "=".repeat(80));
                    
                    // Show summary
                    println!("\n{}", "=".repeat(80));
                    println!("SUMMARY: {} out of {} tests passed before first error", passed_tests, total_tests);
                    println!("{}", "=".repeat(80));
                    
                    std::process::exit(1);
                }
            }
            
            io::stdout().flush().unwrap();
        }
    }
    
    // All tests passed
    println!("\n\n");
    println!("{}", "=".repeat(80));
    println!("                                    ALL TESTS PASSED! 🎉");
    println!("{}", "=".repeat(80));
    println!("Total tests: {}", total_tests);
    println!("Passed: {} (100%)", passed_tests);
    println!("{}", "=".repeat(80));
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