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
    
    eprintln!("DEBUG: parse_heredoc called with delimiter: '{}'", delim);
    let mut body = String::new();
    let mut found_delim = false;
    let mut at_line_start = true;

    // Skip to the next newline token
    while let Some(token) = lexer.peek() {
        match token {
            Token::Newline => {
                eprintln!("DEBUG: Found newline, breaking to start content collection");
                lexer.next(); // consume the newline
                break;
            }
            _ => {
                eprintln!("DEBUG: Skipping token: {:?}", token);
                lexer.next(); // consume other tokens
            }
        }
    }

    // Collect lines until we find the delimiter at start of line
    while let Some(token) = lexer.peek() {
        eprintln!("DEBUG: Processing token: {:?}, at_line_start: {}, pos: {:?}", token, at_line_start, lexer.current_position());
        match token {
            Token::Newline => {
                eprintln!("DEBUG: Found newline in content");
                lexer.next(); // consume the newline
                at_line_start = true;
                body.push('\n');
            }
            Token::Identifier => {
                // This is part of the heredoc content, not a delimiter
                let word = lexer.get_identifier_text()?;
                eprintln!("DEBUG: Adding identifier to body: '{}', at_line_start: {}, delimiter: '{}'", word, at_line_start, delim);
                // Check if this identifier is the delimiter (at start of line)
                if at_line_start && word == delim {
                    eprintln!("DEBUG: Found delimiter at start of line, stopping");
                    found_delim = true;
                    // Consume the delimiter token to prevent it from being parsed as a separate command
                    lexer.next();
                    break;
                }
                // Also check if this is the delimiter at the end of content (fallback)
                if word == delim {
                    eprintln!("DEBUG: Found delimiter at end of content, stopping");
                    found_delim = true;
                    // Consume the delimiter token to prevent it from being parsed as a separate command
                    lexer.next();
                    break;
                }
                // Add newline before this word if we're not at line start and this is a new word
                if !at_line_start {
                    body.push('\n');
                }
                body.push_str(&word);
                at_line_start = false;
                lexer.next();
            }
            Token::Space => {
                // Add spaces to the body
                let text = lexer.get_current_text().unwrap_or_default();
                eprintln!("DEBUG: Adding space token to body: '{}'", text);
                body.push_str(&text);
                at_line_start = false;
                lexer.next();
            }
            Token::Tab => {
                // Add tabs to the body
                let text = lexer.get_current_text().unwrap_or_default();
                eprintln!("DEBUG: Adding tab token to body: '{}'", text);
                body.push_str(&text);
                at_line_start = false;
                lexer.next();
            }
            _ => {
                // For any other token, just consume it and add to body
                let text = lexer.get_current_text().unwrap_or_default();
                eprintln!("DEBUG: Adding other token to body: '{}'", text);
                // Only add space before this token if we're not at line start and the previous token was an identifier
                // This prevents adding spaces between consecutive punctuation tokens
                if !at_line_start && body.ends_with(|c: char| c.is_alphanumeric() || c == '_') {
                    body.push(' ');
                }
                body.push_str(&text);
                at_line_start = false;
                lexer.next();
            }
        }
    }

    eprintln!("DEBUG: Final heredoc body: '{}'", body);
    if found_delim {
        // Ensure the heredoc body ends with a newline
        if !body.ends_with('\n') {
            body.push('\n');
        }
        // Skip any whitespace after the delimiter
        lexer.skip_whitespace_and_comments();
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
