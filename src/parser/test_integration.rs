#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::*;

    #[test]
    fn test_parser_module_structure() {
        // Test that we can create a parser
        let mut parser = Parser::new("echo hello world");
        assert!(parser.parse().is_ok());
    }

    #[test]
    fn test_parse_simple_command() {
        let mut parser = Parser::new("echo hello world");
        let commands = parser.parse().unwrap();
        assert_eq!(commands.len(), 1);
        
        if let crate::ast::Command::Simple(cmd) = &commands[0] {
            assert!(matches!(&cmd.name, crate::ast::Word::Literal(name) if name == "echo"));
            assert_eq!(cmd.args.len(), 2);
        } else {
            panic!("Expected Simple command");
        }
    }
}

