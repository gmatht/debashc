use crate::ast::*;
use crate::lexer::{Lexer, Token};
use crate::parser::errors::ParserError;
use crate::parser::utilities::ParserUtilities;
use crate::parser::words::parse_word;

pub fn parse_environment_variable_value(lexer: &mut Lexer) -> Result<Word, ParserError> {
    if let Some(tok) = lexer.peek() {
        match tok {
            Token::Arithmetic | Token::ArithmeticEval => {
                // Parse arithmetic expression properly
                parse_arithmetic_expression(lexer)
            }
            Token::DollarParen => {
                // Parse variable expansion
                parse_variable_expansion(lexer)
            }
            Token::ParenOpen => {
                // Parse parenthetical text as a literal
                let text = lexer.capture_parenthetical_text()?;
                Ok(Word::Literal(text))
            }
            Token::DoubleQuotedString | Token::SingleQuotedString => {
                // Parse quoted string as a literal
                let text = lexer.get_string_text()?;
                Ok(Word::Literal(text))
            }
            Token::BacktickString => {
                // Parse backtick string as a literal
                let text = lexer.get_raw_token_text()?;
                Ok(Word::Literal(text))
            }
            _ => {
                // Parse as a literal string until separator
                let mut value = String::new();
                loop {
                    match lexer.peek() {
                        Some(Token::Space) | Some(Token::Tab) | Some(Token::Newline) | Some(Token::Semicolon) | None => break,
                        Some(Token::Arithmetic) | Some(Token::ArithmeticEval) => {
                            // Parse arithmetic expression properly
                            return parse_arithmetic_expression(lexer);
                        }
                        Some(Token::DollarParen) => {
                            // Parse variable expansion
                            return parse_variable_expansion(lexer);
                        }
                        Some(Token::ParenOpen) => {
                            // Parse parenthetical text as a literal
                            let text = lexer.capture_parenthetical_text()?;
                            value.push_str(&text);
                        }
                        _ => {
                            if let Some((start, end)) = lexer.get_span() {
                                value.push_str(&lexer.get_text(start, end));
                                lexer.next();
                            } else {
                                break;
                            }
                        }
                    }
                }
                Ok(Word::Literal(value))
            }
        }
    } else {
        Ok(Word::Literal(String::new()))
    }
}

pub fn parse_array_elements(lexer: &mut Lexer) -> Result<Vec<String>, ParserError> {
    let mut elements = Vec::new();
    let mut current_element = String::new();
    
    loop {
        match lexer.peek() {
            Some(Token::ParenClose) => {
                if !current_element.is_empty() {
                    elements.push(current_element.trim().to_string());
                }
                lexer.next(); // consume )
                break;
            }
            Some(Token::Space) | Some(Token::Tab) | Some(Token::Newline) => {
                if !current_element.is_empty() {
                    elements.push(current_element.trim().to_string());
                    current_element.clear();
                }
                lexer.next(); // consume whitespace
            }
            Some(Token::Identifier) | Some(Token::Number) => {
                let text = lexer.get_current_text().unwrap_or_default();
                current_element.push_str(&text);
                lexer.next();
            }
            Some(Token::DoubleQuotedString) | Some(Token::SingleQuotedString) => {
                let text = lexer.get_string_text()?;
                current_element.push_str(&text);
            }
            Some(Token::Dollar) => {
                let var_expansion = parse_variable_expansion(lexer)?;
                current_element.push_str(&var_expansion.to_string());
            }
            _ => {
                let text = lexer.get_current_text().unwrap_or_default();
                current_element.push_str(&text);
                lexer.next();
            }
        }
    }
    
    Ok(elements)
}

pub fn parse_word_list(lexer: &mut Lexer) -> Result<Vec<Word>, ParserError> {
    let mut words = Vec::new();
    
    loop {
        // Skip whitespace and comments
        lexer.skip_whitespace_and_comments();
        
        // Check for end of list
        if lexer.is_eof() || matches!(lexer.peek(), Some(Token::Semicolon | Token::Newline | Token::CarriageReturn | Token::Done | Token::Fi | Token::Then | Token::Else | Token::ParenClose | Token::BraceClose)) {
            break;
        }
        
        // Parse the next word
        let word = parse_word(lexer)?;
        words.push(word);
        
        // Skip whitespace after the word
        lexer.skip_whitespace_and_comments();
    }
    
    Ok(words)
}

// Placeholder functions - these would need to be implemented based on the actual AST structures
fn parse_arithmetic_expression(_lexer: &mut Lexer) -> Result<Word, ParserError> {
    // TODO: Implement arithmetic expression parsing
    Err(ParserError::InvalidSyntax("Arithmetic expressions not yet implemented".to_string()))
}

fn parse_variable_expansion(_lexer: &mut Lexer) -> Result<Word, ParserError> {
    // TODO: Implement variable expansion parsing
    Err(ParserError::InvalidSyntax("Variable expansion not yet implemented".to_string()))
}

