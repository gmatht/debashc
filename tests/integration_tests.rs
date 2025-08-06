use sh2perl::{Lexer, Parser, PerlGenerator, Token};

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
    
    assert!(perl_code.contains("print("));
    assert!(perl_code.contains("escaped"));
    assert!(perl_code.contains("quotes"));
    
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