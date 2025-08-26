use crate::ast::*;
use crate::lexer::{Lexer, Token};
use crate::parser::errors::ParserError;
use crate::parser::utilities::ParserUtilities;
use std::collections::HashMap;

pub fn parse_word(lexer: &mut Lexer) -> Result<Word, ParserError> {
    // Combine contiguous bare-word tokens (identifiers, numbers, slashes) into a single literal
    if matches!(lexer.peek(), Some(Token::Identifier) | Some(Token::Number) | Some(Token::OctalNumber) | Some(Token::Slash)) {
        let mut combined = String::new();
        loop {
            match lexer.peek() {
                Some(Token::Identifier) | Some(Token::Number) | Some(Token::OctalNumber) | Some(Token::Slash) => {
                    // Append raw token text and consume
                    if let Some(text) = lexer.get_current_text() {
                        combined.push_str(&text);
                        lexer.next();
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
        // Skip inline whitespace after consuming the word
        lexer.skip_inline_whitespace_and_comments();
        return Ok(Word::Literal(combined));
    }

    let result = match lexer.peek() {
        Some(Token::Identifier) => Ok(Word::Literal(lexer.get_identifier_text()?)),
        Some(Token::Number) => Ok(Word::Literal(lexer.get_number_text()?)),
        Some(Token::OctalNumber) => Ok(Word::Literal(lexer.get_raw_token_text()?)),
        Some(Token::DoubleQuotedString) => {
            // Always parse as string interpolation for double-quoted strings
            // This handles both simple strings and strings with variables
            Ok(parse_string_interpolation(lexer)?)
        },
        Some(Token::SingleQuotedString) => Ok(Word::Literal(lexer.get_string_text()?)),
        Some(Token::BacktickString) => Ok(Word::Literal(lexer.get_raw_token_text()?)),
        Some(Token::DollarSingleQuotedString) => Ok(parse_ansic_quoted_string(lexer)?),
        Some(Token::DollarDoubleQuotedString) => Ok(parse_string_interpolation(lexer)?),
        Some(Token::BraceOpen) => Ok(parse_brace_expansion(lexer)?),
        Some(Token::Source) => {
            // Treat standalone 'source' as a normal word (e.g., `source file.sh`)
            lexer.next();
            Ok(Word::Literal("source".to_string()))
        }
        Some(Token::Range) => {
            // Treat standalone '..' as a literal (e.g., `cd ..`)
            lexer.next();
            Ok(Word::Literal("..".to_string()))
        }
        Some(Token::Star) => {
            // Treat standalone '*' as a literal (e.g., `ls *`)
            lexer.next();
            Ok(Word::Literal("*".to_string()))
        }
        Some(Token::Dot) => {
            // Treat standalone '.' as a literal (e.g., `ls .`)
            lexer.next();
            Ok(Word::Literal(".".to_string()))
        }
        Some(Token::CasePattern) => {
            // Treat case statement patterns like *.txt as literals.
            // get_raw_token_text() consumes the current token, so do not call next() here.
            Ok(Word::Literal(lexer.get_raw_token_text()?))
        }
        Some(Token::Slash) => {
            // Treat standalone '/' as a literal (e.g., `cd /`)
            lexer.next();
            Ok(Word::Literal("/".to_string()))
        }
        // Test operators
        Some(Token::File) => {
            lexer.next();
            Ok(Word::Literal("-f".to_string()))
        }
        Some(Token::Directory) => {
            lexer.next();
            Ok(Word::Literal("-d".to_string()))
        }
        Some(Token::Exists) => {
            lexer.next();
            Ok(Word::Literal("-e".to_string()))
        }
        Some(Token::Readable) => {
            lexer.next();
            Ok(Word::Literal("-r".to_string()))
        }
        Some(Token::Writable) => {
            lexer.next();
            Ok(Word::Literal("-w".to_string()))
        }
        Some(Token::Executable) => {
            lexer.next();
            Ok(Word::Literal("-x".to_string()))
        }
        Some(Token::Size) => {
            lexer.next();
            Ok(Word::Literal("-s".to_string()))
        }
        Some(Token::Symlink) => {
            lexer.next();
            Ok(Word::Literal("-L".to_string()))
        }
        Some(Token::TestBracketClose) => {
            lexer.next();
            Ok(Word::Literal("]".to_string()))
        }
        Some(Token::Tilde) => {
            // Treat standalone '~' as a literal (e.g., `cd ~`)
            lexer.next();
            Ok(Word::Literal("~".to_string()))
        }
        Some(Token::LongOption) => {
            // Treat long options like --color=always as literals
            Ok(Word::Literal(lexer.get_raw_token_text()?))
        }
        Some(Token::RegexPattern) => {
            // Treat regex patterns as literals
            Ok(Word::Literal(lexer.get_raw_token_text()?))
        }
        Some(Token::RegexMatch) => {
            // Treat regex match operator as literal
            Ok(Word::Literal(lexer.get_raw_token_text()?))
        }
        Some(Token::NameFlag) | Some(Token::MaxDepthFlag) | Some(Token::TypeFlag) => {
            // Treat command-line flags as literals
            Ok(Word::Literal(lexer.get_raw_token_text()?))
        }
        Some(Token::Dollar) => Ok(parse_variable_expansion(lexer)?),
        Some(Token::DollarBrace) | Some(Token::DollarParen) | Some(Token::DollarHashSimple) | Some(Token::DollarAtSimple) | Some(Token::DollarStarSimple)
        | Some(Token::DollarBraceHash) | Some(Token::DollarBraceBang) | Some(Token::DollarBraceStar) | Some(Token::DollarBraceAt)
        | Some(Token::DollarBraceHashStar) | Some(Token::DollarBraceHashAt) | Some(Token::DollarBraceBangStar) | Some(Token::DollarBraceBangAt)
            => Ok(parse_variable_expansion(lexer)?),
        Some(Token::Arithmetic) | Some(Token::ArithmeticEval) => Ok(parse_arithmetic_expression(lexer)?),
        _ => {
            let (line, col) = lexer.offset_to_line_col(0);
            let token = lexer.peek().unwrap_or(Token::Identifier).to_owned();
            Err(ParserError::UnexpectedToken { token, line, col })
        }
    };
    
    // Skip inline whitespace after consuming the word
    lexer.skip_inline_whitespace_and_comments();
    
    result
}

pub fn parse_variable_expansion(lexer: &mut Lexer) -> Result<Word, ParserError> {
    match lexer.peek() {
        Some(Token::Dollar) => {
            lexer.next();
            if let Some(Token::Identifier) = lexer.peek() {
                let var_name = lexer.get_identifier_text()?;
                Ok(Word::Variable(var_name))
            } else {
                Err(ParserError::InvalidSyntax("Expected identifier after $".to_string()))
            }
        }
        Some(Token::DollarHashSimple) => { 
            lexer.next(); 
            Ok(Word::Variable("#".to_string()))
        }
        Some(Token::DollarAtSimple) => { 
            lexer.next(); 
            Ok(Word::Variable("@".to_string()))
        }
        Some(Token::DollarStarSimple) => { 
            lexer.next(); 
            Ok(Word::Variable("*".to_string()))
        }
        Some(Token::DollarBrace) => {
            // Parse ${...} expansions
            lexer.next(); // consume the token
            
            // Check if this is an array access pattern like ${matrix[$i,$j]}
            if let Some(Token::Identifier) = lexer.peek() {
                let array_name = lexer.get_identifier_text()?;
                
                // Look ahead to see if this is followed by [
                if let Some(Token::TestBracket) = lexer.peek_n(1) {
                    // This is an array access, parse it properly
                    lexer.next(); // consume the identifier
                    lexer.next(); // consume the [
                    
                    // Parse the array index content until we find the closing ]
                    let mut index_content = String::new();
                    let mut bracket_depth = 1;
                    
                    while bracket_depth > 0 {
                        if let Some((start, end)) = lexer.get_span() {
                            let token = lexer.peek();
                            
                            match token {
                                Some(Token::TestBracket) => {
                                    bracket_depth += 1;
                                    let text = lexer.get_text(start, end);
                                    index_content.push_str(&text);
                                    lexer.next();
                                }
                                Some(Token::TestBracketClose) => {
                                    bracket_depth -= 1;
                                    if bracket_depth == 0 {
                                        // Don't consume the closing ] yet, let parse_braced_variable_name handle it
                                        break;
                                    } else {
                                        let text = lexer.get_text(start, end);
                                        index_content.push_str(&text);
                                        lexer.next();
                                    }
                                }
                                _ => {
                                    let text = lexer.get_text(start, end);
                                    index_content.push_str(&text);
                                    lexer.next();
                                }
                            }
                        } else {
                            break;
                        }
                    }
                    
                    // Now parse the rest of the braced expression to get to the closing }
                    let rest_content = parse_braced_variable_name(lexer)?;
                    
                    // Combine the array name, index, and rest content
                    let full_content = format!("{}[{}]{}", array_name, index_content, rest_content);
                    
                    // Check if this is array syntax first
                    if full_content.starts_with('#') && full_content.contains('[') && full_content.contains(']') {
                        // This is ${#arr[@]} - array length
                        if let Some(bracket_start) = full_content.find('[') {
                            if let Some(_bracket_end) = full_content.rfind(']') {
                                let array_name = &full_content[1..bracket_start]; // Remove # prefix
                                return Ok(Word::MapLength(array_name.to_string()));
                            }
                        }
                    } else if full_content.starts_with('!') && full_content.contains('[') && full_content.contains(']') {
                        // This is ${!map[@]} - get keys of associative array
                        if let Some(bracket_start) = full_content.find('[') {
                            if let Some(_bracket_end) = full_content.rfind(']') {
                                let map_name = &full_content[1..bracket_start]; // Remove ! prefix
                                return Ok(Word::MapKeys(map_name.to_string()));
                            }
                        }
                    } else if full_content.contains('[') && full_content.contains(']') {
                        // This is a map/array access like ${map[foo]} or ${arr[1]}
                        if let Some(bracket_start) = full_content.find('[') {
                            if let Some(bracket_end) = full_content.rfind(']') {
                                let map_name = &full_content[..bracket_start];
                                let key = &full_content[bracket_start + 1..bracket_end];
                                
                                // Special case: if key is "@", this is array iteration
                                if key == "@" {
                                    // Check if there's array slicing after the closing brace
                                    // Look ahead for :offset:length syntax
                                    if let Some(Token::Colon) = lexer.peek() {
                                        // This is array slicing like ${arr[@]:start:length}
                                        return parse_array_slicing(lexer, map_name.to_string());
                                    }
                                    return Ok(Word::MapAccess(map_name.to_string(), "@".to_string()));
                                }
                                
                                return Ok(Word::MapAccess(map_name.to_string(), key.to_string()));
                            }
                        }
                    }
                    
                    // Check for parameter expansion operators
                    if full_content.contains(":") {
                        // Handle array slicing syntax like ${var:offset} or ${var:start:length}
                        if let Some(colon_pos) = full_content.find(':') {
                            let var_name = &full_content[..colon_pos];
                            let slice_part = &full_content[colon_pos + 1..];
                            
                            if let Some(second_colon) = slice_part.find(':') {
                                // This is ${var:start:length} syntax
                                let offset = &slice_part[..second_colon];
                                let length = &slice_part[second_colon + 1..];
                                return Ok(Word::ParameterExpansion(ParameterExpansion {
                                    variable: var_name.to_string(),
                                    operator: ParameterExpansionOperator::ArraySlice(offset.to_string(), Some(length.to_string())),
                                }));
                            } else {
                                // This is ${var:offset} syntax
                                return Ok(Word::ParameterExpansion(ParameterExpansion {
                                    variable: var_name.to_string(),
                                    operator: ParameterExpansionOperator::ArraySlice(slice_part.to_string(), None),
                                }));
                            }
                        }
                    }
                    
                    return Ok(Word::Variable(full_content));
                }
            }
            
            // Try to parse as a parameter expansion first
            if let Ok(pe) = parse_parameter_expansion(lexer) {
                Ok(pe)
            } else {
                // Fall back to the old method
                let var_name = parse_braced_variable_name(lexer)?;
                
                // Check if this is array syntax first
                if var_name.starts_with('#') && var_name.contains('[') && var_name.contains(']') {
                    // This is ${#arr[@]} - array length
                    if let Some(bracket_start) = var_name.find('[') {
                        if let Some(_bracket_end) = var_name.rfind(']') {
                            let array_name = &var_name[1..bracket_start]; // Remove # prefix
                            return Ok(Word::MapLength(array_name.to_string()));
                        }
                    }
                } else if var_name.starts_with('!') && var_name.contains('[') && var_name.contains(']') {
                    // This is ${!map[@]} - get keys of associative array
                    if let Some(bracket_start) = var_name.find('[') {
                        if let Some(_bracket_end) = var_name.rfind(']') {
                            let map_name = &var_name[1..bracket_start]; // Remove ! prefix
                            return Ok(Word::MapKeys(map_name.to_string()));
                        }
                    }
                } else if var_name.contains('[') && var_name.contains(']') {
                    // This is a map/array access like ${map[foo]} or ${arr[1]}
                    if let Some(bracket_start) = var_name.find('[') {
                        if let Some(bracket_end) = var_name.rfind(']') {
                            let map_name = &var_name[..bracket_start];
                            let key = &var_name[bracket_start + 1..bracket_end];
                            
                            // Special case: if key is "@", this is array iteration
                            if key == "@" {
                                // Check if there's array slicing after the closing brace
                                // Look ahead for :offset:length syntax
                                if let Some(Token::Colon) = lexer.peek() {
                                    // This is array slicing like ${arr[@]:start:length}
                                    return parse_array_slicing(lexer, map_name.to_string());
                                }
                                return Ok(Word::MapAccess(map_name.to_string(), "@".to_string()));
                            }
                            
                            return Ok(Word::MapAccess(map_name.to_string(), key.to_string()));
                        }
                    }
                }
                
                // Check if this is a parameter expansion with operators
                // Check longer patterns first to avoid partial matches
                if var_name.ends_with("^^") {
                    let base_var = var_name.trim_end_matches("^^");
                    Ok(Word::ParameterExpansion(ParameterExpansion {
                        variable: base_var.to_string(),
                        operator: ParameterExpansionOperator::UppercaseAll,
                    }))
                } else if var_name.ends_with(",,") {
                    let base_var = var_name.trim_end_matches(",,");
                    Ok(Word::ParameterExpansion(ParameterExpansion {
                        variable: base_var.to_string(),
                        operator: ParameterExpansionOperator::LowercaseAll,
                    }))
                } else if var_name.ends_with("^") && !var_name.ends_with("^^") {
                    let base_var = var_name.trim_end_matches("^");
                    Ok(Word::ParameterExpansion(ParameterExpansion {
                        variable: base_var.to_string(),
                        operator: ParameterExpansionOperator::UppercaseFirst,
                    }))
                } else if var_name.ends_with("##*/") {
                    let base_var = var_name.trim_end_matches("##*/");
                    Ok(Word::ParameterExpansion(ParameterExpansion {
                        variable: base_var.to_string(),
                        operator: ParameterExpansionOperator::Basename,
                    }))
                } else if var_name.ends_with("%/*") {
                    let base_var = var_name.trim_end_matches("%/*");
                    Ok(Word::ParameterExpansion(ParameterExpansion {
                        variable: base_var.to_string(),
                        operator: ParameterExpansionOperator::Dirname,
                    }))
                } else if var_name.contains("##") && !var_name.ends_with("##*/") {
                    let parts: Vec<&str> = var_name.split("##").collect();
                    if parts.len() == 2 {
                        Ok(Word::ParameterExpansion(ParameterExpansion {
                            variable: parts[0].to_string(),
                            operator: ParameterExpansionOperator::RemoveLongestPrefix(parts[1].to_string()),
                        }))
                    } else {
                        Ok(Word::Variable(var_name))
                    }
                } else if var_name.contains("%%") && !var_name.ends_with("%/*") {
                    let parts: Vec<&str> = var_name.split("%%").collect();
                    if parts.len() == 2 {
                        Ok(Word::ParameterExpansion(ParameterExpansion {
                            variable: parts[0].to_string(),
                            operator: ParameterExpansionOperator::RemoveLongestSuffix(parts[1].to_string()),
                        }))
                    } else {
                        Ok(Word::Variable(var_name))
                    }
                } else if var_name.contains("//") {
                    let parts: Vec<&str> = var_name.split("//").collect();
                    if parts.len() == 3 {
                        Ok(Word::ParameterExpansion(ParameterExpansion {
                            variable: parts[0].to_string(),
                            operator: ParameterExpansionOperator::SubstituteAll(parts[1].to_string(), parts[2].to_string()),
                        }))
                    } else {
                        // Check if this is a map access pattern like map[foo]
                        if var_name.contains('[') && var_name.contains(']') {
                            if let Some(bracket_start) = var_name.find('[') {
                                if let Some(bracket_end) = var_name.rfind(']') {
                                    let map_name = &var_name[..bracket_start];
                                    let key = &var_name[bracket_start + 1..bracket_end];
                                    return Ok(Word::MapAccess(map_name.to_string(), key.to_string()));
                                }
                            }
                        }
                        Ok(Word::Variable(var_name))
                    }
                } else if var_name.contains("/") && !var_name.contains("//") {
                    let parts: Vec<&str> = var_name.split("/").collect();
                    if parts.len() == 3 {
                        Ok(Word::ParameterExpansion(ParameterExpansion {
                            variable: parts[0].to_string(),
                            operator: ParameterExpansionOperator::SubstituteAll(parts[1].to_string(), parts[2].to_string()),
                        }))
                    } else {
                        // Check if this is a map access pattern like map[foo]
                        if var_name.contains('[') && var_name.contains(']') {
                            if let Some(bracket_start) = var_name.find('[') {
                                if let Some(bracket_end) = var_name.rfind(']') {
                                    let map_name = &var_name[..bracket_start];
                                    let key = &var_name[bracket_start + 1..bracket_end];
                                    return Ok(Word::MapAccess(map_name.to_string(), key.to_string()));
                                }
                            }
                        }
                        Ok(Word::Variable(var_name))
                    }
                } else {
                    // Check if this is a map access pattern like map[foo]
                    if var_name.contains('[') && var_name.contains(']') {
                        if let Some(bracket_start) = var_name.find('[') {
                            if let Some(bracket_end) = var_name.rfind(']') {
                                let map_name = &var_name[..bracket_start];
                                let key = &var_name[bracket_start + 1..bracket_end];
                                return Ok(Word::MapAccess(map_name.to_string(), key.to_string()));
                            }
                        }
                    }
                    Ok(Word::Variable(var_name))
                }
            }
        }
        Some(Token::DollarParen) => {
            // Parse $(...) command substitution
            lexer.next();
            let command_text = lexer.capture_parenthetical_text()?;
            // For now, create a simple command as a placeholder
            // TODO: Parse the command_text into an actual Command
            let placeholder_cmd = Command::Simple(SimpleCommand {
                name: Word::Literal("echo".to_string()),
                args: vec![Word::Literal(command_text)],
                redirects: Vec::new(),
                env_vars: HashMap::new(),
            });
            Ok(Word::CommandSubstitution(Box::new(placeholder_cmd)))
        }
        _ => {
            let (line, col) = lexer.offset_to_line_col(0);
            Err(ParserError::UnexpectedToken { token: Token::Identifier, line, col })
        }
    }
}

// Placeholder functions - these would need to be implemented based on the actual AST structures
fn parse_string_interpolation(lexer: &mut Lexer) -> Result<Word, ParserError> {
    use crate::ast::{StringInterpolation, StringPart};
    
    // Get the double-quoted string content (this includes the quotes)
    let string_content = lexer.get_string_text()?;
    
    // Remove the outer quotes
    let content = if string_content.starts_with('"') && string_content.ends_with('"') {
        &string_content[1..string_content.len()-1]
    } else {
        &string_content
    };
    
    // Parse the string content to extract literal parts and variable references
    let mut parts = Vec::new();
    let mut current_literal = String::new();
    let mut i = 0;
    
    while i < content.len() {
        if content[i..].starts_with("$") && i + 1 < content.len() {
            // We found a variable reference
            // First, add any accumulated literal text
            if !current_literal.is_empty() {
                parts.push(StringPart::Literal(current_literal.clone()));
                current_literal.clear();
            }
            
            // Parse the variable name
            i += 1; // skip the $
            if i < content.len() {
                let var_start = i;
                while i < content.len() && content[i..].chars().next().unwrap().is_alphanumeric() {
                    i += 1;
                }
                let var_name = &content[var_start..i];
                if !var_name.is_empty() {
                    parts.push(StringPart::Variable(var_name.to_string()));
                }
            }
        } else {
            // Add to current literal
            current_literal.push(content[i..].chars().next().unwrap());
            i += 1;
        }
    }
    
    // Add any remaining literal text
    if !current_literal.is_empty() {
        parts.push(StringPart::Literal(current_literal));
    }
    
    // If we have no parts, this shouldn't happen, but handle it gracefully
    if parts.is_empty() {
        parts.push(StringPart::Literal(content.to_string()));
    }
    
    Ok(Word::StringInterpolation(StringInterpolation { parts }))
}

fn parse_ansic_quoted_string(_lexer: &mut Lexer) -> Result<Word, ParserError> {
    // TODO: Implement ANSI C quoted string parsing
    Err(ParserError::InvalidSyntax("ANSI C quoted strings not yet implemented".to_string()))
}

fn parse_brace_expansion(lexer: &mut Lexer) -> Result<Word, ParserError> {
    use crate::ast::{BraceExpansion, BraceItem, BraceRange};
    
    // Consume the opening brace
    if !matches!(lexer.peek(), Some(Token::BraceOpen)) {
        return Err(ParserError::InvalidSyntax("Expected '{' for brace expansion".to_string()));
    }
    lexer.next(); // consume '{'
    
    let mut items = Vec::new();
    
    // Parse the content inside braces
    loop {
        match lexer.peek() {
            Some(Token::BraceClose) => {
                lexer.next(); // consume '}'
                break;
            }
            Some(Token::Number) => {
                let start = lexer.get_number_text()?;
                
                // Check if this is a range (look for ..)
                if matches!(lexer.peek(), Some(Token::Range)) {
                    lexer.next(); // consume '..'
                    
                    if let Some(Token::Number) = lexer.peek() {
                        let end = lexer.get_number_text()?;
                        items.push(BraceItem::Range(BraceRange {
                            start,
                            end,
                            step: None,
                            format: None,
                        }));
                    } else {
                        return Err(ParserError::InvalidSyntax("Expected number after '..' in brace range".to_string()));
                    }
                } else {
                    // Just a literal number
                    items.push(BraceItem::Literal(start));
                }
            }
            Some(Token::Identifier) => {
                let text = lexer.get_identifier_text()?;
                items.push(BraceItem::Literal(text));
            }
            Some(Token::Comma) => {
                lexer.next(); // consume ','
                // Continue to next item
            }
            _ => {
                return Err(ParserError::InvalidSyntax("Unexpected token in brace expansion".to_string()));
            }
        }
    }
    
    Ok(Word::BraceExpansion(BraceExpansion {
        prefix: None,
        items,
        suffix: None,
    }))
}

fn parse_arithmetic_expression(lexer: &mut Lexer) -> Result<Word, ParserError> {
    // Parse arithmetic expressions like $((i + 1))
    // First, consume the opening $(( or $(
    match lexer.peek() {
        Some(Token::Arithmetic) | Some(Token::ArithmeticEval) => {
            lexer.next(); // consume $(( or $(
        }
        _ => {
            return Err(ParserError::InvalidSyntax("Expected arithmetic expression start".to_string()));
        }
    }
    
    // Capture the content until we find the closing ))
    let mut expression_parts = Vec::new();
    let mut paren_depth = 1; // We're already inside one level of parentheses
    
    loop {
        match lexer.peek() {
            Some(Token::ArithmeticEvalClose) => {
                // This is the closing )) for $((...))
                lexer.next();
                paren_depth -= 1;
                if paren_depth == 0 {
                    break;
                }
            }
            Some(Token::Arithmetic) => {
                // This is another opening $((...))
                lexer.next();
                paren_depth += 1;
            }
            Some(Token::Identifier) => {
                expression_parts.push(lexer.get_identifier_text()?);
            }
            Some(Token::Number) => {
                expression_parts.push(lexer.get_number_text()?);
            }
            Some(Token::Plus) => {
                expression_parts.push("+".to_string());
                lexer.next();
            }
            Some(Token::Minus) => {
                expression_parts.push("-".to_string());
                lexer.next();
            }
            Some(Token::Star) => {
                expression_parts.push("*".to_string());
                lexer.next();
            }
            Some(Token::Slash) => {
                expression_parts.push("/".to_string());
                lexer.next();
            }
            Some(Token::Space) | Some(Token::Tab) => {
                expression_parts.push(" ".to_string());
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
        tokens: Vec::new(), // We don't need to store individual tokens for now
    }))
}

fn parse_braced_variable_name(_lexer: &mut Lexer) -> Result<String, ParserError> {
    // TODO: Implement braced variable name parsing
    Err(ParserError::InvalidSyntax("Braced variable names not yet implemented".to_string()))
}

fn parse_parameter_expansion(_lexer: &mut Lexer) -> Result<Word, ParserError> {
    // TODO: Implement parameter expansion parsing
    Err(ParserError::InvalidSyntax("Parameter expansion not yet implemented".to_string()))
}

fn parse_array_slicing(_lexer: &mut Lexer, _array_name: String) -> Result<Word, ParserError> {
    // TODO: Implement array slicing parsing
    Err(ParserError::InvalidSyntax("Array slicing not yet implemented".to_string()))
}
