use sh2perl::{Lexer, Parser, Token};

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