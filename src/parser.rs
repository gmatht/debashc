use crate::ast::*;
use crate::lexer::{Lexer, Token, LexerError};
use thiserror::Error;
use std::collections::HashMap;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("Lexer error: {0}")]
    Lexer(#[from] LexerError),
    #[error("Unexpected token: {0:?}")]
    UnexpectedToken(Token),
    #[error("Expected token: {0:?}")]
    ExpectedToken(Token),
    #[error("Unexpected end of input")]
    UnexpectedEOF,
    #[error("Invalid syntax: {0}")]
    InvalidSyntax(String),
}

pub struct Parser {
    lexer: Lexer,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        Self {
            lexer: Lexer::new(input),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Command>, ParserError> {
        let mut commands = Vec::new();
        
        // Skip initial whitespace and comments
        self.skip_whitespace_and_comments();
        
        while !self.lexer.is_eof() {
            // Check if we're at EOF after skipping whitespace and comments
            if self.lexer.is_eof() {
                break;
            }
            
            // Check if we're at a newline or semicolon (empty command)
            if let Some(token) = self.lexer.peek() {
                match token {
                    Token::Newline | Token::Semicolon => {
                        self.lexer.next(); // consume the separator
                        self.skip_whitespace_and_comments();
                        continue;
                    }
                    _ => {}
                }
            }
            
            let command = self.parse_command()?;
            commands.push(command);
            
            // Handle semicolons and newlines
            while let Some(token) = self.lexer.peek() {
                match token {
                    Token::Semicolon | Token::Newline => {
                        self.lexer.next();
                    }
                    _ => break,
                }
            }
            
            // Skip whitespace and comments before next command
            self.skip_whitespace_and_comments();
        }
        
        Ok(commands)
    }

    fn parse_command(&mut self) -> Result<Command, ParserError> {
        // Skip whitespace and comments
        self.skip_whitespace_and_comments();
        
        if self.lexer.is_eof() {
            return Err(ParserError::UnexpectedEOF);
        }

        match self.lexer.peek() {
            Some(Token::If) => self.parse_if_statement(),
            Some(Token::While) => self.parse_while_loop(),
            Some(Token::For) => self.parse_for_loop(),
            Some(Token::Function) => self.parse_function(),
            Some(Token::ParenOpen) => self.parse_subshell(),
            Some(Token::Semicolon) => {
                // Skip semicolon and continue parsing
                self.lexer.next();
                self.parse_command()
            }
            _ => self.parse_pipeline(),
        }
    }

    fn parse_pipeline(&mut self) -> Result<Command, ParserError> {
        let mut commands = Vec::new();
        let mut operators = Vec::new();
        
        commands.push(self.parse_simple_command()?);
        
        while let Some(token) = self.lexer.peek() {
            match token {
                Token::Pipe => {
                    self.lexer.next();
                    operators.push(PipeOperator::Pipe);
                    self.skip_whitespace_and_comments();
                    commands.push(self.parse_simple_command()?);
                }
                Token::And => {
                    self.lexer.next();
                    operators.push(PipeOperator::And);
                    self.skip_whitespace_and_comments();
                    commands.push(self.parse_simple_command()?);
                }
                Token::Or => {
                    self.lexer.next();
                    operators.push(PipeOperator::Or);
                    self.skip_whitespace_and_comments();
                    commands.push(self.parse_simple_command()?);
                }
                Token::Semicolon | Token::Newline => {
                    // Stop parsing pipeline when we hit a command separator
                    break;
                }
                _ => break,
            }
        }
        
        if commands.len() == 1 {
            Ok(commands.remove(0))
        } else {
            Ok(Command::Pipeline(Pipeline { commands, operators }))
        }
    }

    fn parse_simple_command(&mut self) -> Result<Command, ParserError> {
        let mut args = Vec::new();
        let mut redirects = Vec::new();
        let mut env_vars = HashMap::new();
        
        // Parse environment variables (only at the beginning)
        while let Some(token) = self.lexer.peek() {
            match token {
                Token::Identifier => {
                    if let Some(Token::Assign) = self.lexer.peek_n(1) {
                        // Check if this is followed by a command (not just a value)
                        let var_name = self.get_identifier_text()?;
                        self.lexer.next(); // consume =
                        
                        // Parse the value until we hit a space or end
                        let mut value_parts = Vec::new();
                        while let Some(next_token) = self.lexer.peek() {
                            match next_token {
                                Token::Identifier | Token::Number => {
                                    value_parts.push(self.get_identifier_text()?);
                                }
                                Token::Arithmetic => {
                                    // Handle arithmetic expressions like $((i + 1))
                                    self.lexer.next(); // consume the Arithmetic token
                                    value_parts.push("$(".to_string());
                                    // Skip until we find the closing parentheses
                                    while let Some(token) = self.lexer.peek() {
                                        match token {
                                            Token::ParenClose => {
                                                self.lexer.next();
                                                value_parts.push(")".to_string());
                                                // Check if there's another closing parenthesis
                                                if let Some(Token::ParenClose) = self.lexer.peek() {
                                                    self.lexer.next();
                                                    value_parts.push(")".to_string());
                                                }
                                                break;
                                            }
                                            _ => {
                                                self.lexer.next();
                                            }
                                        }
                                    }
                                }
                                Token::Space | Token::Tab | Token::Newline | Token::Semicolon => {
                                    break;
                                }
                                _ => break,
                            }
                        }
                        let value = value_parts.join("");
                        env_vars.insert(var_name, value);
                        
                        // Skip whitespace after the environment variable
                        self.skip_whitespace_and_comments();
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
        
        // Parse command name
        let name = if let Some(token) = self.lexer.peek() {
            match token {
                Token::Identifier => {
                    self.get_identifier_text()?
                }
                Token::TestBracket => {
                    self.lexer.next(); // consume the [
                    "[".to_string()
                }
                Token::True => {
                    self.lexer.next(); // consume true
                    "true".to_string()
                }
                Token::False => {
                    self.lexer.next(); // consume false
                    "false".to_string()
                }
                Token::Dollar | Token::DollarBrace | Token::DollarParen => {
                    self.parse_variable_expansion()?
                }
                _ => {
                    return Err(ParserError::UnexpectedToken(token.clone()));
                }
            }
        } else {
            return Err(ParserError::UnexpectedEOF);
        };
        
        // Skip whitespace before parsing arguments
        self.skip_whitespace_and_comments();
        
        // Parse arguments and redirects
        while let Some(token) = self.lexer.peek() {
            match token {
                Token::Identifier | Token::Number | Token::DoubleQuotedString | Token::SingleQuotedString | Token::CurrentDir => {
                    args.push(self.parse_word()?);
                }
                Token::Dollar | Token::DollarBrace | Token::DollarParen => {
                    args.push(self.parse_variable_expansion()?);
                }
                Token::Assign => {
                    // Handle assignments like =value
                    self.lexer.next(); // consume =
                    if let Some(next_token) = self.lexer.peek() {
                        match next_token {
                            Token::Arithmetic => {
                                // Handle arithmetic expressions like $((i + 1))
                                self.lexer.next(); // consume the Arithmetic token
                                let mut value = "$(".to_string();
                                // Skip until we find the closing parentheses
                                while let Some(token) = self.lexer.peek() {
                                    match token {
                                        Token::ParenClose => {
                                            self.lexer.next();
                                            value.push(')');
                                            // Check if there's another closing parenthesis
                                            if let Some(Token::ParenClose) = self.lexer.peek() {
                                                self.lexer.next();
                                                value.push(')');
                                            }
                                            break;
                                        }
                                        _ => {
                                            self.lexer.next();
                                        }
                                    }
                                }
                                args.push(format!("={}", value));
                            }
                            _ => {
                                // Handle other values
                                let value = self.parse_word()?;
                                args.push(format!("={}", value));
                            }
                        }
                    }
                }
                Token::Minus => {
                    // Handle arguments starting with minus (like -la, -v, etc.)
                    let token_clone = token.clone();
                    self.lexer.next(); // consume the minus
                    if let Some(Token::Identifier) = self.lexer.peek() {
                        let arg = format!("-{}", self.get_identifier_text()?);
                        args.push(arg);
                    } else {
                        return Err(ParserError::UnexpectedToken(token_clone));
                    }
                }
                Token::NonZero => {
                    // Handle -n argument
                    self.lexer.next(); // consume the NonZero token
                    if let Some(Token::Identifier) = self.lexer.peek() {
                        let arg = format!("-n{}", self.get_identifier_text()?);
                        args.push(arg);
                    } else {
                        args.push("-n".to_string());
                    }
                }
                Token::Character => {
                    // Handle -c argument
                    self.lexer.next(); // consume the Character token
                    if let Some(Token::Identifier) = self.lexer.peek() {
                        let arg = format!("-c{}", self.get_identifier_text()?);
                        args.push(arg);
                    } else {
                        args.push("-c".to_string());
                    }
                }
                Token::File => {
                    // Handle -f argument
                    self.lexer.next(); // consume the File token
                    args.push("-f".to_string());
                }
                Token::Directory => {
                    // Handle -d argument
                    self.lexer.next(); // consume the Directory token
                    args.push("-d".to_string());
                }
                Token::Exists => {
                    // Handle -e argument
                    self.lexer.next(); // consume the Exists token
                    args.push("-e".to_string());
                }
                Token::Readable => {
                    // Handle -r argument
                    self.lexer.next(); // consume the Readable token
                    args.push("-r".to_string());
                }
                Token::Writable => {
                    // Handle -w argument
                    self.lexer.next(); // consume the Writable token
                    args.push("-w".to_string());
                }
                Token::Executable => {
                    // Handle -x argument
                    self.lexer.next(); // consume the Executable token
                    args.push("-x".to_string());
                }
                Token::Lt => {
                    // Handle -lt argument
                    self.lexer.next(); // consume the Lt token
                    args.push("-lt".to_string());
                }
                Token::Le => {
                    // Handle -le argument
                    self.lexer.next(); // consume the Le token
                    args.push("-le".to_string());
                }
                Token::Gt => {
                    // Handle -gt argument
                    self.lexer.next(); // consume the Gt token
                    args.push("-gt".to_string());
                }
                Token::Ge => {
                    // Handle -ge argument
                    self.lexer.next(); // consume the Ge token
                    args.push("-ge".to_string());
                }
                Token::Eq => {
                    // Handle -eq argument
                    self.lexer.next(); // consume the Eq token
                    args.push("-eq".to_string());
                }
                Token::Ne => {
                    // Handle -ne argument
                    self.lexer.next(); // consume the Ne token
                    args.push("-ne".to_string());
                }
                Token::Arithmetic => {
                    // Handle arithmetic expressions like $((i + 1))
                    self.lexer.next(); // consume the Arithmetic token
                    // For now, just add the arithmetic expression as a string
                    args.push("$((...))".to_string());
                    // Skip until we find the closing parentheses
                    while let Some(token) = self.lexer.peek() {
                        match token {
                            Token::ParenClose => {
                                self.lexer.next();
                                break;
                            }
                            _ => {
                                self.lexer.next();
                            }
                        }
                    }
                }
                Token::RedirectIn | Token::RedirectOut | Token::RedirectAppend |
                Token::RedirectInOut | Token::Heredoc | Token::HeredocTabs => {
                    redirects.push(self.parse_redirect()?);
                }
                Token::Newline | Token::Semicolon | Token::Done => {
                    // Stop parsing arguments when we hit a command separator
                    break;
                }
                Token::TestBracketClose => {
                    // Handle closing bracket for test commands
                    self.lexer.next(); // consume the ]
                    args.push("]".to_string());
                    break;
                }
                Token::Space | Token::Tab => {
                    // Skip whitespace but continue parsing arguments
                    self.lexer.next();
                }
                _ => break,
            }
        }
        
        Ok(Command::Simple(SimpleCommand {
            name,
            args,
            redirects,
            env_vars,
        }))
    }

    fn parse_if_statement(&mut self) -> Result<Command, ParserError> {
        self.lexer.consume(Token::If)?;
        
        // Skip whitespace
        self.skip_whitespace_and_comments();
        
        // Parse condition - for now, just parse as a simple command
        let condition = Box::new(self.parse_simple_command()?);
        
        // Consume semicolon if present (optional in some shell syntax)
        if let Some(Token::Semicolon) = self.lexer.peek() {
            self.lexer.next();
        }
        
        // Skip whitespace before then
        self.skip_whitespace_and_comments();
        
        self.lexer.consume(Token::Then)?;
        let then_branch = Box::new(self.parse_command()?);
        
        // Skip whitespace and comments before checking for semicolon
        self.skip_whitespace_and_comments();
        
        // Consume semicolon if present after then branch
        if let Some(Token::Semicolon) = self.lexer.peek() {
            self.lexer.next();
            self.skip_whitespace_and_comments();
        } else {
            // If no semicolon, check if we're at the end or if there's an else
            if let Some(Token::Else) = self.lexer.peek() {
                // No semicolon needed before else
            } else if let Some(Token::Fi) = self.lexer.peek() {
                // No semicolon needed before fi
            } else {
                // Expect a semicolon
                return Err(ParserError::ExpectedToken(Token::Semicolon));
            }
        }
        
        let else_branch = if let Some(Token::Else) = self.lexer.peek() {
            self.lexer.next();
            Some(Box::new(self.parse_command()?))
        } else {
            None
        };
        
        // Skip whitespace before fi
        self.skip_whitespace_and_comments();
        
        self.lexer.consume(Token::Fi)?;
        
        Ok(Command::If(IfStatement {
            condition,
            then_branch,
            else_branch,
        }))
    }

    fn parse_while_loop(&mut self) -> Result<Command, ParserError> {
        self.lexer.consume(Token::While)?;
        
        // Skip whitespace after 'while'
        self.skip_whitespace_and_comments();
        
        // Parse condition as a simple command
        let condition = Box::new(self.parse_simple_command()?);
        
        // Skip whitespace before 'do'
        self.skip_whitespace_and_comments();
        
        // Consume semicolon if present
        if let Some(Token::Semicolon) = self.lexer.peek() {
            self.lexer.next();
            self.skip_whitespace_and_comments();
        }
        
        self.lexer.consume(Token::Do)?;
        
        // Skip whitespace after 'do'
        self.skip_whitespace_and_comments();
        
        // Parse the body - parse until we hit 'done'
        let body = Box::new(self.parse_until_done()?);
        
        Ok(Command::While(WhileLoop { condition, body }))
    }

    fn parse_for_loop(&mut self) -> Result<Command, ParserError> {
        self.lexer.consume(Token::For)?;
        
        // Skip whitespace after 'for'
        self.skip_whitespace_and_comments();
        
        let variable = self.get_identifier_text()?;
        
        // Skip whitespace after variable
        self.skip_whitespace_and_comments();
        
        let items = if let Some(Token::In) = self.lexer.peek() {
            self.lexer.next();
            self.skip_whitespace_and_comments();
            self.parse_word_list()?
        } else {
            Vec::new()
        };
        
        // Skip whitespace after word list
        self.skip_whitespace_and_comments();
        
        // Consume semicolon if present
        if let Some(Token::Semicolon) = self.lexer.peek() {
            self.lexer.next();
            self.skip_whitespace_and_comments();
        }
        
        // Skip whitespace before 'do'
        self.skip_whitespace_and_comments();
        
        // Check if we're at 'do'
        if let Some(Token::Do) = self.lexer.peek() {
            self.lexer.next();
        } else {
            return Err(ParserError::ExpectedToken(Token::Do));
        }
        
        // Skip whitespace after 'do'
        self.skip_whitespace_and_comments();
        
        // Parse the body - parse until we hit 'done'
        let body = Box::new(self.parse_until_done()?);
        
        Ok(Command::For(ForLoop {
            variable,
            items,
            body,
        }))
    }

    fn parse_function(&mut self) -> Result<Command, ParserError> {
        self.lexer.consume(Token::Function)?;
        
        let name = self.get_identifier_text()?;
        
        let body = Box::new(self.parse_command()?);
        
        Ok(Command::Function(Function { name, body }))
    }

    fn parse_subshell(&mut self) -> Result<Command, ParserError> {
        self.lexer.consume(Token::ParenOpen)?;
        
        let command = Box::new(self.parse_command()?);
        
        self.lexer.consume(Token::ParenClose)?;
        
        Ok(Command::Subshell(command))
    }

    fn parse_redirect(&mut self) -> Result<Redirect, ParserError> {
        let fd = if let Some(Token::Number) = self.lexer.peek() {
            let fd_str = self.get_number_text()?;
            Some(fd_str.parse().unwrap_or(0))
        } else {
            None
        };
        
        let operator = match self.lexer.next() {
            Some(Token::RedirectIn) => RedirectOperator::Input,
            Some(Token::RedirectOut) => RedirectOperator::Output,
            Some(Token::RedirectAppend) => RedirectOperator::Append,
            Some(Token::RedirectInOut) => RedirectOperator::InputOutput,
            Some(Token::Heredoc) => RedirectOperator::Heredoc,
            Some(Token::HeredocTabs) => RedirectOperator::HeredocTabs,
            _ => return Err(ParserError::InvalidSyntax("Invalid redirect operator".to_string())),
        };
        
        let target = self.parse_word()?;
        
        Ok(Redirect {
            fd,
            operator,
            target,
        })
    }

    fn parse_word(&mut self) -> Result<String, ParserError> {
        let result = match self.lexer.peek() {
            Some(Token::Identifier) => Ok(self.get_identifier_text()?),
            Some(Token::Number) => Ok(self.get_number_text()?),
            Some(Token::DoubleQuotedString) => Ok(self.get_string_text()?),
            Some(Token::SingleQuotedString) => Ok(self.get_string_text()?),
            Some(Token::Dollar) => Ok(self.parse_variable_expansion()?),
            Some(Token::DollarBrace) => Ok(self.parse_variable_expansion()?),
            Some(Token::DollarParen) => Ok(self.parse_variable_expansion()?),
            Some(Token::BraceExpansion) => Ok(self.get_brace_expansion_text()?),
            Some(Token::CurrentDir) => Ok(self.get_current_dir_text()?),
            _ => Err(ParserError::UnexpectedToken(Token::Identifier)),
        };
        
        result
    }

    fn parse_variable_expansion(&mut self) -> Result<String, ParserError> {
        // This is a simplified version - in a real implementation,
        // you'd need to handle the full complexity of shell variable expansion
        let mut result = String::new();
        
        match self.lexer.peek() {
            Some(Token::Dollar) => {
                self.lexer.next();
                if let Some(Token::Identifier) = self.lexer.peek() {
                    result.push_str(&format!("${}", self.get_identifier_text()?));
                }
            }
            Some(Token::DollarBrace) => {
                self.lexer.next();
                if let Some(Token::Identifier) = self.lexer.peek() {
                    result.push_str(&format!("${{{}}}", self.get_identifier_text()?));
                    self.lexer.consume(Token::BraceClose)?;
                }
            }
            Some(Token::DollarParen) => {
                self.lexer.next();
                // Parse command substitution
                result.push_str("$(");
                // This would need more complex parsing
                self.lexer.consume(Token::ParenClose)?;
                result.push(')');
            }
            _ => return Err(ParserError::UnexpectedToken(Token::Identifier)),
        }
        
        Ok(result)
    }

    fn parse_word_list(&mut self) -> Result<Vec<String>, ParserError> {
        let mut words = Vec::new();
        
        while let Some(token) = self.lexer.peek() {
            match token {
                Token::Identifier | Token::Number | Token::DoubleQuotedString |
                Token::SingleQuotedString | Token::Dollar | Token::DollarBrace |
                Token::DollarParen | Token::BraceExpansion => {
                    words.push(self.parse_word()?);
                    // Skip whitespace after each word
                    self.skip_whitespace_and_comments();
                }
                Token::Semicolon | Token::Newline => {
                    // Stop at semicolon or newline, but don't consume it
                    break;
                }
                Token::Space | Token::Tab => {
                    // Skip whitespace
                    self.lexer.next();
                }
                _ => break,
            }
        }
        
        Ok(words)
    }

    fn parse_until_done(&mut self) -> Result<Command, ParserError> {
        let mut commands = Vec::new();
        
        while let Some(token) = self.lexer.peek() {
            match token {
                Token::Done => {
                    // Found 'done', consume it and stop
                    self.lexer.next();
                    break;
                }
                Token::Semicolon => {
                    // Skip semicolon
                    self.lexer.next();
                    self.skip_whitespace_and_comments();
                }
                Token::Newline => {
                    // Skip newline
                    self.lexer.next();
                    self.skip_whitespace_and_comments();
                }
                _ => {
                    // Parse a simple command
                    let command = self.parse_simple_command()?;
                    commands.push(command);
                    
                    // Skip whitespace and comments
                    self.skip_whitespace_and_comments();
                }
            }
        }
        
        if commands.is_empty() {
            return Err(ParserError::InvalidSyntax("Empty for loop body".to_string()));
        }
        
        // For now, just return the first command
        Ok(commands.remove(0))
    }

    fn get_identifier_text(&mut self) -> Result<String, ParserError> {
        if let Some(span) = self.lexer.get_span() {
            let text = self.lexer.get_text(span.0, span.1);
            self.lexer.next(); // consume the identifier
            Ok(text)
        } else {
            Err(ParserError::UnexpectedEOF)
        }
    }

    fn get_number_text(&mut self) -> Result<String, ParserError> {
        if let Some(span) = self.lexer.get_span() {
            let text = self.lexer.get_text(span.0, span.1);
            self.lexer.next(); // consume the number
            Ok(text)
        } else {
            Err(ParserError::UnexpectedEOF)
        }
    }

    fn get_string_text(&mut self) -> Result<String, ParserError> {
        if let Some(span) = self.lexer.get_span() {
            let text = self.lexer.get_text(span.0, span.1);
            self.lexer.next(); // consume the string
            // Strip the quotes from the string
            if text.len() >= 2 {
                Ok(text[1..text.len()-1].to_string())
            } else {
                Ok(text)
            }
        } else {
            Err(ParserError::UnexpectedEOF)
        }
    }

    fn get_brace_expansion_text(&mut self) -> Result<String, ParserError> {
        if let Some(span) = self.lexer.get_span() {
            let text = self.lexer.get_text(span.0, span.1);
            self.lexer.next(); // consume the brace expansion
            Ok(text)
        } else {
            Err(ParserError::UnexpectedEOF)
        }
    }

    fn get_current_dir_text(&mut self) -> Result<String, ParserError> {
        if let Some(span) = self.lexer.get_span() {
            let text = self.lexer.get_text(span.0, span.1);
            self.lexer.next(); // consume the current dir
            Ok(text)
        } else {
            Err(ParserError::UnexpectedEOF)
        }
    }

    fn skip_whitespace_and_comments(&mut self) {
        while let Some(token) = self.lexer.peek() {
            match token {
                Token::Space | Token::Tab | Token::Comment | Token::Newline => {
                    self.lexer.next();
                }
                _ => break,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_command() {
        let input = "echo hello world";
        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_pipeline() {
        let input = "ls | grep test";
        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_if_statement() {
        let input = "if true; then echo hello; fi";
        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_ok());
    }
} 