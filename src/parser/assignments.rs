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
    let mut loop_count = 0;
    
    println!("DEBUG: Starting parse_array_elements");
    
    loop {
        loop_count += 1;
        if loop_count > 100 {
            return Err(ParserError::InvalidSyntax("Array parsing loop limit exceeded".to_string()));
        }
        
        let token = lexer.peek();
        println!("DEBUG: Loop {}: token = {:?}, current_element = '{}'", loop_count, token, current_element);
        
        match token {
            None => {
                // End of tokens reached, break out of the loop
                println!("DEBUG: End of tokens reached, breaking");
                break;
            }
            Some(Token::ParenClose) => {
                println!("DEBUG: Found closing parenthesis, adding current_element: '{}'", current_element);
                if !current_element.is_empty() {
                    elements.push(current_element.trim().to_string());
                }
                lexer.next(); // consume )
                break;
            }
            Some(Token::Space) | Some(Token::Tab) | Some(Token::Newline) => {
                println!("DEBUG: Found whitespace/newline, adding current_element: '{}'", current_element);
                if !current_element.is_empty() {
                    elements.push(current_element.trim().to_string());
                    current_element.clear();
                }
                lexer.next(); // consume whitespace
            }
            Some(Token::Identifier) | Some(Token::Number) => {
                let text = lexer.get_current_text().unwrap_or_default();
                println!("DEBUG: Found identifier/number: '{}'", text);
                current_element.push_str(&text);
                lexer.next(); // consume the token
            }
            Some(Token::DoubleQuotedString) | Some(Token::SingleQuotedString) => {
                let text = lexer.get_string_text()?;
                println!("DEBUG: Found string: '{}'", text);
                current_element.push_str(&text);
                lexer.next(); // consume the string token
            }
            Some(Token::Dollar) => {
                // For now, just consume the $ and treat it as part of the element
                current_element.push('$');
                lexer.next(); // consume the $ token
                // If there's an identifier after $, include it
                if let Some(Token::Identifier) = lexer.peek() {
                    let text = lexer.get_current_text().unwrap_or_default();
                    current_element.push_str(&text);
                    lexer.next(); // consume the identifier
                }
            }
            _ => {
                // For any other token, get its text and advance
                if let Some(text) = lexer.get_current_text() {
                    println!("DEBUG: Found other token: '{}'", text);
                    current_element.push_str(&text);
                }
                lexer.next(); // consume the token
            }
        }
    }
    
    println!("DEBUG: Final elements: {:?}", elements);
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
fn parse_arithmetic_expression(lexer: &mut Lexer) -> Result<Word, ParserError> {
    // Handle arithmetic expressions like $((i + 1))
    // First, consume the opening $(( or $( token
    match lexer.peek() {
        Some(Token::Arithmetic) | Some(Token::ArithmeticEval) => {
            lexer.next(); // consume $(( or $(
        }
        _ => {
            return Err(ParserError::InvalidSyntax("Expected arithmetic expression start".to_string()));
        }
    }
    
    let mut expression_parts = Vec::new();
    
    // Simple case: just parse until we find the closing ))
    loop {
        match lexer.peek() {
            Some(Token::ArithmeticEvalClose) => {
                // Found the closing )), consume it and break
                lexer.next();
                break;
            }
            Some(Token::Identifier) => {
                let var_name = lexer.get_identifier_text()?;
                expression_parts.push(var_name);
                lexer.next(); // consume the identifier token
            }
            Some(Token::Number) => {
                let num_text = lexer.get_number_text()?;
                expression_parts.push(num_text);
                lexer.next(); // consume the number token
            }
            Some(Token::Plus) => {
                // Plus operator
                lexer.next();
                expression_parts.push("+".to_string());
            }
            Some(Token::Minus) => {
                // Minus operator
                lexer.next();
                expression_parts.push("-".to_string());
            }
            Some(Token::Star) => {
                // Multiplication operator
                lexer.next();
                expression_parts.push("*".to_string());
            }
            Some(Token::Slash) => {
                // Division operator
                lexer.next();
                expression_parts.push("/".to_string());
            }
            Some(Token::Space) | Some(Token::Tab) => {
                // Skip whitespace
                lexer.next();
            }
            Some(Token::Dollar) => {
                // Handle variable references like $i
                lexer.next();
                if let Some(Token::Identifier) = lexer.peek() {
                    let var_name = lexer.get_identifier_text()?;
                    expression_parts.push(format!("${}", var_name));
                } else {
                    return Err(ParserError::InvalidSyntax("Expected identifier after $ in arithmetic expression".to_string()));
                }
            }
            None => {
                return Err(ParserError::InvalidSyntax("Unexpected end of input in arithmetic expression".to_string()));
            }
            _ => {
                // For any other token, just consume it and add its text
                if let Some(text) = lexer.get_current_text() {
                    expression_parts.push(text);
                    lexer.next();
                } else {
                    break;
                }
            }
        }
    }
    
    let expression = expression_parts.join("");
    
    // Return as an Arithmetic Word variant
    Ok(Word::Arithmetic(ArithmeticExpression {
        expression,
        tokens: vec![], // We don't need to store individual tokens for now
    }))
}

fn parse_variable_expansion(_lexer: &mut Lexer) -> Result<Word, ParserError> {
    // TODO: Implement variable expansion parsing
    Err(ParserError::InvalidSyntax("Variable expansion not yet implemented".to_string()))
}

