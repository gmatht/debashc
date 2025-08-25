use crate::ast::*;
use crate::lexer::{Lexer, Token};
use crate::parser::errors::ParserError;
use crate::parser::utilities::ParserUtilities;
use crate::parser::words::parse_word;

pub fn parse_redirect(lexer: &mut Lexer) -> Result<Redirect, ParserError> {
    let fd = if let Some(Token::Number) = lexer.peek() {
        let fd_str = lexer.get_number_text()?;
        Some(fd_str.parse().unwrap_or(0))
    } else {
        None
    };
    
    let operator = match lexer.next() {
        Some(Token::RedirectIn) => RedirectOperator::Input,
        Some(Token::RedirectOut) => RedirectOperator::Output,
        Some(Token::RedirectAppend) => RedirectOperator::Append,
        Some(Token::RedirectInOut) => RedirectOperator::Input, // Use Input as fallback
        Some(Token::Heredoc) => RedirectOperator::Heredoc,
        Some(Token::HeredocTabs) => RedirectOperator::HeredocTabs,
        Some(Token::HereString) => RedirectOperator::HereString,
        Some(Token::RedirectOutErr) => RedirectOperator::Output, // Use Output as fallback
        Some(Token::RedirectInErr) => RedirectOperator::Input, // Use Input as fallback
        Some(Token::RedirectOutClobber) => RedirectOperator::Output, // Use Output as fallback
        Some(Token::RedirectAll) => RedirectOperator::Output, // Use Output as fallback
        Some(Token::RedirectAllAppend) => RedirectOperator::Append, // Use Append as fallback
        _ => return Err(ParserError::InvalidSyntax("Invalid redirect operator".to_string())),
    };
    
    // Here-string: '<<< word' often lexes as '<<' '<' then word; accept optional extra '<'
    if matches!(operator, RedirectOperator::Heredoc) {
        if let Some(Token::RedirectIn) = lexer.peek() { 
            lexer.next(); 
        }
    }
    
    // Skip whitespace before target
    lexer.skip_whitespace_and_comments();

    // Process substitution as redirect target, allowing an optional extra '<' before '('
    // For here-strings, parse the string content as the target
    let target = if matches!(operator, RedirectOperator::HereString) {
        // For here-strings, parse the string content that follows
        parse_word(lexer)?
    } else if matches!(lexer.peek(), Some(Token::RedirectIn)) && matches!(lexer.peek_n(1), Some(Token::ParenOpen)) {
        // consume the extra '<' and capture ( ... )
        lexer.next();
        Word::Literal(lexer.capture_parenthetical_text()?)
    } else if matches!(lexer.peek(), Some(Token::ParenOpen)) {
        Word::Literal(lexer.capture_parenthetical_text()?)
    } else {
        parse_word(lexer)?
    };
    
    // If this is a heredoc, capture lines until the delimiter is found at start of line
    // If this is a here-string, the target is the string content
    let heredoc_body = match operator {
        RedirectOperator::Heredoc | RedirectOperator::HeredocTabs => {
            parse_heredoc(lexer, &target)?
        }
        RedirectOperator::HereString => {
            // For here-strings, the target is the string content
            // We need to extract the string content from the target
            match &target {
                Word::Literal(s) => Some(s.clone()),
                _ => None,
            }
        }
        _ => None,
    };

    Ok(Redirect { fd, operator, target, heredoc_body })
}

fn parse_heredoc(lexer: &mut Lexer, target: &Word) -> Result<Option<String>, ParserError> {
    let delim = match target {
        Word::Literal(s) => s.clone(),
        _ => return Err(ParserError::InvalidSyntax("Heredoc delimiter must be a literal string".to_string())),
    };
    
    let mut body = String::new();
    let mut found_delim = false;

    // Skip to the next newline token
    while let Some(token) = lexer.peek() {
        match token {
            Token::Newline => {
                lexer.next(); // consume the newline
                break;
            }
            _ => {
                lexer.next(); // consume other tokens
            }
        }
    }

    // Collect lines until we find the delimiter at start of line
    while let Some(token) = lexer.peek() {
        match token {
            Token::Newline => {
                lexer.next(); // consume the newline
                // Check if the next token is the delimiter
                if let Some(Token::Identifier) = lexer.peek() {
                    let next_word = lexer.get_current_text().unwrap_or_default();
                    if next_word == delim {
                        found_delim = true;
                        break;
                    }
                }
                body.push('\n');
            }
            Token::Identifier => {
                let word = lexer.get_identifier_text()?;
                if word == delim {
                    found_delim = true;
                    break;
                }
                body.push_str(&word);
            }
            _ => {
                // For any other token, just consume it and add to body
                let text = lexer.get_current_text().unwrap_or_default();
                body.push_str(&text);
                lexer.next();
            }
        }
    }

    if found_delim {
        Ok(Some(body))
    } else {
        Ok(Some(String::new()))
    }
}

pub fn parse_process_substitution(lexer: &mut Lexer, is_input: bool) -> Result<Redirect, ParserError> {
    // Consume the opening < or >
    lexer.next();
    
    // Parse the inner command
    let inner = lexer.capture_parenthetical_text()?;
    
    // Parse the inner command
    let inner_cmd = parse_command_from_text(lexer, &inner)?;
    
    let operator = if is_input {
        RedirectOperator::ProcessSubstitutionInput(Box::new(inner_cmd))
    } else {
        RedirectOperator::ProcessSubstitutionOutput(Box::new(inner_cmd))
    };
    
    Ok(Redirect {
        fd: None,
        operator,
        target: Word::Literal("".to_string()), // Not used for process substitution
        heredoc_body: None,
    })
}

// Placeholder function - this would need to be implemented
fn parse_command_from_text(_lexer: &mut Lexer, _text: &str) -> Result<Command, ParserError> {
    // TODO: Implement command parsing from text
    Err(ParserError::InvalidSyntax("Command parsing from text not yet implemented".to_string()))
}
