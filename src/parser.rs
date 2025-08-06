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
        
        // Parse environment variables
        while let Some(token) = self.lexer.peek() {
            match token {
                Token::Identifier => {
                    if let Some(Token::Assign) = self.lexer.peek_n(1) {
                        let var_name = self.get_identifier_text()?;
                        self.lexer.next(); // consume =
                        let value = self.parse_word()?;
                        env_vars.insert(var_name, value);
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
                Token::Identifier | Token::Number | Token::DoubleQuotedString | Token::SingleQuotedString => {
                    args.push(self.parse_word()?);
                }
                Token::Dollar | Token::DollarBrace | Token::DollarParen => {
                    args.push(self.parse_variable_expansion()?);
                }
                Token::RedirectIn | Token::RedirectOut | Token::RedirectAppend |
                Token::RedirectInOut | Token::Heredoc | Token::HeredocTabs => {
                    redirects.push(self.parse_redirect()?);
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
        
        // Consume semicolon if present after then branch
        if let Some(Token::Semicolon) = self.lexer.peek() {
            self.lexer.next();
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
        
        let condition = Box::new(self.parse_command()?);
        
        self.lexer.consume(Token::Do)?;
        let body = Box::new(self.parse_command()?);
        
        self.lexer.consume(Token::Done)?;
        
        Ok(Command::While(WhileLoop { condition, body }))
    }

    fn parse_for_loop(&mut self) -> Result<Command, ParserError> {
        self.lexer.consume(Token::For)?;
        
        let variable = self.get_identifier_text()?;
        
        let items = if let Some(Token::In) = self.lexer.peek() {
            self.lexer.next();
            self.parse_word_list()?
        } else {
            Vec::new()
        };
        
        self.lexer.consume(Token::Do)?;
        let body = Box::new(self.parse_command()?);
        self.lexer.consume(Token::Done)?;
        
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
            _ => Err(ParserError::UnexpectedToken(Token::Identifier)),
        };
        
        // Skip whitespace after consuming the word
        self.skip_whitespace_and_comments();
        
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
                Token::DollarParen => {
                    words.push(self.parse_word()?);
                }
                _ => break,
            }
        }
        
        Ok(words)
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

    fn skip_whitespace_and_comments(&mut self) {
        while let Some(token) = self.lexer.peek() {
            match token {
                Token::Space | Token::Tab | Token::Newline | Token::Comment => {
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