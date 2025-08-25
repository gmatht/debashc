use crate::lexer::{Lexer, Token};
use crate::parser::errors::ParserError;

pub trait ParserUtilities {
    fn skip_whitespace_and_comments(&mut self);
    fn skip_inline_whitespace_and_comments(&mut self);
    fn capture_parenthetical_text(&mut self) -> Result<String, ParserError>;
    fn capture_double_bracket_expression(&mut self) -> Result<String, ParserError>;
    fn capture_single_bracket_expression(&mut self) -> Result<String, ParserError>;
    fn get_identifier_text(&mut self) -> Result<String, ParserError>;
    fn get_number_text(&mut self) -> Result<String, ParserError>;
    fn get_raw_token_text(&mut self) -> Result<String, ParserError>;
    fn get_string_text(&mut self) -> Result<String, ParserError>;
    fn get_current_text(&mut self) -> Option<String>;
    fn get_text(&mut self, start: usize, end: usize) -> String;
    fn get_span(&mut self) -> Option<(usize, usize)>;
    fn current_position(&mut self) -> usize;
    fn offset_to_line_col(&mut self, offset: usize) -> (usize, usize);
    fn peek(&mut self) -> Option<Token>;
    fn peek_n(&mut self, n: usize) -> Option<Token>;
    fn next(&mut self) -> Option<Token>;
    fn is_eof(&mut self) -> bool;
    fn consume(&mut self, expected: Token) -> Result<(), ParserError>;
}

impl ParserUtilities for Lexer {
    fn get_text(&mut self, start: usize, end: usize) -> String {
        self.input[start..end].to_string()
    }

    fn get_span(&mut self) -> Option<(usize, usize)> {
        self.tokens.get(self.current).map(|(_, start, end)| (*start, *end))
    }

    fn skip_whitespace_and_comments(&mut self) {
        while let Some(token) = self.peek() {
            match token {
                Token::Space | Token::Tab | Token::Comment | Token::Newline => {
                    self.next();
                }
                _ => break,
            }
        }
    }

    fn skip_inline_whitespace_and_comments(&mut self) {
        while let Some(token) = self.peek() {
            match token {
                Token::Space | Token::Tab | Token::Comment => {
                    self.next();
                }
                _ => break,
            }
        }
    }

    fn capture_parenthetical_text(&mut self) -> Result<String, ParserError> {
        let mut content = String::new();
        let mut depth = 1;
        
        // Consume the opening parenthesis
        self.next();
        
        while depth > 0 {
            match self.peek() {
                Some(Token::ParenOpen) => {
                    depth += 1;
                    content.push('(');
                    self.next();
                }
                Some(Token::ParenClose) => {
                    depth -= 1;
                    if depth > 0 {
                        content.push(')');
                    }
                    self.next();
                }
                Some(_) => {
                    if let Some(text) = self.get_current_text() {
                        content.push_str(&text);
                    }
                    self.next();
                }
                None => return Err(ParserError::UnexpectedEOF),
            }
        }
        
        Ok(content)
    }

    fn capture_double_bracket_expression(&mut self) -> Result<String, ParserError> {
        let mut content = String::new();
        let mut depth = 2; // Start with depth 2 for [[
        
        // Consume the first two [
        self.next(); // consume first [
        self.next(); // consume second [
        
        while depth > 0 {
            match self.peek() {
                Some(Token::TestBracket) => {
                    depth += 1;
                    content.push('[');
                    self.next();
                }
                Some(Token::TestBracketClose) => {
                    depth -= 1;
                    if depth > 0 {
                        content.push(']');
                    }
                    self.next();
                }
                Some(_) => {
                    if let Some(text) = self.get_current_text() {
                        content.push_str(&text);
                    }
                    self.next();
                }
                None => return Err(ParserError::UnexpectedEOF),
            }
        }
        
        Ok(content)
    }

    fn capture_single_bracket_expression(&mut self) -> Result<String, ParserError> {
        let mut content = String::new();
        let mut depth = 1; // Start with depth 1 for [
        
        // Consume the opening [
        self.next();
        
        while depth > 0 {
            match self.peek() {
                Some(Token::TestBracket) => {
                    depth += 1;
                    content.push('[');
                    self.next();
                }
                Some(Token::TestBracketClose) => {
                    depth -= 1;
                    if depth > 0 {
                        content.push(']');
                    }
                    self.next();
                }
                Some(_) => {
                    if let Some(text) = self.get_current_text() {
                        content.push_str(&text);
                    }
                    self.next();
                }
                None => return Err(ParserError::UnexpectedEOF),
            }
        }
        
        Ok(content)
    }

    fn get_identifier_text(&mut self) -> Result<String, ParserError> {
        if let Some(Token::Identifier) = self.peek() {
            if let Some(text) = self.get_current_text() {
                self.next();
                Ok(text)
            } else {
                Err(ParserError::InvalidSyntax("Failed to get identifier text".to_string()))
            }
        } else {
            Err(ParserError::InvalidSyntax("Expected identifier".to_string()))
        }
    }

    fn get_number_text(&mut self) -> Result<String, ParserError> {
        if let Some(Token::Number) = self.peek() {
            if let Some(text) = self.get_current_text() {
                self.next();
                Ok(text)
            } else {
                Err(ParserError::InvalidSyntax("Failed to get number text".to_string()))
            }
        } else {
            Err(ParserError::InvalidSyntax("Expected number".to_string()))
        }
    }

    fn get_raw_token_text(&mut self) -> Result<String, ParserError> {
        if let Some(text) = self.get_current_text() {
            self.next();
            Ok(text)
        } else {
            Err(ParserError::InvalidSyntax("Failed to get token text".to_string()))
        }
    }

    fn get_string_text(&mut self) -> Result<String, ParserError> {
        if let Some((start, end)) = self.get_span() {
            let text = self.get_text(start, end);
            self.next();
            Ok(text)
        } else {
            Err(ParserError::InvalidSyntax("Failed to get string text".to_string()))
        }
    }

    fn get_current_text(&mut self) -> Option<String> {
        if let Some((start, end)) = self.get_span() {
            Some(self.get_text(start, end))
        } else {
            None
        }
    }



    fn current_position(&mut self) -> usize {
        self.current
    }

    fn offset_to_line_col(&mut self, offset: usize) -> (usize, usize) {
        self.offset_to_line_col(offset)
    }

    fn peek(&mut self) -> Option<Token> {
        self.tokens.get(self.current).map(|(token, _, _)| token.clone())
    }

    fn peek_n(&mut self, n: usize) -> Option<Token> {
        self.tokens.get(self.current + n).map(|(token, _, _)| token.clone())
    }

    fn next(&mut self) -> Option<Token> {
        if self.current < self.tokens.len() {
            let token = self.tokens[self.current].0.clone();
            self.current += 1;
            Some(token)
        } else {
            None
        }
    }

    fn is_eof(&mut self) -> bool {
        self.current >= self.tokens.len()
    }

    fn consume(&mut self, expected: Token) -> Result<(), ParserError> {
        if let Some(token) = self.peek() {
            if std::mem::discriminant(&token) == std::mem::discriminant(&expected) {
                self.next();
                Ok(())
            } else {
                let (line, col) = self.offset_to_line_col(0);
                Err(ParserError::UnexpectedToken { token, line, col })
            }
        } else {
            Err(ParserError::UnexpectedEOF)
        }
    }
}

