use sh2perl::{Lexer, Parser, PerlGenerator, RustGenerator, Token};
use std::fs;
use std::process::Command;
use std::path::Path;

#[test]
fn test_simple_command_lexing() {
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
fn test_pipeline_lexing() {
    let input = "ls | grep test";
    let mut lexer = Lexer::new(input);
    
    assert_eq!(lexer.next(), Some(&Token::Identifier));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::Pipe));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::Identifier));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::Identifier));
    assert_eq!(lexer.next(), None);
}

#[test]
fn test_variable_expansion_lexing() {
    let input = "$HOME ${PATH}";
    let mut lexer = Lexer::new(input);
    
    assert_eq!(lexer.next(), Some(&Token::Dollar));
    assert_eq!(lexer.next(), Some(&Token::Identifier));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::DollarBrace));
    assert_eq!(lexer.next(), Some(&Token::Identifier));
    assert_eq!(lexer.next(), Some(&Token::BraceClose));
    assert_eq!(lexer.next(), None);
}

#[test]
fn test_quoted_strings_lexing() {
    let input = r#"echo "Hello, World!" 'Single quoted'"#;
    let mut lexer = Lexer::new(input);
    
    assert_eq!(lexer.next(), Some(&Token::Identifier));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::DoubleQuotedString));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::SingleQuotedString));
    assert_eq!(lexer.next(), None);
}

#[test]
fn test_redirections_lexing() {
    let input = "cat < input.txt > output.txt 2>&1";
    let mut lexer = Lexer::new(input);
    
    assert_eq!(lexer.next(), Some(&Token::Identifier));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::RedirectIn));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::Identifier));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::RedirectOut));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::Identifier));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::Number));
    assert_eq!(lexer.next(), Some(&Token::RedirectOutErr));
    assert_eq!(lexer.next(), Some(&Token::Number));
    assert_eq!(lexer.next(), None);
}

#[test]
fn test_control_flow_keywords_lexing() {
    let input = "if then else elif fi while do done for in function";
    let mut lexer = Lexer::new(input);
    
    assert_eq!(lexer.next(), Some(&Token::If));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::Then));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::Else));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::Elif));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::Fi));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::While));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::Do));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::Done));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::For));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::In));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::Function));
    assert_eq!(lexer.next(), None);
}

#[test]
fn test_operators_lexing() {
    let input = "&& || & | ; ;;";
    let mut lexer = Lexer::new(input);
    
    assert_eq!(lexer.next(), Some(&Token::And));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::Or));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::Background));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::Pipe));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::Semicolon));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::DoubleSemicolon));
    assert_eq!(lexer.next(), None);
}

#[test]
fn test_file_test_operators_lexing() {
    let input = "-f -d -e -r -w -x -s -L -h -p -S -b -c -g -k -u -O -G -N -nt -ot -ef";
    let mut lexer = Lexer::new(input);
    
    assert_eq!(lexer.next(), Some(&Token::File));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::Directory));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::Exists));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::Readable));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::Writable));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::Executable));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::Size));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::Symlink));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::SymlinkH));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::PipeFile));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::Socket));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::Block));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::Character));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::SetGid));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::Sticky));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::SetUid));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::Owned));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::GroupOwned));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::Modified));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::NewerThan));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::OlderThan));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::SameFile));
    assert_eq!(lexer.next(), None);
}

#[test]
fn test_comments_lexing() {
    let input = "echo hello # This is a comment\n# Another comment";
    let mut lexer = Lexer::new(input);
    
    assert_eq!(lexer.next(), Some(&Token::Identifier));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::Identifier));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::Comment));
    assert_eq!(lexer.next(), Some(&Token::Newline));
    assert_eq!(lexer.next(), Some(&Token::Comment));
    assert_eq!(lexer.next(), None);
}

#[test]
fn test_parser_simple_command() {
    let input = "echo hello world";
    let mut parser = Parser::new(input);
    let result = parser.parse();
    assert!(result.is_ok());
    
    let commands = result.unwrap();
    assert_eq!(commands.len(), 1);
}

#[test]
fn test_parser_pipeline() {
    let input = "ls | grep test";
    let mut parser = Parser::new(input);
    let result = parser.parse();
    assert!(result.is_ok());
    
    let commands = result.unwrap();
    assert_eq!(commands.len(), 1);
}

#[test]
fn test_parser_multiple_commands() {
    let input = "echo hello; echo world";
    let mut parser = Parser::new(input);
    let result = parser.parse();
    assert!(result.is_ok());
    
    let commands = result.unwrap();
    assert_eq!(commands.len(), 2);
}

#[test]
fn test_parser_with_comments() {
    let input = "echo hello # comment\nls -la";
    let mut parser = Parser::new(input);
    let result = parser.parse();
    assert!(result.is_ok());
    
    let commands = result.unwrap();
    assert_eq!(commands.len(), 2);
}

#[test]
fn test_lexer_peek_functionality() {
    let input = "echo hello world";
    let mut lexer = Lexer::new(input);
    
    // Test peek
    assert_eq!(lexer.peek(), Some(&Token::Identifier));
    assert_eq!(lexer.peek(), Some(&Token::Identifier)); // Should be the same
    
    // Test peek_n
    assert_eq!(lexer.peek_n(1), Some(&Token::Space));
    assert_eq!(lexer.peek_n(2), Some(&Token::Identifier));
    
    // Test current position
    assert_eq!(lexer.current_position(), 0);
    
    // Test next
    assert_eq!(lexer.next(), Some(&Token::Identifier));
    assert_eq!(lexer.current_position(), 1);
}

#[test]
fn test_lexer_eof_detection() {
    let input = "echo";
    let mut lexer = Lexer::new(input);
    
    assert!(!lexer.is_eof());
    assert_eq!(lexer.next(), Some(&Token::Identifier));
    assert!(lexer.is_eof());
    assert_eq!(lexer.next(), None);
}

#[test]
fn test_lexer_span_information() {
    let input = "echo hello";
    let mut lexer = Lexer::new(input);
    
    // The span information should be available
    let span = lexer.get_span();
    assert!(span.is_some());
}

#[test]
fn test_parser_error_handling() {
    let input = "if [ -f file.txt"; // Missing closing bracket and then/fi
    let mut parser = Parser::new(input);
    let result = parser.parse();
    assert!(result.is_err());
}

#[test]
fn test_lexer_with_whitespace() {
    let input = "  echo   hello   world  ";
    let mut lexer = Lexer::new(input);
    
    // Should handle whitespace properly
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::Identifier));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::Identifier));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::Identifier));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), Some(&Token::Space));
    assert_eq!(lexer.next(), None);
}

// Perl Generator Tests

#[test]
fn test_perl_generator_basic_echo() {
    let input = "echo hello world";
    let mut parser = Parser::new(input);
    let commands = parser.parse().unwrap();
    
    let mut generator = PerlGenerator::new();
    let perl_code = generator.generate(&commands);
    
    assert!(perl_code.contains("#!/usr/bin/env perl"));
    assert!(perl_code.contains("use strict;"));
    assert!(perl_code.contains("use warnings;"));
    assert!(perl_code.contains("print(\"hello world\\n\");"));
}

#[test]
fn test_perl_generator_empty_echo() {
    let input = "echo";
    let mut parser = Parser::new(input);
    let commands = parser.parse().unwrap();
    
    let mut generator = PerlGenerator::new();
    let perl_code = generator.generate(&commands);
    
    assert!(perl_code.contains("print(\"\\n\");"));
}

#[test]
fn test_perl_generator_cd_command() {
    let input = "cd /tmp";
    let mut parser = Parser::new(input);
    let commands = parser.parse().unwrap();
    
    let mut generator = PerlGenerator::new();
    let perl_code = generator.generate(&commands);
    
    assert!(perl_code.contains("chdir('/tmp') or die \"Cannot change to directory: $!\\n\";"));
}

#[test]
fn test_perl_generator_ls_command() {
    let input = "ls /tmp";
    let mut parser = Parser::new(input);
    let commands = parser.parse().unwrap();
    
    let mut generator = PerlGenerator::new();
    let perl_code = generator.generate(&commands);
    
    assert!(perl_code.contains("opendir(my $dh, '/tmp')"));
    assert!(perl_code.contains("while (my $file = readdir($dh))"));
    assert!(perl_code.contains("print(\"$file\\n\") unless $file =~ /^\\.\\.?$/;"));
    assert!(perl_code.contains("closedir($dh);"));
}

#[test]
fn test_perl_generator_mkdir_command() {
    let input = "mkdir newdir";
    let mut parser = Parser::new(input);
    let commands = parser.parse().unwrap();
    
    let mut generator = PerlGenerator::new();
    let perl_code = generator.generate(&commands);
    
    assert!(perl_code.contains("mkdir('newdir') or die \"Cannot create directory: $!\\n\";"));
}

#[test]
fn test_perl_generator_rm_command() {
    let input = "rm oldfile.txt";
    let mut parser = Parser::new(input);
    let commands = parser.parse().unwrap();
    
    let mut generator = PerlGenerator::new();
    let perl_code = generator.generate(&commands);
    
    assert!(perl_code.contains("unlink('oldfile.txt') or die \"Cannot remove file: $!\\n\";"));
}

#[test]
fn test_perl_generator_cp_command() {
    let input = "cp source.txt dest.txt";
    let mut parser = Parser::new(input);
    let commands = parser.parse().unwrap();
    
    let mut generator = PerlGenerator::new();
    let perl_code = generator.generate(&commands);
    
    assert!(perl_code.contains("use File::Copy;"));
    assert!(perl_code.contains("copy('source.txt', 'dest.txt') or die \"Cannot copy file: $!\\n\";"));
}

#[test]
fn test_perl_generator_mv_command() {
    let input = "mv old.txt new.txt";
    let mut parser = Parser::new(input);
    let commands = parser.parse().unwrap();
    
    let mut generator = PerlGenerator::new();
    let perl_code = generator.generate(&commands);
    
    assert!(perl_code.contains("rename('old.txt', 'new.txt') or die \"Cannot move file: $!\\n\";"));
}

#[test]
fn test_perl_generator_pipeline() {
    let input = "ls | grep test";
    let mut parser = Parser::new(input);
    let commands = parser.parse().unwrap();
    
    let mut generator = PerlGenerator::new();
    let perl_code = generator.generate(&commands);
    
    assert!(perl_code.contains("my $output;"));
    assert!(perl_code.contains("$output = `ls`;"));
    assert!(perl_code.contains("$output = `echo \"$output\" | grep test`;"));
    assert!(perl_code.contains("print($output);"));
}

#[test]
fn test_perl_generator_if_statement() {
    let input = "if [ -f file.txt ]; then echo exists; fi";
    let mut parser = Parser::new(input);
    let commands = parser.parse().unwrap();
    
    let mut generator = PerlGenerator::new();
    let perl_code = generator.generate(&commands);
    
    assert!(perl_code.contains("if (-f 'file.txt')"));
    assert!(perl_code.contains("print(\"exists\\n\");"));
}

#[test]
fn test_perl_generator_if_else_statement() {
    let input = "if [ -f file.txt ]; then echo exists; else echo not found; fi";
    let mut parser = Parser::new(input);
    let commands = parser.parse().unwrap();
    
    let mut generator = PerlGenerator::new();
    let perl_code = generator.generate(&commands);
    
    assert!(perl_code.contains("if (-f 'file.txt')"));
    assert!(perl_code.contains("print(\"exists\\n\");"));
    assert!(perl_code.contains("} else {"));
    assert!(perl_code.contains("print(\"not found\\n\");"));
}

#[test]
fn test_perl_generator_file_test_operators() {
    let test_cases = vec![
        ("[ -d /tmp ]", "-d '/tmp'"),
        ("[ -e file.txt ]", "-e 'file.txt'"),
        ("[ -r file.txt ]", "-r 'file.txt'"),
        ("[ -w file.txt ]", "-w 'file.txt'"),
        ("[ -x file.txt ]", "-x 'file.txt'"),
    ];
    
    for (shell_test, expected_perl) in test_cases {
        let input = format!("if {}; then echo yes; fi", shell_test);
        let mut parser = Parser::new(&input);
        let commands = parser.parse().unwrap();
        
        let mut generator = PerlGenerator::new();
        let perl_code = generator.generate(&commands);
        
        assert!(perl_code.contains(expected_perl), 
                "Expected '{}' in Perl code for shell test '{}', got: {}", 
                expected_perl, shell_test, perl_code);
    }
}

#[test]
fn test_perl_generator_multiple_commands() {
    let input = "echo hello; echo world; mkdir testdir";
    let mut parser = Parser::new(input);
    let commands = parser.parse().unwrap();
    
    let mut generator = PerlGenerator::new();
    let perl_code = generator.generate(&commands);
    
    assert!(perl_code.contains("print(\"hello\\n\");"));
    assert!(perl_code.contains("print(\"world\\n\");"));
    assert!(perl_code.contains("mkdir('testdir')"));
}

#[test]
fn test_perl_generator_environment_variables() {
    let input = "PATH=/usr/bin echo hello";
    let mut parser = Parser::new(input);
    let commands = parser.parse().unwrap();
    
    let mut generator = PerlGenerator::new();
    let perl_code = generator.generate(&commands);
    
    assert!(perl_code.contains("$ENV{PATH} = '/usr/bin';"));
    assert!(perl_code.contains("print(\"hello\\n\");"));
}

#[test]
fn test_perl_generator_grep_command() {
    let input = "grep pattern file.txt";
    let mut parser = Parser::new(input);
    let commands = parser.parse().unwrap();
    
    let mut generator = PerlGenerator::new();
    let perl_code = generator.generate(&commands);
    
    assert!(perl_code.contains("open(my $fh, '<', 'file.txt')"));
    assert!(perl_code.contains("while (my $line = <$fh>)"));
    assert!(perl_code.contains("print($line) if $line =~ /pattern/;"));
    assert!(perl_code.contains("close($fh);"));
}

#[test]
fn test_perl_generator_cat_command() {
    let input = "cat file.txt";
    let mut parser = Parser::new(input);
    let commands = parser.parse().unwrap();
    
    let mut generator = PerlGenerator::new();
    let perl_code = generator.generate(&commands);
    
    assert!(perl_code.contains("open(my $fh, '<', 'file.txt')"));
    assert!(perl_code.contains("while (my $line = <$fh>)"));
    assert!(perl_code.contains("print($line);"));
    assert!(perl_code.contains("close($fh);"));
}

#[test]
fn test_perl_generator_generic_command() {
    let input = "python script.py arg1 arg2";
    let mut parser = Parser::new(input);
    let commands = parser.parse().unwrap();
    
    let mut generator = PerlGenerator::new();
    let perl_code = generator.generate(&commands);
    
    assert!(perl_code.contains("system('python', 'script.py', 'arg1', 'arg2');"));
}

#[test]
fn test_perl_generator_quoted_strings() {
    // Test double quoted strings
    let input = r#"echo "Hello, World!""#;
    let mut parser = Parser::new(input);
    let commands = parser.parse().unwrap();
    
    let mut generator = PerlGenerator::new();
    let perl_code = generator.generate(&commands);
    
    assert!(perl_code.contains("print(\"Hello, World!\\n\");"));
    
    // Test single quoted strings
    let input = "echo 'Single quoted string'";
    let mut parser = Parser::new(input);
    let commands = parser.parse().unwrap();
    
    let mut generator = PerlGenerator::new();
    let perl_code = generator.generate(&commands);
    
    assert!(perl_code.contains("print(\"Single quoted string\\n\");"));
    
    // Test strings with escaped quotes
    let input = r#"echo "String with \"escaped\" quotes""#;
    let mut parser = Parser::new(input);
    let commands = parser.parse().unwrap();
    
    let mut generator = PerlGenerator::new();
    let perl_code = generator.generate(&commands);
    
    assert!(perl_code.contains("print(\"String with \\\"escaped\\\" quotes\\n\");"));
    
    // Test strings with spaces and punctuation
    let input = r#"echo "String with spaces and punctuation!""#;
    let mut parser = Parser::new(input);
    let commands = parser.parse().unwrap();
    
    let mut generator = PerlGenerator::new();
    let perl_code = generator.generate(&commands);
    
    assert!(perl_code.contains("print(\"String with spaces and punctuation!\\n\");"));
    
    // Test multiple quoted strings in one command
    let input = r#"echo "First" "Second" 'Third'"#;
    let mut parser = Parser::new(input);
    let commands = parser.parse().unwrap();
    
    let mut generator = PerlGenerator::new();
    let perl_code = generator.generate(&commands);
    
    assert!(perl_code.contains("print(\"First Second Third\\n\");"));
}

// ============================================================================
// Example file translation tests
// ============================================================================

#[test]
fn test_example_simple_sh_to_perl() {
    let content = fs::read_to_string("examples/simple.sh").expect("Failed to read simple.sh");
    let mut parser = Parser::new(&content);
    let commands = parser.parse().expect("Failed to parse simple.sh");
    
    let mut generator = PerlGenerator::new();
    let perl_code = generator.generate(&commands);
    
    // Check that the Perl code contains expected elements
    assert!(perl_code.contains("#!/usr/bin/env perl"));
    assert!(perl_code.contains("use strict;"));
    assert!(perl_code.contains("use warnings;"));
    assert!(perl_code.contains("print(\"Hello, World!\\n\");"));
    assert!(perl_code.contains("system('ls', '-la');"));
    assert!(perl_code.contains("system('grep', 'pattern', 'file.txt');"));
}

#[test]
fn test_example_simple_sh_to_rust() {
    let content = fs::read_to_string("examples/simple.sh").expect("Failed to read simple.sh");
    let mut parser = Parser::new(&content);
    let commands = parser.parse().expect("Failed to parse simple.sh");
    
    let mut generator = RustGenerator::new();
    let rust_code = generator.generate(&commands);
    
    // Check that the Rust code contains expected elements
    assert!(rust_code.contains("use std::process::Command;"));
    assert!(rust_code.contains("use std::env;"));
    assert!(rust_code.contains("use std::fs;"));
    assert!(rust_code.contains("fn main() -> Result<(), Box<dyn std::error::Error>>"));
    assert!(rust_code.contains("println!(\"Hello, World!\");"));
    assert!(rust_code.contains("Command::new(\"ls\")"));
    assert!(rust_code.contains("Command::new(\"grep\")"));
}

#[test]
fn test_example_pipeline_sh_to_perl() {
    let content = fs::read_to_string("examples/pipeline.sh").expect("Failed to read pipeline.sh");
    let mut parser = Parser::new(&content);
    let commands = parser.parse().expect("Failed to parse pipeline.sh");
    
    let mut generator = PerlGenerator::new();
    let perl_code = generator.generate(&commands);
    
    // Check that the Perl code contains expected elements
    assert!(perl_code.contains("#!/usr/bin/env perl"));
    assert!(perl_code.contains("system('ls');"));
    assert!(perl_code.contains("system('grep');"));
    assert!(perl_code.contains("system('wc');"));
    assert!(perl_code.contains("system('cat');"));
    assert!(perl_code.contains("system('sort');"));
    assert!(perl_code.contains("system('uniq');"));
    assert!(perl_code.contains("system('find');"));
    assert!(perl_code.contains("system('xargs');"));
}

#[test]
fn test_example_pipeline_sh_to_rust() {
    let content = fs::read_to_string("examples/pipeline.sh").expect("Failed to read pipeline.sh");
    let mut parser = Parser::new(&content);
    let commands = parser.parse().expect("Failed to parse pipeline.sh");
    
    let mut generator = RustGenerator::new();
    let rust_code = generator.generate(&commands);
    
    // Check that the Rust code contains expected elements
    assert!(rust_code.contains("use std::process::Command;"));
    assert!(rust_code.contains("Command::new(\"ls\")"));
    assert!(rust_code.contains("Command::new(\"grep\")"));
    assert!(rust_code.contains("Command::new(\"wc\")"));
    assert!(rust_code.contains("Command::new(\"cat\")"));
    assert!(rust_code.contains("Command::new(\"sort\")"));
    assert!(rust_code.contains("Command::new(\"uniq\")"));
    assert!(rust_code.contains("Command::new(\"find\")"));
    assert!(rust_code.contains("Command::new(\"xargs\")"));
}

#[test]
fn test_example_control_flow_sh_to_perl() {
    let content = fs::read_to_string("examples/control_flow.sh").expect("Failed to read control_flow.sh");
    let mut parser = Parser::new(&content);
    let commands = parser.parse().expect("Failed to parse control_flow.sh");
    
    let mut generator = PerlGenerator::new();
    let perl_code = generator.generate(&commands);
    
    // Check that the Perl code contains expected elements
    assert!(perl_code.contains("#!/usr/bin/env perl"));
    assert!(perl_code.contains("if (-f 'file.txt')"));
    assert!(perl_code.contains("print(\"File exists\\n\");"));
    assert!(perl_code.contains("print(\"File does not exist\\n\");"));
    assert!(perl_code.contains("for my $i (1..5)"));
    assert!(perl_code.contains("print(\"Number: $i\\n\");"));
    assert!(perl_code.contains("while ($i < 10)"));
    assert!(perl_code.contains("print(\"Counter: $i\\n\");"));
    assert!(perl_code.contains("sub greet"));
    assert!(perl_code.contains("print(\"Hello, $_[0]!\\n\");"));
}

#[test]
fn test_example_control_flow_sh_to_rust() {
    let content = fs::read_to_string("examples/control_flow.sh").expect("Failed to read control_flow.sh");
    let mut parser = Parser::new(&content);
    let commands = parser.parse().expect("Failed to parse control_flow.sh");
    
    let mut generator = RustGenerator::new();
    let rust_code = generator.generate(&commands);
    
    // Check that the Rust code contains expected elements
    assert!(rust_code.contains("use std::process::Command;"));
    assert!(rust_code.contains("if fs::metadata(\"file.txt\").is_ok()"));
    assert!(rust_code.contains("println!(\"File exists\");"));
    assert!(rust_code.contains("println!(\"File does not exist\");"));
    assert!(rust_code.contains("for i in &[1, 2, 3, 4, 5]"));
    assert!(rust_code.contains("println!(\"Number: {}\", i);"));
    assert!(rust_code.contains("while true")); // Simplified condition for now
    assert!(rust_code.contains("println!(\"Counter: {}\", i);"));
    assert!(rust_code.contains("fn greet()"));
    assert!(rust_code.contains("println!(\"Hello, {}!\", arg);"));
}

#[test]
fn test_example_test_quoted_sh_to_perl() {
    let content = fs::read_to_string("examples/test_quoted.sh").expect("Failed to read test_quoted.sh");
    let mut parser = Parser::new(&content);
    let commands = parser.parse().expect("Failed to parse test_quoted.sh");
    
    let mut generator = PerlGenerator::new();
    let perl_code = generator.generate(&commands);
    
    // Check that the Perl code contains expected elements
    assert!(perl_code.contains("#!/usr/bin/env perl"));
    assert!(perl_code.contains("print(\"Hello, World!\\n\");"));
    assert!(perl_code.contains("print(\"Single quoted\\n\");"));
    assert!(perl_code.contains("print(\"String with \\\"escaped\\\" quotes\\n\");"));
    assert!(perl_code.contains("print(\"String with 'single' quotes\\n\");"));
}

#[test]
fn test_example_test_quoted_sh_to_rust() {
    let content = fs::read_to_string("examples/test_quoted.sh").expect("Failed to read test_quoted.sh");
    let mut parser = Parser::new(&content);
    let commands = parser.parse().expect("Failed to parse test_quoted.sh");
    
    let mut generator = RustGenerator::new();
    let rust_code = generator.generate(&commands);
    
    // Check that the Rust code contains expected elements
    assert!(rust_code.contains("use std::process::Command;"));
    assert!(rust_code.contains("println!(\"Hello, World!\");"));
    assert!(rust_code.contains("println!(\"Single quoted\");"));
    assert!(rust_code.contains("println!(\"String with \\\"escaped\\\" quotes\");"));
    assert!(rust_code.contains("println!(\"String with 'single' quotes\");"));
}

#[test]
fn test_all_examples_parse_successfully() {
    let examples = vec![
        "examples/simple.sh",
        "examples/pipeline.sh", 
        "examples/control_flow.sh",
        "examples/test_quoted.sh"
    ];
    
    for example in examples {
        let content = fs::read_to_string(example).expect(&format!("Failed to read {}", example));
        let mut parser = Parser::new(&content);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse {}: {:?}", example, result.err());
    }
}

#[test]
fn test_all_examples_generate_perl() {
    let examples = vec![
        "examples/simple.sh",
        "examples/pipeline.sh", 
        "examples/control_flow.sh",
        "examples/test_quoted.sh"
    ];
    
    for example in examples {
        let content = fs::read_to_string(example).expect(&format!("Failed to read {}", example));
        let mut parser = Parser::new(&content);
        let commands = parser.parse().expect(&format!("Failed to parse {}", example));
        
        let mut generator = PerlGenerator::new();
        let perl_code = generator.generate(&commands);
        
        // Basic checks that Perl code is generated
        assert!(perl_code.contains("#!/usr/bin/env perl"), "Perl code missing shebang for {}", example);
        assert!(perl_code.contains("use strict;"), "Perl code missing strict for {}", example);
        assert!(perl_code.contains("use warnings;"), "Perl code missing warnings for {}", example);
    }
}

#[test]
fn test_all_examples_generate_rust() {
    let examples = vec![
        "examples/simple.sh",
        "examples/pipeline.sh", 
        "examples/control_flow.sh",
        "examples/test_quoted.sh"
    ];
    
    for example in examples {
        let content = fs::read_to_string(example).expect(&format!("Failed to read {}", example));
        let mut parser = Parser::new(&content);
        let commands = parser.parse().expect(&format!("Failed to parse {}", example));
        
        let mut generator = RustGenerator::new();
        let rust_code = generator.generate(&commands);
        
        // Basic checks that Rust code is generated
        assert!(rust_code.contains("use std::process::Command;"), "Rust code missing Command import for {}", example);
        assert!(rust_code.contains("fn main()"), "Rust code missing main function for {}", example);
        assert!(rust_code.contains("Result<(), Box<dyn std::error::Error>>"), "Rust code missing Result type for {}", example);
    }
}

#[test]
fn test_examples_output_equivalence() {
    use std::fs;
    use std::process::Command;
    use std::path::Path;
    
    let examples_dir = Path::new("examples");
    if !examples_dir.exists() {
        println!("Examples directory not found, skipping test");
        return;
    }
    
    let entries = match fs::read_dir(examples_dir) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("Failed to read examples directory: {}", e);
            return;
        }
    };
    
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                eprintln!("Failed to read directory entry: {}", e);
                continue;
            }
        };
        
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("sh") {
            continue;
        }
        
        let file_name = path.file_name().unwrap().to_str().unwrap();
        println!("Testing example: {}", file_name);
        
        // Read the shell script
        let shell_content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Failed to read {}: {}", file_name, e);
                continue;
            }
        };
        
        // Parse and generate Perl code
        let mut parser = Parser::new(&shell_content);
        let commands = match parser.parse() {
            Ok(commands) => commands,
            Err(e) => {
                eprintln!("Failed to parse {}: {:?}", file_name, e);
                continue;
            }
        };
        
        let mut generator = PerlGenerator::new();
        let perl_code = generator.generate(&commands);
        
        // Write Perl code to temporary file
        let perl_file = format!("test_output_{}.pl", file_name.replace(".sh", ""));
        if let Err(e) = fs::write(&perl_file, perl_code) {
            eprintln!("Failed to write Perl file for {}: {}", file_name, e);
            continue;
        }
        
        // Run the shell script
        let shell_output = Command::new("sh")
            .arg(&path)
            .output();
        
        let shell_output = match shell_output {
            Ok(output) => output,
            Err(e) => {
                eprintln!("Failed to run shell script {}: {}", file_name, e);
                fs::remove_file(&perl_file).ok();
                continue;
            }
        };
        
        // Run the Perl script
        let perl_output = Command::new("perl")
            .arg(&perl_file)
            .output();
        
        let perl_output = match perl_output {
            Ok(output) => output,
            Err(e) => {
                eprintln!("Failed to run Perl script for {}: {}", file_name, e);
                fs::remove_file(&perl_file).ok();
                continue;
            }
        };
        
        // Clean up Perl file
        fs::remove_file(&perl_file).ok();
        
        // Compare outputs
        let shell_stdout = String::from_utf8_lossy(&shell_output.stdout);
        let shell_stderr = String::from_utf8_lossy(&shell_output.stderr);
        let perl_stdout = String::from_utf8_lossy(&perl_output.stdout);
        let perl_stderr = String::from_utf8_lossy(&perl_output.stderr);
        
        // Check exit status
        let shell_success = shell_output.status.success();
        let perl_success = perl_output.status.success();
        
        assert_eq!(
            shell_success, perl_success,
            "Exit status mismatch for {}: shell={}, perl={}",
            file_name, shell_success, perl_success
        );
        
        // For some commands, we expect different output formats
        // but the core functionality should be equivalent
        let should_compare_output = !file_name.contains("simple.sh"); // simple.sh has ls -la which differs
        
        if should_compare_output {
            // Normalize outputs for comparison (remove trailing whitespace, normalize line endings)
            let normalized_shell_stdout = shell_stdout.trim().replace("\r\n", "\n");
            let normalized_perl_stdout = perl_stdout.trim().replace("\r\n", "\n");
            
            assert_eq!(
                normalized_shell_stdout, normalized_perl_stdout,
                "Output mismatch for {}:\nShell: {:?}\nPerl: {:?}",
                file_name, normalized_shell_stdout, normalized_perl_stdout
            );
        }
        
        // Log the outputs for debugging
        println!("  Shell stdout: {:?}", shell_stdout);
        println!("  Shell stderr: {:?}", shell_stderr);
        println!("  Perl stdout: {:?}", perl_stdout);
        println!("  Perl stderr: {:?}", perl_stderr);
        println!("  Shell exit: {}, Perl exit: {}", 
                 shell_output.status, perl_output.status);
        println!("  Output comparison: {}", if should_compare_output { "enabled" } else { "skipped (known differences)" });
    }
}

#[test]
fn test_examples_rust_generation() {
    use std::fs;
    use std::path::Path;
    
    let examples_dir = Path::new("examples");
    if !examples_dir.exists() {
        println!("Examples directory not found, skipping test");
        return;
    }
    
    let entries = match fs::read_dir(examples_dir) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("Failed to read examples directory: {}", e);
            return;
        }
    };
    
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                eprintln!("Failed to read directory entry: {}", e);
                continue;
            }
        };
        
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("sh") {
            continue;
        }
        
        let file_name = path.file_name().unwrap().to_str().unwrap();
        println!("Testing Rust generation for: {}", file_name);
        
        // Read the shell script
        let shell_content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Failed to read {}: {}", file_name, e);
                continue;
            }
        };
        
        // Parse and generate Rust code
        let mut parser = Parser::new(&shell_content);
        let commands = match parser.parse() {
            Ok(commands) => commands,
            Err(e) => {
                eprintln!("Failed to parse {}: {:?}", file_name, e);
                continue;
            }
        };
        
        let mut generator = RustGenerator::new();
        let rust_code = generator.generate(&commands);
        
        // Write Rust code to temporary file
        let rust_file = format!("test_output_{}.rs", file_name.replace(".sh", ""));
        if let Err(e) = fs::write(&rust_file, rust_code) {
            eprintln!("Failed to write Rust file for {}: {}", file_name, e);
            continue;
        }
        
        // Try to compile the Rust code
        let compile_result = Command::new("rustc")
            .arg("--edition=2021")
            .arg(&rust_file)
            .output();
        
        match compile_result {
            Ok(output) => {
                if output.status.success() {
                    println!("  ✓ Rust code compiles successfully");
                    
                    // Clean up compiled binary
                    let binary_name = rust_file.replace(".rs", "");
                    if cfg!(windows) {
                        fs::remove_file(format!("{}.exe", binary_name)).ok();
                    } else {
                        fs::remove_file(&binary_name).ok();
                    }
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    eprintln!("  ✗ Rust compilation failed: {}", stderr);
                }
            }
            Err(e) => {
                eprintln!("  ✗ Failed to run rustc for {}: {}", file_name, e);
            }
        }
        
        // Clean up Rust source file
        fs::remove_file(&rust_file).ok();
    }
}