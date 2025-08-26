use crate::ast::*;
use crate::lexer::{Lexer, Token};
use crate::parser::errors::ParserError;
use crate::parser::utilities::ParserUtilities;
use crate::parser::commands::Parser;
use crate::parser::words::parse_word;
use std::collections::HashMap;

// Add the missing parse_word_list function
fn parse_word_list(parser: &mut Parser) -> Result<Vec<Word>, ParserError> {
    let mut words = Vec::new();
    
    loop {
        // Skip whitespace and comments
        parser.lexer.skip_whitespace_and_comments();
        
        // Check for end of list
        if parser.lexer.is_eof() || matches!(parser.lexer.peek(), Some(Token::Semicolon | Token::Newline | Token::CarriageReturn | Token::Done | Token::Fi | Token::Then | Token::Else | Token::ParenClose | Token::BraceClose)) {
            break;
        }
        
        // Parse the next word
        let word = parse_word(&mut parser.lexer)?;
        words.push(word);
        
        // Skip whitespace after the word
        parser.lexer.skip_whitespace_and_comments();
    }
    
    Ok(words)
}

pub fn parse_if_statement(parser: &mut Parser) -> Result<Command, ParserError> {
    parser.lexer.consume(Token::If)?;
    
    // Skip whitespace
    parser.lexer.skip_whitespace_and_comments();
    
    // Parse condition - check for test expression first, then arithmetic evaluation
    let condition = if let Some(Token::TestBracket) = parser.lexer.peek() {
        // Handle test expression like: if [ -f "file.txt" ]; then
        Box::new(parse_test_expression(&mut parser.lexer)?)
    } else if let Some(Token::ArithmeticEval) = parser.lexer.peek() {
        // Handle arithmetic evaluation like: if (( a > b )); then
        let arithmetic_word = parse_arithmetic_expression(parser)?;
        Box::new(Command::Simple(SimpleCommand {
            name: Word::Literal("test".to_string()),
            args: vec![arithmetic_word],
            redirects: Vec::new(),
            env_vars: HashMap::new(),
        }))
    } else {
        // Parse as a pipeline to handle && and || operators
        Box::new(parse_pipeline(parser)?)
    };
    
    // Consume optional separator (semicolon or newline) after condition
    match parser.lexer.peek() {
        Some(Token::Semicolon) | Some(Token::Newline) => { parser.lexer.next(); },
        _ => {}
    }
    
    // Skip whitespace/newlines before then
    while matches!(parser.lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline)) {
        parser.lexer.next();
    }
    
    parser.lexer.consume(Token::Then)?;
    // Allow newline/whitespace after 'then'
    while matches!(parser.lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline)) {
        parser.lexer.next();
    }
    
    // Parse one or more commands in the then-branch until Else, Elif, or Fi
    let mut then_cmds = Vec::new();
    loop {
        match parser.lexer.peek() {
            Some(Token::Else) | Some(Token::Elif) | Some(Token::Fi) | None => break,
            _ => {
                let cmd = parser.parse_command()?;
                then_cmds.push(cmd);
                // Skip separators between commands
                while matches!(parser.lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline | Token::Semicolon | Token::CarriageReturn)) {
                    parser.lexer.next();
                }
            }
        }
    }
    let then_branch = Box::new(Command::Block(Block { commands: then_cmds }));
    
    // Skip whitespace/newlines before checking for separator
    while matches!(parser.lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline)) {
        parser.lexer.next();
    }
    
    // Consume optional separator (semicolon or newline) after then branch
    match parser.lexer.peek() {
        Some(Token::Semicolon) | Some(Token::Newline) => {
            parser.lexer.next();
            while matches!(parser.lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline)) {
                parser.lexer.next();
            }
        },
        _ => {}
    }
    
    let else_branch = if let Some(Token::Else) = parser.lexer.peek() {
        parser.lexer.next();
        // Allow newline/whitespace after 'else'
        while matches!(parser.lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline)) {
            parser.lexer.next();
        }
        let mut else_cmds = Vec::new();
        loop {
            match parser.lexer.peek() {
                Some(Token::Fi) | None => break,
                _ => {
                    let cmd = parser.parse_command()?;
                    else_cmds.push(cmd);
                    while matches!(parser.lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline | Token::Semicolon | Token::CarriageReturn)) {
                        parser.lexer.next();
                    }
                }
            }
        }
        Some(Box::new(Command::Block(Block { commands: else_cmds })))
    } else if let Some(Token::Elif) = parser.lexer.peek() {
        // Handle multiple elif statements by building a nested if-else structure
        let mut elif_branches = Vec::new();
        
        // Parse all elif statements
        while let Some(Token::Elif) = parser.lexer.peek() {
            parser.lexer.next();
            // Allow newline/whitespace after 'elif'
            while matches!(parser.lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline)) {
                parser.lexer.next();
            }
            
            // Parse the elif condition
            let elif_condition = if let Some(Token::TestBracket) = parser.lexer.peek() {
                // Handle test expression like: elif [ -f "file.txt" ]; then
                Box::new(parse_test_expression(&mut parser.lexer)?)
            } else if let Some(Token::ArithmeticEval) = parser.lexer.peek() {
                // Handle arithmetic evaluation like: elif (( a == b )); then
                            let arithmetic_word = parse_arithmetic_expression(parser)?;
            Box::new(Command::Simple(SimpleCommand {
                name: Word::Literal("test".to_string()),
                args: vec![arithmetic_word],
                redirects: Vec::new(),
                env_vars: HashMap::new(),
            }))
        } else {
            // Parse as a pipeline to handle && and || operators
            Box::new(parse_pipeline(parser)?)
            };
            
            // Consume optional separator (semicolon or newline) after condition
            match parser.lexer.peek() {
                Some(Token::Semicolon) | Some(Token::Newline) => { parser.lexer.next(); },
                _ => {}
            }
            
            // Skip whitespace/newlines before then
            while matches!(parser.lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline)) {
                parser.lexer.next();
            }
            
            parser.lexer.consume(Token::Then)?;
            // Allow newline/whitespace after 'then'
            while matches!(parser.lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline)) {
                parser.lexer.next();
            }
            
            // Parse one or more commands in the elif then-branch until Else, Elif, or Fi
            let mut elif_then_cmds = Vec::new();
            loop {
                match parser.lexer.peek() {
                    Some(Token::Else) | Some(Token::Elif) | Some(Token::Fi) | None => break,
                    _ => {
                        let cmd = parser.parse_command()?;
                        elif_then_cmds.push(cmd);
                        // Skip separators between commands
                        while matches!(parser.lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline | Token::Semicolon | Token::CarriageReturn)) {
                            parser.lexer.next();
                        }
                    }
                }
            }
            let elif_then_branch = Box::new(Command::Block(Block { commands: elif_then_cmds }));
            
            elif_branches.push((elif_condition, elif_then_branch));
        }
        
        // Now check for else statement
        let final_else_branch = if let Some(Token::Else) = parser.lexer.peek() {
            parser.lexer.next();
            // Allow newline/whitespace after 'else'
            while matches!(parser.lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline)) {
                parser.lexer.next();
            }
            let mut else_cmds = Vec::new();
            loop {
                match parser.lexer.peek() {
                    Some(Token::Fi) | None => break,
                    _ => {
                        let cmd = parser.parse_command()?;
                        else_cmds.push(cmd);
                        while matches!(parser.lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline | Token::Semicolon | Token::CarriageReturn)) {
                            parser.lexer.next();
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
    while matches!(parser.lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline)) {
        parser.lexer.next();
    }
    
    parser.lexer.consume(Token::Fi)?;
    
    Ok(Command::If(IfStatement {
        condition,
        then_branch,
        else_branch,
    }))
}

pub fn parse_case_statement(parser: &mut Parser) -> Result<Command, ParserError> {
    parser.lexer.consume(Token::Case)?;
    
    // Skip whitespace after 'case'
    parser.lexer.skip_whitespace_and_comments();
    
    // Parse the word to match against
    let word = parse_word(&mut parser.lexer)?;
    
    // Skip whitespace before 'in'
    parser.lexer.skip_whitespace_and_comments();
    
    // Consume 'in'
    parser.lexer.consume(Token::In)?;
    
    // Skip whitespace after 'in'
    parser.lexer.skip_whitespace_and_comments();
    
    let mut cases = Vec::new();
    
    // Parse case clauses until 'esac'
    loop {
        // Skip whitespace/newlines
        parser.lexer.skip_whitespace_and_comments();
        
        match parser.lexer.peek() {
            Some(Token::Esac) => break,
            None => return Err(ParserError::UnexpectedEOF),
            _ => {
                // Parse a case clause
                let mut patterns = Vec::new();
                
                // Parse first pattern
                patterns.push(parse_word(&mut parser.lexer)?);
                
                // Parse additional patterns separated by '|'
                while matches!(parser.lexer.peek(), Some(Token::Pipe)) {
                    parser.lexer.next(); // consume '|'
                    parser.lexer.skip_whitespace_and_comments();
                    patterns.push(parse_word(&mut parser.lexer)?);
                }
                
                // Expect closing parenthesis as part of the case pattern
                if let Some(Token::ParenClose) = parser.lexer.peek() {
                    parser.lexer.next(); // consume ')'
                } else {
                    return Err(ParserError::InvalidSyntax("Expected ')' after case pattern".to_string()));
                }
                
                // Skip whitespace after pattern
                parser.lexer.skip_whitespace_and_comments();
                
                // Parse body commands until ';;'
                let mut body = Vec::new();
                loop {
                    match parser.lexer.peek() {
                        Some(Token::DoubleSemicolon) => break,
                        Some(Token::Esac) => break,
                        None => return Err(ParserError::UnexpectedEOF),
                        _ => {
                            let cmd = parser.parse_command()?;
                            body.push(cmd);
                            // Skip separators between commands
                            while matches!(parser.lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline | Token::Semicolon | Token::CarriageReturn)) {
                                parser.lexer.next();
                            }
                        }
                    }
                }
                
                // Consume ';;' if present
                if matches!(parser.lexer.peek(), Some(Token::DoubleSemicolon)) {
                    parser.lexer.next();
                }
                
                cases.push(CaseClause { patterns, body });
            }
        }
    }
    
    // Consume 'esac'
    parser.lexer.consume(Token::Esac)?;
    
    Ok(Command::Case(CaseStatement { word, cases }))
}

pub fn parse_while_loop(parser: &mut Parser) -> Result<Command, ParserError> {
    parser.lexer.consume(Token::While)?;
    // Skip whitespace after 'while'
    parser.lexer.skip_whitespace_and_comments();
    // Parse condition - check for test expression first
    let condition = if let Some(Token::TestBracket) = parser.lexer.peek() {
        // Handle test expression like: while [ $i -lt 10 ]; do
        Box::new(parse_test_expression(&mut parser.lexer)?)
    } else {
        // Parse as a regular command
        Box::new(parser.parse_command()?)
    };

    // Optional separator after condition (semicolon or newline) and skip whitespace
    match parser.lexer.peek() {
        Some(Token::Semicolon) | Some(Token::Newline) | Some(Token::CarriageReturn) => { parser.lexer.next(); },
        _ => {}
    }
    while matches!(parser.lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline | Token::CarriageReturn)) {
        parser.lexer.next();
    }

    // Expect 'do'
    parser.lexer.consume(Token::Do)?;

    // Allow newline/whitespace after 'do'
    while matches!(parser.lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline | Token::CarriageReturn)) {
        parser.lexer.next();
    }

    // Parse body commands into a Block
    let mut body_commands = Vec::new();
    
    // Parse commands in body until 'done'
    loop {
        // Skip separators
        while matches!(parser.lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline | Token::CarriageReturn | Token::Semicolon)) {
            parser.lexer.next();
        }
        match parser.lexer.peek() {
            Some(Token::Done) | None => break,
            _ => {
                // Parse and add command to body
                let pre_pos = parser.lexer.current_position();
                let command = parser.parse_command()?;
                body_commands.push(command);
                if parser.lexer.current_position() == pre_pos {
                    if parser.lexer.next().is_none() { break; }
                }
            }
        }
    }

    // Allow optional separator after body before 'done'
    loop {
        match parser.lexer.peek() {
            Some(Token::Space) | Some(Token::Tab) | Some(Token::Comment) | Some(Token::Newline | Token::CarriageReturn) => {
                parser.lexer.next();
                continue;
            }
            Some(Token::Semicolon) => {
                parser.lexer.next();
                // consume any following whitespace/newlines as well
                continue;
            }
            _ => {}
        }
        break;
    }

    parser.lexer.consume(Token::Done)?;
    
    let body = Block { commands: body_commands };
    Ok(Command::While(WhileLoop { condition, body }))
}

pub fn parse_for_loop(parser: &mut Parser) -> Result<Command, ParserError> {
    parser.lexer.consume(Token::For)?;
    // Allow whitespace/comments after 'for'
    parser.lexer.skip_whitespace_and_comments();

    // Variable name
    let variable = match parser.lexer.peek() {
        Some(Token::Identifier) => parser.lexer.get_identifier_text()?,
        Some(t) => return Err(ParserError::UnexpectedToken { token: t.clone(), line: 1, col: 1 }),
        None => return Err(ParserError::UnexpectedEOF),
    };

    // Allow whitespace/comments after variable
    parser.lexer.skip_whitespace_and_comments();

    // Optional 'in' list
    let items = if let Some(Token::In) = parser.lexer.peek() {
        parser.lexer.next();
        // Allow whitespace/comments after 'in'
        parser.lexer.skip_whitespace_and_comments();
        let words = parse_word_list(parser)?;
        // Optional separator before 'do'
        while matches!(parser.lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::CarriageReturn)) {
            parser.lexer.next();
        }
        match parser.lexer.peek() {
            Some(Token::Semicolon) | Some(Token::Newline) | Some(Token::CarriageReturn) => {
                parser.lexer.next();
            }
            _ => {}
        }
        words
    } else {
        // No 'in' list; optional separator before 'do'
        while matches!(parser.lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::CarriageReturn)) {
            parser.lexer.next();
        }
        match parser.lexer.peek() {
            Some(Token::Semicolon) | Some(Token::Newline) | Some(Token::CarriageReturn) => {
                parser.lexer.next();
            }
            _ => {}
        }
        Vec::new()
    };

    // Allow whitespace/newlines/comments before 'do'
    while matches!(parser.lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline | Token::CarriageReturn)) {
        parser.lexer.next();
    }
    parser.lexer.consume(Token::Do)?;
    
    // Parse body commands into a Block
    let mut body_commands = Vec::new();
    
    // Parse commands in body until 'done'
    loop {
        // Skip separators
        while matches!(parser.lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline | Token::CarriageReturn)) {
            parser.lexer.next();
        }
        
        // Check for 'done' first
        if let Some(Token::Done) = parser.lexer.peek() {
            break;
        }
        
        // Check for semicolon - this should separate commands in the loop body
        if let Some(Token::Semicolon) = parser.lexer.peek() {
            parser.lexer.next(); // consume semicolon
            // Skip whitespace after semicolon
            parser.lexer.skip_whitespace_and_comments();
            
            // Check if the next token is 'done'
            if let Some(Token::Done) = parser.lexer.peek() {
                break;
            }
            
            // Continue parsing the next command in the loop body
            continue;
        }
        
        // Parse command in body
        let pre_pos = parser.lexer.current_position();
        let command = parser.parse_command()?;
        body_commands.push(command);
        if parser.lexer.current_position() == pre_pos {
            if parser.lexer.next().is_none() { break; }
        }
    }

    // Allow optional separator after body before 'done'
    loop {
        match parser.lexer.peek() {
            Some(Token::Space) | Some(Token::Tab) | Some(Token::Comment) | Some(Token::Newline | Token::CarriageReturn) => {
                parser.lexer.next();
                continue;
            }
            Some(Token::Semicolon) => {
                parser.lexer.next();
                // consume any following whitespace/newlines as well
                continue;
            }
            _ => {}
        }
        break;
    }

    parser.lexer.consume(Token::Done)?;
    
    // Skip whitespace after 'done' before checking for pipe
    parser.lexer.skip_whitespace_and_comments();
    
    // Check if there's a pipeline after the for loop
    let mut final_command = Command::For(ForLoop {
        variable,
        items,
        body: Block { commands: body_commands },
    });
    
    // If there's a pipe after 'done', parse the pipeline
    if let Some(Token::Pipe) = parser.lexer.peek() {
        final_command = parse_pipeline_from_command(&mut parser.lexer, final_command)?;
    }
    
    Ok(final_command)
}

pub fn parse_function(parser: &mut Parser) -> Result<Command, ParserError> {
    parser.lexer.consume(Token::Function)?;
    // Allow whitespace between 'function' and name
    parser.lexer.skip_whitespace_and_comments();

    let name = match parser.lexer.peek() {
        Some(Token::Identifier) => parser.lexer.get_identifier_text()?,
        Some(t) => {
            let (line, col) = parser.lexer.offset_to_line_col(0);
            return Err(ParserError::UnexpectedToken { token: t.clone(), line, col });
        }
        None => return Err(ParserError::UnexpectedEOF),
    };

    // Skip whitespace after name
    parser.lexer.skip_whitespace_and_comments();

    // Parse parameters if present: function name(param1, param2)
    let mut parameters = Vec::new();
    if let Some(Token::ParenOpen) = parser.lexer.peek() {
        // Consume opening parenthesis
        parser.lexer.next();
        
        // Parse parameters until closing parenthesis
        loop {
            parser.lexer.skip_whitespace_and_comments();
            
            match parser.lexer.peek() {
                Some(Token::ParenClose) => {
                    parser.lexer.next(); // consume closing parenthesis
                    break;
                }
                Some(Token::Identifier) => {
                    let param = parser.lexer.get_identifier_text()?;
                    parameters.push(param);
                    
                    // Check for comma separator
                    parser.lexer.skip_whitespace_and_comments();
                    if let Some(Token::Comma) = parser.lexer.peek() {
                        parser.lexer.next(); // consume comma
                    } else if let Some(Token::ParenClose) = parser.lexer.peek() {
                        // No comma, must be last parameter
                        continue;
                    } else {
                        // Expect comma or closing parenthesis
                        let (line, col) = parser.lexer.offset_to_line_col(0);
                        return Err(ParserError::UnexpectedToken { 
                            token: parser.lexer.peek().unwrap().clone(), 
                            line, 
                            col 
                        });
                    }
                }
                _ => {
                    let (line, col) = parser.lexer.offset_to_line_col(0);
                    return Err(ParserError::UnexpectedToken { 
                        token: parser.lexer.peek().unwrap().clone(), 
                        line, 
                        col 
                    });
                }
            }
        }
        
        // Skip whitespace/newlines after parentheses
        while matches!(parser.lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline)) {
            parser.lexer.next();
        }
    }

    // Brace-wrapped function body: { ... }
    let body = if let Some(Token::BraceOpen) = parser.lexer.peek() {
        // Consume '{'
        parser.lexer.next();
        // Allow whitespace/newlines
        while matches!(parser.lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline)) {
            parser.lexer.next();
        }
        
        // Parse body commands into a Block
        let mut body_commands = Vec::new();
        
        // Parse first command
        body_commands.push(parser.parse_command()?);

        // Parse additional commands inside the block
        loop {
            // Skip separators
            while matches!(parser.lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline | Token::Semicolon)) {
                parser.lexer.next();
            }
            match parser.lexer.peek() {
                Some(Token::BraceClose) | None => break,
                _ => {
                    let pre_pos = parser.lexer.current_position();
                    let command = parser.parse_command()?;
                    body_commands.push(command);
                    if parser.lexer.current_position() == pre_pos {
                        if parser.lexer.next().is_none() { break; }
                    }
                }
            }
        }

        // Expect closing '}'
        parser.lexer.consume(Token::BraceClose)?;
        Block { commands: body_commands }
    } else {
        // Fallback: parse next as a single command body
        let command = parser.parse_command()?;
        Block { commands: vec![command] }
    };
    
    Ok(Command::Function(Function { name, parameters, body }))
}

pub fn parse_posix_function(parser: &mut Parser) -> Result<Command, ParserError> {
    // Get the function name
    let name = parser.lexer.get_identifier_text()?;
    
    // Consume the opening parenthesis
    parser.lexer.consume(Token::ParenOpen)?;
    
    // Parse parameters if present: name(param1, param2)
    let mut parameters = Vec::new();
    if let Some(Token::ParenClose) = parser.lexer.peek() {
        // No parameters, just consume closing parenthesis
        parser.lexer.next();
    } else {
        // Parse parameters until closing parenthesis
        loop {
            parser.lexer.skip_whitespace_and_comments();
            
            match parser.lexer.peek() {
                Some(Token::ParenClose) => {
                    parser.lexer.next(); // consume closing parenthesis
                    break;
                }
                Some(Token::Identifier) => {
                    let param = parser.lexer.get_identifier_text()?;
                    parameters.push(param);
                    
                    // Check for comma separator
                    parser.lexer.skip_whitespace_and_comments();
                    if let Some(Token::Comma) = parser.lexer.peek() {
                        parser.lexer.next(); // consume comma
                    } else if let Some(Token::ParenClose) = parser.lexer.peek() {
                        // No comma, must be last parameter
                        continue;
                    } else {
                        // Expect comma or closing parenthesis
                        let (line, col) = parser.lexer.offset_to_line_col(0);
                        return Err(ParserError::UnexpectedToken { 
                            token: parser.lexer.peek().unwrap().clone(), 
                            line, 
                            col 
                        });
                    }
                }
                _ => {
                    let (line, col) = parser.lexer.offset_to_line_col(0);
                    return Err(ParserError::UnexpectedToken { 
                        token: parser.lexer.peek().unwrap().clone(), 
                        line, 
                        col 
                    });
                }
            }
        }
    }
    
    // Skip whitespace/newlines after parentheses
    while matches!(parser.lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline)) {
        parser.lexer.next();
    }
    
    // Consume the opening brace
    parser.lexer.consume(Token::BraceOpen)?;
    
    // Allow whitespace/newlines after opening brace
    while matches!(parser.lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline)) {
        parser.lexer.next();
    }
    
    // Parse the function body as a block of commands
    let mut body_commands = Vec::new();
    
    // Parse commands until we find the closing brace
    loop {
        // Skip separators
        while matches!(parser.lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline | Token::Semicolon)) {
            parser.lexer.next();
        }
        
        match parser.lexer.peek() {
            Some(Token::BraceClose) => {
                parser.lexer.next(); // consume the closing brace
                break;
            }
            None => {
                return Err(ParserError::UnexpectedEOF);
            }
            _ => {
                // Parse the next command
                let command = parser.parse_command()?;
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

pub fn parse_block(parser: &mut Parser) -> Result<Command, ParserError> {
    // Parse a standalone block: { ... }
    
    // Consume the opening brace
    parser.lexer.consume(Token::BraceOpen)?;
    
    // Allow whitespace/newlines after opening brace
    while matches!(parser.lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline)) {
        parser.lexer.next();
    }
    
    // Parse the block body as a list of commands
    let mut body_commands = Vec::new();
    
    // Parse commands until we find the closing brace
    loop {
        // Skip separators
        while matches!(parser.lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline | Token::Semicolon)) {
            parser.lexer.next();
        }
        
        match parser.lexer.peek() {
            Some(Token::BraceClose) => {
                parser.lexer.next(); // consume the closing brace
                break;
            }
            None => {
                return Err(ParserError::UnexpectedEOF);
            }
            _ => {
                // Parse the next command
                let command = parser.parse_command()?;
                body_commands.push(command);
            }
        }
    }
    
    Ok(Command::Block(Block { commands: body_commands }))
}

pub fn parse_break_statement(parser: &mut Parser) -> Result<Command, ParserError> {
    parser.lexer.consume(Token::Break)?;
    
    // Optional argument (loop level)
    let mut level = None;
    parser.lexer.skip_whitespace_and_comments();
    
    if let Some(Token::Number) = parser.lexer.peek() {
        let level_text = parser.lexer.get_number_text()?;
        level = Some(level_text);
    }
    
    Ok(Command::Break(level))
}

pub fn parse_continue_statement(parser: &mut Parser) -> Result<Command, ParserError> {
    parser.lexer.consume(Token::Continue)?;
    
    // Optional argument (loop level)
    let mut level = None;
    parser.lexer.skip_whitespace_and_comments();
    
    if let Some(Token::Number) = parser.lexer.peek() {
        let level_text = parser.lexer.get_number_text()?;
        level = Some(level_text);
    }
    
    Ok(Command::Continue(level))
}

pub fn parse_return_statement(parser: &mut Parser) -> Result<Command, ParserError> {
    parser.lexer.consume(Token::Return)?;
    
    // Optional return value
    let mut return_value = None;
    parser.lexer.skip_whitespace_and_comments();
    
    if !parser.lexer.is_eof() && !matches!(parser.lexer.peek(), Some(Token::Newline | Token::Semicolon | Token::CarriageReturn)) {
        // Parse the return value as a word
        return_value = Some(parse_word(&mut parser.lexer)?);
    }
    
    Ok(Command::Return(return_value))
}

// Placeholder functions - these would need to be implemented based on the actual AST structures
fn parse_arithmetic_expression(parser: &mut Parser) -> Result<Word, ParserError> {
    // Handle arithmetic expressions like $((i + 1))
    // First, consume the opening $(( or $( token
    match parser.lexer.peek() {
        Some(Token::Arithmetic) | Some(Token::ArithmeticEval) => {
            parser.lexer.next(); // consume $(( or $(
        }
        _ => {
            return Err(ParserError::InvalidSyntax("Expected arithmetic expression start".to_string()));
        }
    }
    
    let mut expression_parts = Vec::new();
    
    // Simple case: just parse until we find the closing ))
    loop {
        match parser.lexer.peek() {
            Some(Token::ArithmeticEvalClose) => {
                // Found the closing )), consume it and break
                parser.lexer.next();
                break;
            }
            Some(Token::Identifier) => {
                // Variable reference like 'i'
                let var_name = parser.lexer.get_identifier_text()?;
                expression_parts.push(var_name);
                parser.lexer.next(); // consume the identifier token
            }
            Some(Token::Number) => {
                // Number like '1'
                let num = parser.lexer.get_number_text()?;
                expression_parts.push(num);
                parser.lexer.next(); // consume the number token
            }
            Some(Token::Plus) => {
                // Plus operator
                parser.lexer.next();
                expression_parts.push("+".to_string());
            }
            Some(Token::Minus) => {
                // Minus operator
                parser.lexer.next();
                expression_parts.push("-".to_string());
            }
            Some(Token::Star) => {
                // Multiplication operator
                parser.lexer.next();
                expression_parts.push("*".to_string());
            }
            Some(Token::Slash) => {
                // Division operator
                parser.lexer.next();
                expression_parts.push("/".to_string());
            }
            Some(Token::Space) | Some(Token::Tab) => {
                // Skip whitespace
                parser.lexer.next();
            }
            Some(Token::Dollar) => {
                // Handle variable references like $i
                parser.lexer.next();
                if let Some(Token::Identifier) = parser.lexer.peek() {
                    let var_name = parser.lexer.get_identifier_text()?;
                    expression_parts.push(format!("${}", var_name));
                } else {
                    return Err(ParserError::InvalidSyntax("Expected identifier after $ in arithmetic expression".to_string()));
                }
            }
            None => {
                return Err(ParserError::UnexpectedEOF);
            }
            _ => {
                // For any other token, just consume it and add its text
                if let Some(text) = parser.lexer.get_current_text() {
                    expression_parts.push(text);
                    parser.lexer.next();
                } else {
                    break;
                }
            }
        }
    }
    
    let expression = expression_parts.join(" ");
    
    Ok(Word::Arithmetic(ArithmeticExpression {
        expression,
        tokens: vec![], // TODO: Store actual tokens if needed
    }))
}

fn parse_pipeline(parser: &mut Parser) -> Result<Command, ParserError> {
    // For control flow constructs, we only need to parse a single command
    // This is used for test conditions in if statements, not for general pipelines
    parse_simple_command(parser)
}

pub fn parse_simple_command(parser: &mut Parser) -> Result<Command, ParserError> {
    // Skip whitespace and comments at the beginning
    parser.lexer.skip_whitespace_and_comments();
    
    // Check if this is a test expression first
    if matches!(parser.lexer.peek(), Some(Token::TestBracket)) {
        return parse_test_expression(&mut parser.lexer);
    }
    
    let mut args = Vec::new();
    let mut redirects = Vec::new();
    let mut env_vars = HashMap::new();
    
    // Parse the command name
    let name = match parser.lexer.peek() {
        Some(Token::Identifier) => {
            let name_text = parser.lexer.get_identifier_text()?;
            Word::Literal(name_text)
        }
        _ => {
            return Err(ParserError::InvalidSyntax("Expected command name".to_string()));
        }
    };
    
    // Parse arguments
    parser.lexer.skip_whitespace_and_comments();
    while let Some(token) = parser.lexer.peek() {
        match token {
            Token::Identifier | Token::DoubleQuotedString | Token::SingleQuotedString | 
            Token::Dollar | Token::DollarParen | Token::BacktickString |
            Token::File | Token::Directory | Token::Exists | Token::Readable | Token::Writable | 
            Token::Executable | Token::Size | Token::Symlink => {
                let word = parse_word(&mut parser.lexer)?;
                args.push(word);
                // Skip whitespace after the word
                parser.lexer.skip_whitespace_and_comments();
            }
            _ => break,
        }
    }
    
    // For now, skip redirects as they're not needed for basic control flow parsing
    // TODO: Implement redirect parsing if needed
    
    Ok(Command::Simple(SimpleCommand {
        name,
        args,
        redirects,
        env_vars,
    }))
}

fn parse_command(parser: &mut Parser) -> Result<Command, ParserError> {
    // Skip whitespace and comments
    parser.lexer.skip_whitespace_and_comments();
    
    if parser.lexer.is_eof() {
        return Err(ParserError::UnexpectedEOF);
    }
    
    // Check if this is a test expression
    if matches!(parser.lexer.peek(), Some(Token::TestBracket)) {
        parse_test_expression(&mut parser.lexer)
    } else if matches!(parser.lexer.peek(), Some(Token::Identifier)) {
        // Check if this is a standalone variable assignment: identifier=value
        let mut pos = 1;
        while pos < 10 && matches!(parser.lexer.peek_n(pos), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline)) {
            pos += 1;
        }
        if matches!(parser.lexer.peek_n(pos), Some(Token::Assign | Token::PlusAssign | Token::MinusAssign | Token::StarAssign | Token::SlashAssign | Token::PercentAssign)) {
            parse_assignment(parser)
        } else {
            parse_simple_command(parser)
        }
    } else {
        parse_simple_command(parser)
    }
}

fn parse_test_expression(lexer: &mut Lexer) -> Result<Command, ParserError> {
    use crate::ast::{TestExpression, TestModifiers};
    
    // Consume the opening [
    if !matches!(lexer.peek(), Some(Token::TestBracket)) {
        return Err(ParserError::InvalidSyntax("Expected '[' for test expression".to_string()));
    }
    lexer.next(); // consume '['
    
    // Capture the content between [ and ]
    let mut expression_parts = Vec::new();
    
    loop {
        match lexer.peek() {
            Some(Token::TestBracketClose) => {
                lexer.next(); // consume ']'
                break;
            }
            Some(Token::File) => {
                expression_parts.push("-f".to_string());
                lexer.next();
            }
            Some(Token::Directory) => {
                expression_parts.push("-d".to_string());
                lexer.next();
            }
            Some(Token::Exists) => {
                expression_parts.push("-e".to_string());
                lexer.next();
            }
            Some(Token::Readable) => {
                expression_parts.push("-r".to_string());
                lexer.next();
            }
            Some(Token::Writable) => {
                expression_parts.push("-w".to_string());
                lexer.next();
            }
            Some(Token::Executable) => {
                expression_parts.push("-x".to_string());
                lexer.next();
            }
            Some(Token::Size) => {
                expression_parts.push("-s".to_string());
                lexer.next();
            }
            Some(Token::Symlink) => {
                expression_parts.push("-L".to_string());
                lexer.next();
            }
            Some(Token::Identifier) => {
                expression_parts.push(lexer.get_identifier_text()?);
            }
            Some(Token::DoubleQuotedString) | Some(Token::SingleQuotedString) => {
                expression_parts.push(lexer.get_string_text()?);
            }
            Some(Token::Dollar) => {
                // Handle variable references like $i
                lexer.next(); // consume $
                if let Some(Token::Identifier) = lexer.peek() {
                    let var_name = lexer.get_identifier_text()?;
                    expression_parts.push(format!("${}", var_name));
                } else {
                    return Err(ParserError::InvalidSyntax("Expected identifier after $ in test expression".to_string()));
                }
            }
            Some(Token::Number) => {
                expression_parts.push(lexer.get_number_text()?);
            }
            Some(Token::Lt) => {
                expression_parts.push("-lt".to_string());
                lexer.next();
            }
            Some(Token::Le) => {
                expression_parts.push("-le".to_string());
                lexer.next();
            }
            Some(Token::Gt) => {
                expression_parts.push("-gt".to_string());
                lexer.next();
            }
            Some(Token::Ge) => {
                expression_parts.push("-ge".to_string());
                lexer.next();
            }
            Some(Token::Eq) => {
                expression_parts.push("-eq".to_string());
                lexer.next();
            }
            Some(Token::Ne) => {
                expression_parts.push("-ne".to_string());
                lexer.next();
            }
            Some(Token::Space) | Some(Token::Tab) => {
                lexer.next(); // skip whitespace
            }
            None => {
                return Err(ParserError::InvalidSyntax("Unexpected end of input in test expression".to_string()));
            }
            _ => {
                return Err(ParserError::InvalidSyntax("Unexpected token in test expression".to_string()));
            }
        }
    }
    
    let expression = expression_parts.join(" ");
    
    Ok(Command::TestExpression(TestExpression {
        expression,
        modifiers: TestModifiers {
            extglob: false,
            nocasematch: false,
            globstar: false,
            nullglob: false,
            failglob: false,
            dotglob: false,
        },
    }))
}

fn parse_assignment(parser: &mut Parser) -> Result<Command, ParserError> {
    // Parse a standalone assignment like: var=value or var=$((expr))
    let var_name = parser.lexer.get_identifier_text()?;
    
    // Skip whitespace before assignment operator
    parser.lexer.skip_whitespace_and_comments();
    
    // Consume the assignment operator
    let _operator = parser.lexer.next();
    
    // Skip whitespace after assignment operator
    parser.lexer.skip_whitespace_and_comments();
    
    // Parse the value
    let value = parse_word(&mut parser.lexer)?;
    
    // Create a simple command that represents the assignment
    let mut env_vars = HashMap::new();
    env_vars.insert(var_name.clone(), value);
    
    Ok(Command::Simple(SimpleCommand {
        name: Word::Literal("assignment".to_string()), // Placeholder name
        args: Vec::new(),
        redirects: Vec::new(),
        env_vars,
    }))
}

fn parse_pipeline_from_command(_lexer: &mut Lexer, _command: Command) -> Result<Command, ParserError> {
    // TODO: Implement pipeline from command parsing
    Err(ParserError::InvalidSyntax("Pipeline from command parsing not yet implemented".to_string()))
}
