use crate::ast::*;
use crate::lexer::{Lexer, Token};
use crate::parser::errors::ParserError;
use crate::parser::utilities::ParserUtilities;
use crate::parser::words::parse_word;
use std::collections::HashMap;

// Add the missing parse_word_list function
fn parse_word_list(lexer: &mut Lexer) -> Result<Vec<Word>, ParserError> {
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

pub fn parse_if_statement(lexer: &mut Lexer) -> Result<Command, ParserError> {
    lexer.consume(Token::If)?;
    
    // Skip whitespace
    lexer.skip_whitespace_and_comments();
    
    // Parse condition - check for arithmetic evaluation first
    let condition = if let Some(Token::ArithmeticEval) = lexer.peek() {
        // Handle arithmetic evaluation like: if (( a > b )); then
        let arithmetic_word = parse_arithmetic_expression(lexer)?;
        Box::new(Command::Simple(SimpleCommand {
            name: Word::Literal("test".to_string()),
            args: vec![arithmetic_word],
            redirects: Vec::new(),
            env_vars: HashMap::new(),
        }))
    } else {
        // Parse as a pipeline to handle && and || operators
        Box::new(parse_pipeline(lexer)?)
    };
    
    // Consume optional separator (semicolon or newline) after condition
    match lexer.peek() {
        Some(Token::Semicolon) | Some(Token::Newline) => { lexer.next(); },
        _ => {}
    }
    
    // Skip whitespace/newlines before then
    while matches!(lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline)) {
        lexer.next();
    }
    
    lexer.consume(Token::Then)?;
    // Allow newline/whitespace after 'then'
    while matches!(lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline)) {
        lexer.next();
    }
    
    // Parse one or more commands in the then-branch until Else, Elif, or Fi
    let mut then_cmds = Vec::new();
    loop {
        match lexer.peek() {
            Some(Token::Else) | Some(Token::Elif) | Some(Token::Fi) | None => break,
            _ => {
                let cmd = parse_command(lexer)?;
                then_cmds.push(cmd);
                // Skip separators between commands
                while matches!(lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline | Token::Semicolon | Token::CarriageReturn)) {
                    lexer.next();
                }
            }
        }
    }
    let then_branch = Box::new(Command::Block(Block { commands: then_cmds }));
    
    // Skip whitespace/newlines before checking for separator
    while matches!(lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline)) {
        lexer.next();
    }
    
    // Consume optional separator (semicolon or newline) after then branch
    match lexer.peek() {
        Some(Token::Semicolon) | Some(Token::Newline) => {
            lexer.next();
            while matches!(lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline)) {
                lexer.next();
            }
        },
        _ => {}
    }
    
    let else_branch = if let Some(Token::Else) = lexer.peek() {
        lexer.next();
        // Allow newline/whitespace after 'else'
        while matches!(lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline)) {
            lexer.next();
        }
        let mut else_cmds = Vec::new();
        loop {
            match lexer.peek() {
                Some(Token::Fi) | None => break,
                _ => {
                    let cmd = parse_command(lexer)?;
                    else_cmds.push(cmd);
                    while matches!(lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline | Token::Semicolon | Token::CarriageReturn)) {
                        lexer.next();
                    }
                }
            }
        }
        Some(Box::new(Command::Block(Block { commands: else_cmds })))
    } else if let Some(Token::Elif) = lexer.peek() {
        // Handle multiple elif statements by building a nested if-else structure
        let mut elif_branches = Vec::new();
        
        // Parse all elif statements
        while let Some(Token::Elif) = lexer.peek() {
            lexer.next();
            // Allow newline/whitespace after 'elif'
            while matches!(lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline)) {
                lexer.next();
            }
            
            // Parse the elif condition
            let elif_condition = if let Some(Token::ArithmeticEval) = lexer.peek() {
                // Handle arithmetic evaluation like: elif (( a == b )); then
                let arithmetic_word = parse_arithmetic_expression(lexer)?;
                Box::new(Command::Simple(SimpleCommand {
                    name: Word::Literal("test".to_string()),
                    args: vec![arithmetic_word],
                    redirects: Vec::new(),
                    env_vars: HashMap::new(),
                }))
            } else {
                // Parse as a pipeline to handle && and || operators
                Box::new(parse_pipeline(lexer)?)
            };
            
            // Consume optional separator (semicolon or newline) after condition
            match lexer.peek() {
                Some(Token::Semicolon) | Some(Token::Newline) => { lexer.next(); },
                _ => {}
            }
            
            // Skip whitespace/newlines before then
            while matches!(lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline)) {
                lexer.next();
            }
            
            lexer.consume(Token::Then)?;
            // Allow newline/whitespace after 'then'
            while matches!(lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline)) {
                lexer.next();
            }
            
            // Parse one or more commands in the elif then-branch until Else, Elif, or Fi
            let mut elif_then_cmds = Vec::new();
            loop {
                match lexer.peek() {
                    Some(Token::Else) | Some(Token::Elif) | Some(Token::Fi) | None => break,
                    _ => {
                        let cmd = parse_command(lexer)?;
                        elif_then_cmds.push(cmd);
                        // Skip separators between commands
                        while matches!(lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline | Token::Semicolon | Token::CarriageReturn)) {
                            lexer.next();
                        }
                    }
                }
            }
            let elif_then_branch = Box::new(Command::Block(Block { commands: elif_then_cmds }));
            
            elif_branches.push((elif_condition, elif_then_branch));
        }
        
        // Now check for else statement
        let final_else_branch = if let Some(Token::Else) = lexer.peek() {
            lexer.next();
            // Allow newline/whitespace after 'else'
            while matches!(lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline)) {
                lexer.next();
            }
            let mut else_cmds = Vec::new();
            loop {
                match lexer.peek() {
                    Some(Token::Fi) | None => break,
                    _ => {
                        let cmd = parse_command(lexer)?;
                        else_cmds.push(cmd);
                        while matches!(lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline | Token::Semicolon | Token::CarriageReturn)) {
                            lexer.next();
                        }
                    }
                }
            }
            Some(Box::new(Command::Block(Block { commands: else_cmds })))
        } else {
            None
        };
        
        // Build nested if-else structure
        let mut current_else_branch = final_else_branch;
        
        // Build from the last elif to the first
        for (condition, then_branch) in elif_branches.into_iter().rev() {
            current_else_branch = Some(Box::new(Command::If(IfStatement {
                condition,
                then_branch,
                else_branch: current_else_branch,
            })));
        }
        
        current_else_branch
    } else {
        None
    };
    
    // Skip whitespace/newlines before fi
    while matches!(lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline)) {
        lexer.next();
    }
    
    lexer.consume(Token::Fi)?;
    
    Ok(Command::If(IfStatement {
        condition,
        then_branch,
        else_branch,
    }))
}

pub fn parse_case_statement(lexer: &mut Lexer) -> Result<Command, ParserError> {
    lexer.consume(Token::Case)?;
    
    // Skip whitespace after 'case'
    lexer.skip_whitespace_and_comments();
    
    // Parse the word to match against
    let word = parse_word(lexer)?;
    
    // Skip whitespace before 'in'
    lexer.skip_whitespace_and_comments();
    
    // Consume 'in'
    lexer.consume(Token::In)?;
    
    // Skip whitespace after 'in'
    lexer.skip_whitespace_and_comments();
    
    let mut cases = Vec::new();
    
    // Parse case clauses until 'esac'
    loop {
        // Skip whitespace/newlines
        lexer.skip_whitespace_and_comments();
        
        match lexer.peek() {
            Some(Token::Esac) => break,
            None => return Err(ParserError::UnexpectedEOF),
            _ => {
                // Parse a case clause
                let mut patterns = Vec::new();
                
                // Parse first pattern
                patterns.push(parse_word(lexer)?);
                
                // Parse additional patterns separated by '|'
                while matches!(lexer.peek(), Some(Token::Pipe)) {
                    lexer.next(); // consume '|'
                    lexer.skip_whitespace_and_comments();
                    patterns.push(parse_word(lexer)?);
                }
                
                // Expect closing parenthesis as part of the case pattern
                if let Some(Token::ParenClose) = lexer.peek() {
                    lexer.next(); // consume ')'
                } else {
                    return Err(ParserError::InvalidSyntax("Expected ')' after case pattern".to_string()));
                }
                
                // Skip whitespace after pattern
                lexer.skip_whitespace_and_comments();
                
                // Parse body commands until ';;'
                let mut body = Vec::new();
                loop {
                    match lexer.peek() {
                        Some(Token::DoubleSemicolon) => break,
                        Some(Token::Esac) => break,
                        None => return Err(ParserError::UnexpectedEOF),
                        _ => {
                            let cmd = parse_command(lexer)?;
                            body.push(cmd);
                            // Skip separators between commands
                            while matches!(lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline | Token::Semicolon | Token::CarriageReturn)) {
                                lexer.next();
                            }
                        }
                    }
                }
                
                // Consume ';;' if present
                if matches!(lexer.peek(), Some(Token::DoubleSemicolon)) {
                    lexer.next();
                }
                
                cases.push(CaseClause { patterns, body });
            }
        }
    }
    
    // Consume 'esac'
    lexer.consume(Token::Esac)?;
    
    Ok(Command::Case(CaseStatement { word, cases }))
}

pub fn parse_while_loop(lexer: &mut Lexer) -> Result<Command, ParserError> {
    lexer.consume(Token::While)?;
    // Parse condition
    let condition = Box::new(parse_command(lexer)?);

    // Optional separator after condition (semicolon or newline) and skip whitespace
    match lexer.peek() {
        Some(Token::Semicolon) | Some(Token::Newline) | Some(Token::CarriageReturn) => { lexer.next(); },
        _ => {}
    }
    while matches!(lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline | Token::CarriageReturn)) {
        lexer.next();
    }

    // Expect 'do'
    lexer.consume(Token::Do)?;

    // Allow newline/whitespace after 'do'
    while matches!(lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline | Token::CarriageReturn)) {
        lexer.next();
    }

    // Parse body commands into a Block
    let mut body_commands = Vec::new();
    
    // Parse first command
    body_commands.push(parse_command(lexer)?);

    // Parse additional commands in body until 'done'
    loop {
        // Skip separators
        while matches!(lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline | Token::CarriageReturn | Token::Semicolon)) {
            lexer.next();
        }
        match lexer.peek() {
            Some(Token::Done) | None => break,
            _ => {
                // Parse and add command to body
                let pre_pos = lexer.current_position();
                let command = parse_command(lexer)?;
                body_commands.push(command);
                if lexer.current_position() == pre_pos {
                    if lexer.next().is_none() { break; }
                }
            }
        }
    }

    // Allow optional separator after body before 'done'
    loop {
        match lexer.peek() {
            Some(Token::Space) | Some(Token::Tab) | Some(Token::Comment) | Some(Token::Newline | Token::CarriageReturn) => {
                lexer.next();
                continue;
            }
            Some(Token::Semicolon) => {
                lexer.next();
                // consume any following whitespace/newlines as well
                continue;
            }
            _ => {}
        }
        break;
    }

    lexer.consume(Token::Done)?;
    
    let body = Block { commands: body_commands };
    Ok(Command::While(WhileLoop { condition, body }))
}

pub fn parse_for_loop(lexer: &mut Lexer) -> Result<Command, ParserError> {
    lexer.consume(Token::For)?;
    // Allow whitespace/comments after 'for'
    lexer.skip_whitespace_and_comments();

    // Variable name
    let variable = match lexer.peek() {
        Some(Token::Identifier) => lexer.get_identifier_text()?,
        Some(t) => return Err(ParserError::UnexpectedToken { token: t.clone(), line: 1, col: 1 }),
        None => return Err(ParserError::UnexpectedEOF),
    };

    // Allow whitespace/comments after variable
    lexer.skip_whitespace_and_comments();

    // Optional 'in' list
    let items = if let Some(Token::In) = lexer.peek() {
        lexer.next();
        // Allow whitespace/comments after 'in'
        lexer.skip_whitespace_and_comments();
        let words = parse_word_list(lexer)?;
        // Optional separator before 'do'
        while matches!(lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::CarriageReturn)) {
            lexer.next();
        }
        match lexer.peek() {
            Some(Token::Semicolon) | Some(Token::Newline) | Some(Token::CarriageReturn) => {
                lexer.next();
            }
            _ => {}
        }
        words
    } else {
        // No 'in' list; optional separator before 'do'
        while matches!(lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::CarriageReturn)) {
            lexer.next();
        }
        match lexer.peek() {
            Some(Token::Semicolon) | Some(Token::Newline) | Some(Token::CarriageReturn) => {
                lexer.next();
            }
            _ => {}
        }
        Vec::new()
    };

    // Allow whitespace/newlines/comments before 'do'
    while matches!(lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline | Token::CarriageReturn)) {
        lexer.next();
    }
    lexer.consume(Token::Do)?;
    
    // Parse body commands into a Block
    let mut body_commands = Vec::new();
    
    // Parse first command
    body_commands.push(parse_command(lexer)?);

    // Parse additional commands in body until 'done'
    loop {
        // Skip separators
        while matches!(lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline | Token::CarriageReturn)) {
            lexer.next();
        }
        
        // Check for 'done' first
        if let Some(Token::Done) = lexer.peek() {
            break;
        }
        
        // Check for semicolon - this should separate commands in the loop body
        if let Some(Token::Semicolon) = lexer.peek() {
            lexer.next(); // consume semicolon
            // Skip whitespace after semicolon
            lexer.skip_whitespace_and_comments();
            
            // Check if the next token is 'done'
            if let Some(Token::Done) = lexer.peek() {
                break;
            }
            
            // Continue parsing the next command in the loop body
            continue;
        }
        
        // Parse additional command in body
        let pre_pos = lexer.current_position();
        let command = parse_command(lexer)?;
        body_commands.push(command);
        if lexer.current_position() == pre_pos {
            if lexer.next().is_none() { break; }
        }
    }

    // Allow optional separator after body before 'done'
    loop {
        match lexer.peek() {
            Some(Token::Space) | Some(Token::Tab) | Some(Token::Comment) | Some(Token::Newline | Token::CarriageReturn) => {
                lexer.next();
                continue;
            }
            Some(Token::Semicolon) => {
                lexer.next();
                // consume any following whitespace/newlines as well
                continue;
            }
            _ => {}
        }
        break;
    }

    lexer.consume(Token::Done)?;
    
    // Skip whitespace after 'done' before checking for pipe
    lexer.skip_whitespace_and_comments();
    
    // Check if there's a pipeline after the for loop
    let mut final_command = Command::For(ForLoop {
        variable,
        items,
        body: Block { commands: body_commands },
    });
    
    // If there's a pipe after 'done', parse the pipeline
    if let Some(Token::Pipe) = lexer.peek() {
        final_command = parse_pipeline_from_command(lexer, final_command)?;
    }
    
    Ok(final_command)
}

pub fn parse_function(lexer: &mut Lexer) -> Result<Command, ParserError> {
    lexer.consume(Token::Function)?;
    // Allow whitespace between 'function' and name
    lexer.skip_whitespace_and_comments();

    let name = match lexer.peek() {
        Some(Token::Identifier) => lexer.get_identifier_text()?,
        Some(t) => {
            let (line, col) = lexer.offset_to_line_col(0);
            return Err(ParserError::UnexpectedToken { token: t.clone(), line, col });
        }
        None => return Err(ParserError::UnexpectedEOF),
    };

    // Skip whitespace after name
    lexer.skip_whitespace_and_comments();

    // Parse parameters if present: function name(param1, param2)
    let mut parameters = Vec::new();
    if let Some(Token::ParenOpen) = lexer.peek() {
        // Consume opening parenthesis
        lexer.next();
        
        // Parse parameters until closing parenthesis
        loop {
            lexer.skip_whitespace_and_comments();
            
            match lexer.peek() {
                Some(Token::ParenClose) => {
                    lexer.next(); // consume closing parenthesis
                    break;
                }
                Some(Token::Identifier) => {
                    let param = lexer.get_identifier_text()?;
                    parameters.push(param);
                    
                    // Check for comma separator
                    lexer.skip_whitespace_and_comments();
                    if let Some(Token::Comma) = lexer.peek() {
                        lexer.next(); // consume comma
                    } else if let Some(Token::ParenClose) = lexer.peek() {
                        // No comma, must be last parameter
                        continue;
                    } else {
                        // Expect comma or closing parenthesis
                        let (line, col) = lexer.offset_to_line_col(0);
                        return Err(ParserError::UnexpectedToken { 
                            token: lexer.peek().unwrap().clone(), 
                            line, 
                            col 
                        });
                    }
                }
                _ => {
                    let (line, col) = lexer.offset_to_line_col(0);
                    return Err(ParserError::UnexpectedToken { 
                        token: lexer.peek().unwrap().clone(), 
                        line, 
                        col 
                    });
                }
            }
        }
        
        // Skip whitespace/newlines after parentheses
        while matches!(lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline)) {
            lexer.next();
        }
    }

    // Brace-wrapped function body: { ... }
    let body = if let Some(Token::BraceOpen) = lexer.peek() {
        // Consume '{'
        lexer.next();
        // Allow whitespace/newlines
        while matches!(lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline)) {
            lexer.next();
        }
        
        // Parse body commands into a Block
        let mut body_commands = Vec::new();
        
        // Parse first command
        body_commands.push(parse_command(lexer)?);

        // Parse additional commands inside the block
        loop {
            // Skip separators
            while matches!(lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline | Token::Semicolon)) {
                lexer.next();
            }
            match lexer.peek() {
                Some(Token::BraceClose) | None => break,
                _ => {
                    let pre_pos = lexer.current_position();
                    let command = parse_command(lexer)?;
                    body_commands.push(command);
                    if lexer.current_position() == pre_pos {
                        if lexer.next().is_none() { break; }
                    }
                }
            }
        }

        // Expect closing '}'
        lexer.consume(Token::BraceClose)?;
        Block { commands: body_commands }
    } else {
        // Fallback: parse next as a single command body
        let command = parse_command(lexer)?;
        Block { commands: vec![command] }
    };
    
    Ok(Command::Function(Function { name, parameters, body }))
}

pub fn parse_posix_function(lexer: &mut Lexer) -> Result<Command, ParserError> {
    // Get the function name
    let name = lexer.get_identifier_text()?;
    
    // Consume the opening parenthesis
    lexer.consume(Token::ParenOpen)?;
    
    // Parse parameters if present: name(param1, param2)
    let mut parameters = Vec::new();
    if let Some(Token::ParenClose) = lexer.peek() {
        // No parameters, just consume closing parenthesis
        lexer.next();
    } else {
        // Parse parameters until closing parenthesis
        loop {
            lexer.skip_whitespace_and_comments();
            
            match lexer.peek() {
                Some(Token::ParenClose) => {
                    lexer.next(); // consume closing parenthesis
                    break;
                }
                Some(Token::Identifier) => {
                    let param = lexer.get_identifier_text()?;
                    parameters.push(param);
                    
                    // Check for comma separator
                    lexer.skip_whitespace_and_comments();
                    if let Some(Token::Comma) = lexer.peek() {
                        lexer.next(); // consume comma
                    } else if let Some(Token::ParenClose) = lexer.peek() {
                        // No comma, must be last parameter
                        continue;
                    } else {
                        // Expect comma or closing parenthesis
                        let (line, col) = lexer.offset_to_line_col(0);
                        return Err(ParserError::UnexpectedToken { 
                            token: lexer.peek().unwrap().clone(), 
                            line, 
                            col 
                        });
                    }
                }
                _ => {
                    let (line, col) = lexer.offset_to_line_col(0);
                    return Err(ParserError::UnexpectedToken { 
                        token: lexer.peek().unwrap().clone(), 
                        line, 
                        col 
                    });
                }
            }
        }
    }
    
    // Skip whitespace/newlines after parentheses
    while matches!(lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline)) {
        lexer.next();
    }
    
    // Consume the opening brace
    lexer.consume(Token::BraceOpen)?;
    
    // Allow whitespace/newlines after opening brace
    while matches!(lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline)) {
        lexer.next();
    }
    
    // Parse the function body as a block of commands
    let mut body_commands = Vec::new();
    
    // Parse commands until we find the closing brace
    loop {
        // Skip separators
        while matches!(lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline | Token::Semicolon)) {
            lexer.next();
        }
        
        match lexer.peek() {
            Some(Token::BraceClose) => {
                lexer.next(); // consume the closing brace
                break;
            }
            None => {
                return Err(ParserError::UnexpectedEOF);
            }
            _ => {
                // Parse the next command
                let command = parse_command(lexer)?;
                body_commands.push(command);
            }
        }
    }
    
    Ok(Command::Function(Function { 
        name, 
        parameters,
        body: Block { commands: body_commands }
    }))
}

pub fn parse_block(lexer: &mut Lexer) -> Result<Command, ParserError> {
    // Parse a standalone block: { ... }
    
    // Consume the opening brace
    lexer.consume(Token::BraceOpen)?;
    
    // Allow whitespace/newlines after opening brace
    while matches!(lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline)) {
        lexer.next();
    }
    
    // Parse the block body as a list of commands
    let mut body_commands = Vec::new();
    
    // Parse commands until we find the closing brace
    loop {
        // Skip separators
        while matches!(lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline | Token::Semicolon)) {
            lexer.next();
        }
        
        match lexer.peek() {
            Some(Token::BraceClose) => {
                lexer.next(); // consume the closing brace
                break;
            }
            None => {
                return Err(ParserError::UnexpectedEOF);
            }
            _ => {
                // Parse the next command
                let command = parse_command(lexer)?;
                body_commands.push(command);
            }
        }
    }
    
    Ok(Command::Block(Block { commands: body_commands }))
}

pub fn parse_break_statement(lexer: &mut Lexer) -> Result<Command, ParserError> {
    lexer.consume(Token::Break)?;
    
    // Optional argument (loop level)
    let mut level = None;
    lexer.skip_whitespace_and_comments();
    
    if let Some(Token::Number) = lexer.peek() {
        let level_text = lexer.get_number_text()?;
        level = Some(level_text);
    }
    
    Ok(Command::Break(level))
}

pub fn parse_continue_statement(lexer: &mut Lexer) -> Result<Command, ParserError> {
    lexer.consume(Token::Continue)?;
    
    // Optional argument (loop level)
    let mut level = None;
    lexer.skip_whitespace_and_comments();
    
    if let Some(Token::Number) = lexer.peek() {
        let level_text = lexer.get_number_text()?;
        level = Some(level_text);
    }
    
    Ok(Command::Continue(level))
}

pub fn parse_return_statement(lexer: &mut Lexer) -> Result<Command, ParserError> {
    lexer.consume(Token::Return)?;
    
    // Optional return value
    let mut return_value = None;
    lexer.skip_whitespace_and_comments();
    
    if !lexer.is_eof() && !matches!(lexer.peek(), Some(Token::Newline | Token::Semicolon | Token::CarriageReturn)) {
        // Parse the return value as a word
        return_value = Some(parse_word(lexer)?);
    }
    
    Ok(Command::Return(return_value))
}

// Placeholder functions - these would need to be implemented based on the actual AST structures
fn parse_arithmetic_expression(_lexer: &mut Lexer) -> Result<Word, ParserError> {
    // TODO: Implement arithmetic expression parsing
    Err(ParserError::InvalidSyntax("Arithmetic expressions not yet implemented".to_string()))
}

fn parse_pipeline(_lexer: &mut Lexer) -> Result<Command, ParserError> {
    // TODO: Implement pipeline parsing
    Err(ParserError::InvalidSyntax("Pipeline parsing not yet implemented".to_string()))
}

fn parse_command(_lexer: &mut Lexer) -> Result<Command, ParserError> {
    // TODO: Implement command parsing
    Err(ParserError::InvalidSyntax("Command parsing not yet implemented".to_string()))
}

fn parse_pipeline_from_command(_lexer: &mut Lexer, _command: Command) -> Result<Command, ParserError> {
    // TODO: Implement pipeline from command parsing
    Err(ParserError::InvalidSyntax("Pipeline from command parsing not yet implemented".to_string()))
}
