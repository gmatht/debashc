use crate::ast::*;
use crate::lexer::{Lexer, Token};
use crate::parser::errors::ParserError;
use crate::parser::utilities::ParserUtilities;
use crate::parser::words::parse_word;
use crate::parser::redirects::parse_redirect;
use crate::parser::assignments::parse_array_elements;
use crate::parser::control_flow::{
    parse_if_statement, parse_case_statement, parse_while_loop, parse_for_loop,
    parse_function, parse_posix_function, parse_break_statement, parse_continue_statement,
    parse_return_statement, parse_block
};
use std::collections::HashMap;

pub struct Parser {
    pub lexer: Lexer,
    shopt_state: TestModifiers,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        Self {
            lexer: Lexer::new(input),
            shopt_state: TestModifiers::default(),
        }
    }

    pub fn new_with_lexer(lexer: Lexer) -> Self {
        Self {
            lexer,
            shopt_state: TestModifiers::default(),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Command>, ParserError> {
        let mut commands = Vec::new();
        
        // Skip initial whitespace and comments
        self.lexer.skip_whitespace_and_comments();
        
        // If the file contains only comments and whitespace, return empty commands
        if self.lexer.is_eof() {
            return Ok(commands);
        }
        
        while !self.lexer.is_eof() {
            // Progress guard to prevent hangs: ensure each loop consumes or advances tokens
            let loop_start_pos = self.lexer.current_position();
            
            // Check if we're at EOF after skipping whitespace and comments
            if self.lexer.is_eof() {
                break;
            }
            
            // Check if we're at a newline/semicolon/& (empty command or separator)
            if let Some(token) = self.lexer.peek() {
                match token {
                    Token::Newline => {
                        // Consume consecutive newlines
                        let mut count = 0usize;
                        while matches!(self.lexer.peek(), Some(Token::Newline)) {
                            self.lexer.next();
                            count += 1;
                        }
                        // If two or more, record a blank line in AST
                        if count >= 2 {
                            commands.push(Command::BlankLine);
                        }
                        self.lexer.skip_whitespace_and_comments();
                        continue;
                    }
                    Token::Semicolon | Token::CarriageReturn | Token::Background => {
                        self.lexer.next();
                        self.lexer.skip_whitespace_and_comments();
                        continue;
                    }
                    _ => {}
                }
            }
            
            // Handle comments at the top level
            while let Some(Token::Comment) = self.lexer.peek() {
                self.lexer.next();
                self.lexer.skip_whitespace_and_comments();
                if self.lexer.is_eof() {
                    break;
                }
            }
            
            if self.lexer.is_eof() {
                break;
            }
            
            let mut command = self.parse_command()?;

            // Check if this command is followed by a pipeline or logical operator
            if let Some(token) = self.lexer.peek() {
                match token {
                    Token::Pipe => {
                        command = self.parse_pipeline_from_command(command)?;
                    }
                    Token::Or | Token::And => {
                        command = self.parse_pipeline_from_command(command)?;
                    }
                    _ => {}
                }
            }

            // If a background '&' follows immediately, wrap the command
            if let Some(Token::Background) = self.lexer.peek() {
                self.lexer.next();
                command = Command::Background(Box::new(command));
            }

            commands.push(command);
            
            // Handle semicolons, newlines, and background '&'
            let mut newline_count = 0usize;
            while let Some(token) = self.lexer.peek() {
                match token {
                    Token::Semicolon | Token::Background => {
                        self.lexer.next();
                    }
                    Token::Newline => {
                        self.lexer.next();
                        newline_count += 1;
                    }
                    _ => break,
                }
            }
            if newline_count >= 2 {
                commands.push(Command::BlankLine);
            }
            
            // Skip whitespace and comments before next command
            self.lexer.skip_whitespace_and_comments();

            // If no progress was made in this iteration, advance by one token to avoid infinite loop
            let loop_end_pos = self.lexer.current_position();
            if loop_end_pos == loop_start_pos {
                // Consume one token defensively. If already EOF, break.
                if self.lexer.next().is_none() {
                    break;
                }
            }
        }
        
        Ok(commands)
    }

    pub fn parse_command(&mut self) -> Result<Command, ParserError> {
        // Skip whitespace and comments
        self.lexer.skip_whitespace_and_comments();
        
        if self.lexer.is_eof() {
            return Err(ParserError::UnexpectedEOF);
        }

        let command = if let Some(Token::Identifier) = self.lexer.peek() {
            // Check if this is a function definition: identifier() { ... }
            let paren1 = self.lexer.peek_n(1);
            let paren2 = self.lexer.peek_n(2);
            if matches!(paren1, Some(Token::ParenOpen)) 
                && matches!(paren2, Some(Token::ParenClose)) {
                // Check if the next non-whitespace token is a brace
                let mut pos = 3;
                while pos < 10 && matches!(self.lexer.peek_n(pos), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline)) {
                    pos += 1;
                }
                let brace_token = self.lexer.peek_n(pos);
                if matches!(brace_token, Some(Token::BraceOpen)) {
                    parse_posix_function(self)?
                } else {
                    self.parse_pipeline()?
                }
            } else {
                // Check if this is an array assignment: identifier=(...)
                if matches!(paren1, Some(Token::Assign)) && matches!(self.lexer.peek_n(2), Some(Token::ParenOpen)) {
                                    // This is an array assignment, parse it directly
                let var_name = self.lexer.get_identifier_text()?;
                // get_identifier_text() already advanced past the identifier
                self.lexer.next(); // consume the =
                self.lexer.next(); // consume the (
                // Now parse the array elements (the lexer should be at the first element)
                let elements = parse_array_elements(&mut self.lexer)?;
                // The closing ) should already be consumed by parse_array_elements
                    
                    // Create a simple command with environment variables
                    let mut env_vars = HashMap::new();
                    env_vars.insert(var_name.clone(), Word::Array(var_name, elements));
                    
                    Command::Simple(SimpleCommand {
                        name: Word::Literal("true".to_string()),
                        args: Vec::new(),
                        redirects: Vec::new(),
                        env_vars,
                    })
                } else {
                    // Check if this is a standalone variable assignment: identifier=value
                    let mut pos = 1;
                    while pos < 10 && matches!(self.lexer.peek_n(pos), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline)) {
                        pos += 1;
                    }
                    if matches!(self.lexer.peek_n(pos), Some(Token::Assign | Token::PlusAssign | Token::MinusAssign | Token::StarAssign | Token::SlashAssign | Token::PercentAssign)) {
                        self.parse_standalone_assignment()?
                    } else {
                        self.parse_pipeline()?
                    }
                }
            }
        } else {
            match self.lexer.peek() {
                Some(Token::Comment) => {
                    // Comments should be handled at the top level
                    return Err(ParserError::InvalidSyntax("Unexpected comment in command parsing".to_string()));
                }
                Some(Token::If) => parse_if_statement(self)?,
                Some(Token::Case) => parse_case_statement(self)?,
                Some(Token::While) => parse_while_loop(self)?,
                Some(Token::For) => parse_for_loop(self)?,
                Some(Token::Function) => parse_function(self)?,
                Some(Token::Break) => parse_break_statement(self)?,
                Some(Token::Continue) => parse_continue_statement(self)?,
                Some(Token::Return) => parse_return_statement(self)?,
                // Bash arithmetic evaluation: (( ... ))
                Some(Token::ParenOpen) if matches!(self.lexer.peek_n(1), Some(Token::ParenOpen)) => {
                    self.parse_double_paren_command()?
                }
                Some(Token::ParenOpen) => self.parse_subshell()?,
                Some(Token::BraceOpen) => parse_block(self)?,
                Some(Token::TestBracket) => self.parse_test_expression()?,
                Some(Token::Semicolon) | Some(Token::Newline) | Some(Token::CarriageReturn) => {
                    // Skip semicolon and continue parsing
                    self.lexer.next();
                    self.parse_command()?
                }
                _ => self.parse_pipeline()?,
            }
        };

        // Check for redirects that follow the command
        self.parse_command_redirects(command)
    }

    fn parse_command_redirects(&mut self, command: Command) -> Result<Command, ParserError> {
        // Check if there are redirects following the command
        let mut redirects = Vec::new();
        
        // Skip whitespace and comments
        self.lexer.skip_whitespace_and_comments();
        
        // Parse redirects until we hit a command separator or other non-redirect token
        while let Some(token) = self.lexer.peek() {
            match token {
                Token::Number | Token::RedirectIn | Token::RedirectOut | Token::RedirectAppend | 
                Token::RedirectInOut | Token::Heredoc | Token::HeredocTabs | Token::HereString |
                Token::RedirectOutErr | Token::RedirectInErr | Token::RedirectOutClobber | 
                Token::RedirectAll | Token::RedirectAllAppend => {
                    redirects.push(parse_redirect(&mut self.lexer)?);
                }
                _ => break,
            }
        }
        
        if redirects.is_empty() {
            Ok(command)
        } else {
            // Wrap the command with redirects
            Ok(Command::Redirect(RedirectCommand {
                command: Box::new(command),
                redirects,
            }))
        }
    }

    fn parse_pipeline(&mut self) -> Result<Command, ParserError> {
        let first_command = self.parse_simple_command()?;
        self.parse_pipeline_from_command(first_command)
    }

    fn parse_pipeline_from_command(&mut self, first_command: Command) -> Result<Command, ParserError> {
        let mut commands = Vec::new();
        let mut operators = Vec::new();
        
        commands.push(first_command);
        
        while let Some(_) = self.lexer.peek() {
            // Skip any whitespace/comments before checking for an operator
            self.lexer.skip_whitespace_and_comments();
            let Some(token) = self.lexer.peek() else { break; };
            match token {
                Token::Pipe => {
                    self.lexer.next();
                    operators.push(PipeOperator::Pipe);
                    self.lexer.skip_whitespace_and_comments();
                    commands.push(self.parse_simple_command()?);
                }
                Token::And => {
                    self.lexer.next();
                    operators.push(PipeOperator::And);
                    self.lexer.skip_whitespace_and_comments();
                    commands.push(self.parse_simple_command()?);
                }
                Token::Or => {
                    self.lexer.next();
                    operators.push(PipeOperator::Or);
                    self.lexer.skip_whitespace_and_comments();
                    commands.push(self.parse_simple_command()?);
                }
                Token::Semicolon | Token::Newline => {
                    // Stop parsing pipeline when we hit a command separator
                    break;
                }
                _ => {
                    break;
                }
            }
        }
        
        if commands.len() == 1 {
            Ok(commands.remove(0))
        } else {
            Ok(Command::Pipeline(Pipeline { commands, operators }))
        }
    }

    fn parse_simple_command(&mut self) -> Result<Command, ParserError> {
        // Skip whitespace and comments at the beginning
        self.lexer.skip_whitespace_and_comments();
        
        let mut args = Vec::new();
        let mut redirects = Vec::new();
        let mut env_vars = HashMap::new();
        
        // Parse environment variable-style assignments at the start
        while let Some(token) = self.lexer.peek() {
            match token {
                Token::Identifier => {
                    // Handle compound assignment operators
                    let compound_op = self.lexer.peek_n(1).cloned();
                    if let Some(compound_op) = compound_op {
                        match compound_op {
                            Token::PlusAssign => {
                                // Handle array append (var+=(...)) or compound assignment (var+=value)
                                if matches!(self.lexer.peek_n(2), Some(Token::ParenOpen)) {
                                    // Handle array append like: var+=(value)
                                    let var_name = self.lexer.get_identifier_text()?;
                                    self.lexer.next(); // consume +=
                                    self.lexer.next(); // consume (
                                    let elements = parse_array_elements(&mut self.lexer)?;
                                    
                                    let array_word = Word::Array(var_name.clone(), elements);
                                    env_vars.insert(var_name, array_word);
                                    self.lexer.skip_whitespace_and_comments();
                                } else {
                                    // Handle compound assignment like: var+=value
                                    let var_name = self.lexer.get_identifier_text()?;
                                    self.lexer.next(); // consume +=
                                    
                                    let value_word = parse_word(&mut self.lexer)?;
                                    
                                    let arithmetic_expr = format!("{}+{}", var_name, value_word.to_string());
                                    let compound_word = Word::Arithmetic(ArithmeticExpression {
                                        expression: arithmetic_expr,
                                        tokens: vec![],
                                    });
                                    
                                    env_vars.insert(var_name, compound_word);
                                    self.lexer.skip_whitespace_and_comments();
                                }
                            }
                            Token::Assign => {
                                if matches!(self.lexer.peek_n(2), Some(Token::ParenOpen)) {
                                    // Handle array declaration like: arr=(one two three)
                                    let var_name = self.lexer.get_identifier_text()?;
                                    self.lexer.next(); // consume =
                                    self.lexer.next(); // consume (
                                    let elements = parse_array_elements(&mut self.lexer)?;
                                    let array_word = Word::Array(var_name.clone(), elements);
                                    env_vars.insert(var_name, array_word);
                                    self.lexer.skip_whitespace_and_comments();
                                } else {
                                    // Handle regular assignment like: var=value
                                    let var_name = self.lexer.get_identifier_text()?;
                                    self.lexer.next(); // consume =
                                    let value_word = parse_word(&mut self.lexer)?;
                                    env_vars.insert(var_name, value_word);
                                    self.lexer.skip_whitespace_and_comments();
                                }
                            }
                            _ => {
                                break;
                            }
                        }
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
        
        // Parse command name
        let mut is_double_bracket = false;
        let name = if let Some(token) = self.lexer.peek() {
            match token {
                Token::Identifier => {
                    let name = self.lexer.get_identifier_text()?;
                    self.lexer.next(); // consume the identifier
                    Word::Literal(name)
                }
                Token::Set | Token::Export | Token::Readonly | Token::Local | Token::Declare | Token::Typeset |
                Token::Unset | Token::Shift | Token::Eval | Token::Exec | Token::Source | Token::Trap | Token::Wait | Token::Shopt | Token::Exit => {
                    Word::Literal(self.lexer.get_raw_token_text()?)
                }
                Token::TestBracket => {
                    self.lexer.next(); // consume the first [
                    if let Some(Token::TestBracket) = self.lexer.peek() {
                        self.lexer.next(); // consume the second [
                        is_double_bracket = true;
                        Word::Literal("[[".to_string())
                    } else {
                        Word::Literal("[".to_string())
                    }
                }
                Token::True => {
                    self.lexer.next(); // consume true
                    Word::Literal("true".to_string())
                }
                Token::False => {
                    self.lexer.next(); // consume false
                    Word::Literal("false".to_string())
                }
                Token::Dollar | Token::DollarBrace | Token::DollarParen
                | Token::DollarBraceHash | Token::DollarBraceBang | Token::DollarBraceStar | Token::DollarBraceAt
                | Token::DollarBraceHashStar | Token::DollarBraceHashAt | Token::DollarBraceBangStar | Token::DollarBraceBangAt => {
                    self.parse_variable_expansion()?
                }
                Token::Arithmetic => {
                    self.parse_arithmetic_expression()?
                }

                _ => {
                    let (line, col) = self.lexer.offset_to_line_col(0);
                    return Err(ParserError::UnexpectedToken { token: token.to_owned(), line, col });
                }
            }
        } else {
            return Err(ParserError::UnexpectedEOF);
        };
        
        // Skip inline whitespace before parsing arguments (but stop at newlines)
        self.lexer.skip_inline_whitespace_and_comments();
        
        // Special handling for Bash double-bracket test: capture everything until closing ']]'
        if is_double_bracket {
            let expr = self.lexer.capture_double_bracket_expression()?;
            return Ok(Command::TestExpression(TestExpression {
                expression: expr,
                modifiers: self.get_current_shopt_state(),
            }));
        }

        // Special handling for Bash single-bracket test: capture everything until closing ']'
        if let Word::Literal(name_str) = &name {
            if name_str == "[" {
                let expr = self.lexer.capture_single_bracket_expression()?;
                args.push(Word::Literal(expr));
            }
        }

        // Parse arguments
        while let Some(token) = self.lexer.peek() {
            match token {
                Token::Space | Token::Tab | Token::Comment | Token::Newline | Token::CarriageReturn => {
                    break;
                }
                Token::RedirectIn | Token::RedirectOut | Token::RedirectAppend | Token::RedirectInErr | Token::RedirectOutErr | Token::RedirectInOut | Token::Heredoc | Token::HeredocTabs | Token::HereString => {
                    break;
                }
                Token::Pipe | Token::And | Token::Or | Token::Semicolon | Token::Background => {
                    break;
                }
                Token::Character | Token::NonZero | Token::Exists | Token::File | Token::Size | Token::Readable | Token::Writable | Token::Executable | Token::NewerThan | Token::OlderThan => {
                    // These are valid argument tokens
                    args.push(parse_word(&mut self.lexer)?);
                }
                _ => {
                    args.push(parse_word(&mut self.lexer)?);
                }
            }
        }
        
        Ok(Command::Simple(SimpleCommand {
            name,
            args,
            redirects,
            env_vars,
        }))
    }

    fn parse_standalone_assignment(&mut self) -> Result<Command, ParserError> {
        // Get the variable name
        let var_name = self.lexer.get_identifier_text()?;
        
        // Consume the assignment token (=, +=, -=, etc.)
        let assignment_op = self.lexer.peek().cloned().unwrap();
        match assignment_op {
            Token::Assign | Token::PlusAssign | Token::MinusAssign | Token::StarAssign | Token::SlashAssign | Token::PercentAssign => {
                self.lexer.next();
            }
            _ => return Err(ParserError::InvalidSyntax("Expected assignment operator".to_string())),
        }
        
        // Parse the value
        let value_word = parse_word(&mut self.lexer)?;
        
        // Check if there's a command following this assignment
        self.lexer.skip_whitespace_and_comments();
        if let Some(Token::Identifier) = self.lexer.peek() {
            // There's a command following, parse it as a command with environment variables
            let mut env_vars = HashMap::new();
            env_vars.insert(var_name, value_word);
            
            let command = self.parse_command()?;
            
            // Merge the environment variables with the command's environment variables
            match command {
                Command::Simple(mut simple_cmd) => {
                    // Merge environment variables
                    for (key, value) in env_vars {
                        simple_cmd.env_vars.insert(key, value);
                    }
                    Ok(Command::Simple(simple_cmd))
                }
                _ => {
                    // For non-simple commands, wrap in a block with environment variables
                    let mut env_vars_cmd = HashMap::new();
                    for (key, value) in env_vars {
                        env_vars_cmd.insert(key, value);
                    }
                    
                    let env_cmd = Command::Simple(SimpleCommand {
                        name: Word::Literal("true".to_string()),
                        args: Vec::new(),
                        redirects: Vec::new(),
                        env_vars: env_vars_cmd,
                    });
                    
                    Ok(Command::Block(Block {
                        commands: vec![env_cmd, command],
                    }))
                }
            }
        } else {
            // No command following, this is a standalone assignment
            let mut env_vars = HashMap::new();
            env_vars.insert(var_name, value_word);
            
            Ok(Command::Simple(SimpleCommand {
                name: Word::Literal("true".to_string()), // Use 'true' as a dummy command
                args: Vec::new(),
                redirects: Vec::new(),
                env_vars,
            }))
        }
    }

    fn parse_subshell(&mut self) -> Result<Command, ParserError> {
        self.lexer.consume(Token::ParenOpen)?;
        
        // Parse one or more commands until ')'
        let mut commands = Vec::new();
        loop {
            // Skip separators within subshell body
            while matches!(
                self.lexer.peek(),
                Some(Token::Space | Token::Tab | Token::Comment | Token::Newline | Token::Semicolon | Token::CarriageReturn)
            ) {
                self.lexer.next();
            }
            match self.lexer.peek() {
                Some(Token::ParenClose) | None => break,
                _ => {
                    let mut cmd = self.parse_command()?;
                    // Background marker inside subshell
                    if let Some(Token::Background) = self.lexer.peek() {
                        self.lexer.next();
                        cmd = Command::Background(Box::new(cmd));
                    }
                    commands.push(cmd);
                }
            }
        }

        self.lexer.consume(Token::ParenClose)?;
        
        if commands.len() == 1 {
            Ok(Command::Subshell(Box::new(commands.remove(0))))
        } else {
            Ok(Command::Subshell(Box::new(Command::Block(Block { commands }))))
        }
    }

    fn parse_double_paren_command(&mut self) -> Result<Command, ParserError> {
        // TODO: Implement double paren command parsing
        Err(ParserError::InvalidSyntax("Double paren commands not yet implemented".to_string()))
    }

    fn parse_test_expression(&mut self) -> Result<Command, ParserError> {
        use crate::ast::{TestExpression, TestModifiers};
        
        // Consume the opening [
        if !matches!(self.lexer.peek(), Some(Token::TestBracket)) {
            return Err(ParserError::InvalidSyntax("Expected '[' for test expression".to_string()));
        }
        self.lexer.next(); // consume '['
        
        // Capture the content between [ and ]
        let mut expression_parts = Vec::new();
        
        loop {
            match self.lexer.peek() {
                Some(Token::TestBracketClose) => {
                    self.lexer.next(); // consume ']'
                    break;
                }
                Some(Token::File) => {
                    expression_parts.push("-f".to_string());
                    self.lexer.next();
                }
                Some(Token::Directory) => {
                    expression_parts.push("-d".to_string());
                    self.lexer.next();
                }
                Some(Token::Exists) => {
                    expression_parts.push("-e".to_string());
                    self.lexer.next();
                }
                Some(Token::Readable) => {
                    expression_parts.push("-r".to_string());
                    self.lexer.next();
                }
                Some(Token::Writable) => {
                    expression_parts.push("-w".to_string());
                    self.lexer.next();
                }
                Some(Token::Executable) => {
                    expression_parts.push("-x".to_string());
                    self.lexer.next();
                }
                Some(Token::Size) => {
                    expression_parts.push("-s".to_string());
                    self.lexer.next();
                }
                Some(Token::Symlink) => {
                    expression_parts.push("-L".to_string());
                    self.lexer.next();
                }
                                 Some(Token::Identifier) => {
                     let identifier = self.lexer.get_identifier_text()?;
                     expression_parts.push(identifier);
                     self.lexer.next(); // consume the identifier token
                 }
                 Some(Token::DoubleQuotedString) | Some(Token::SingleQuotedString) => {
                     let string_text = self.lexer.get_string_text()?;
                     expression_parts.push(string_text);
                     self.lexer.next(); // consume the string token
                 }
                Some(Token::Space) | Some(Token::Tab) => {
                    self.lexer.next(); // skip whitespace
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

    fn parse_variable_expansion(&mut self) -> Result<Word, ParserError> {
        // Check what type of variable expansion we have
        match self.lexer.peek() {
            Some(Token::Dollar) => {
                // Simple variable reference like $i
                self.lexer.next(); // consume the $ token
                
                // Expect an identifier after the $
                if let Some(Token::Identifier) = self.lexer.peek() {
                    let var_name = self.lexer.get_identifier_text()?;
                    Ok(Word::Variable(var_name))
                } else {
                    Err(ParserError::InvalidSyntax("Expected identifier after $ in variable expansion".to_string()))
                }
            }
            Some(Token::DollarBrace) => {
                // Parameter expansion like ${i}
                self.lexer.next(); // consume the ${ token
                
                // Parse the content until we find the closing }
                let mut expression_parts = Vec::new();
                
                loop {
                    match self.lexer.peek() {
                        Some(Token::BraceClose) => {
                            // Found the closing }, consume it and break
                            self.lexer.next();
                            break;
                        }
                                                 Some(Token::Identifier) => {
                             // Variable name like 'i'
                             let var_name = self.lexer.get_identifier_text()?;
                             expression_parts.push(var_name);
                             self.lexer.next(); // consume the identifier token
                         }
                         Some(Token::Number) => {
                             // Number like '1'
                             let num_text = self.lexer.get_number_text()?;
                             expression_parts.push(num_text);
                             self.lexer.next(); // consume the number token
                         }
                        Some(Token::Space) | Some(Token::Tab) => {
                            // Skip whitespace
                            self.lexer.next();
                        }
                        None => {
                            return Err(ParserError::InvalidSyntax("Unexpected end of input in parameter expansion".to_string()));
                        }
                        _ => {
                            return Err(ParserError::InvalidSyntax("Unexpected token in parameter expansion".to_string()));
                        }
                    }
                }
                
                // For now, just create a simple parameter expansion
                // In a full implementation, this would parse operators like :-, :+, :?, etc.
                let var_name = expression_parts.join("");
                Ok(Word::ParameterExpansion(ParameterExpansion {
                    variable: var_name,
                    operator: ParameterExpansionOperator::None,
                }))
            }
            _ => {
                Err(ParserError::InvalidSyntax("Expected $ or ${ in variable expansion".to_string()))
            }
        }
    }

    fn parse_arithmetic_expression(&mut self) -> Result<Word, ParserError> {
        // Handle arithmetic expressions like $((i + 1))
        // The lexer should have already consumed the opening $( tokens
        // We need to parse the content until we find the closing ))
        
        let mut expression_parts = Vec::new();
        
        loop {
            match self.lexer.peek() {
                Some(Token::ArithmeticEvalClose) => {
                    // Found the closing )), consume it and break
                    self.lexer.next();
                    break;
                }
                Some(Token::Identifier) => {
                    // Variable reference like 'i'
                    let var_name = self.lexer.get_identifier_text()?;
                    expression_parts.push(var_name);
                    self.lexer.next(); // consume the identifier token
                }
                Some(Token::Number) => {
                    // Number like '1'
                    let num_text = self.lexer.get_number_text()?;
                    expression_parts.push(num_text);
                    self.lexer.next(); // consume the number token
                }
                Some(Token::Plus) => {
                    // Plus operator
                    self.lexer.next();
                    expression_parts.push("+".to_string());
                }
                Some(Token::Minus) => {
                    // Minus operator
                    self.lexer.next();
                    expression_parts.push("-".to_string());
                }
                Some(Token::Star) => {
                    // Multiplication operator
                    self.lexer.next();
                    expression_parts.push("*".to_string());
                }
                Some(Token::Slash) => {
                    // Division operator
                    self.lexer.next();
                    expression_parts.push("/".to_string());
                }
                Some(Token::Space) | Some(Token::Tab) => {
                    // Skip whitespace
                    self.lexer.next();
                }
                None => {
                    return Err(ParserError::InvalidSyntax("Unexpected end of input in arithmetic expression".to_string()));
                }
                _ => {
                    return Err(ParserError::InvalidSyntax("Unexpected token in arithmetic expression".to_string()));
                }
            }
        }
        
        // Create an arithmetic expression word
        let expression = expression_parts.join("");
        Ok(Word::Arithmetic(ArithmeticExpression {
            expression,
            tokens: vec![], // For now, leave tokens empty
        }))
    }



    fn update_shopt_state(&mut self, option: &str, enable: bool) {
        match option {
            "extglob" => self.shopt_state.extglob = enable,
            "nocasematch" => self.shopt_state.nocasematch = enable,
            "globstar" => self.shopt_state.globstar = enable,
            "nullglob" => self.shopt_state.nullglob = enable,
            "failglob" => self.shopt_state.failglob = enable,
            "dotglob" => self.shopt_state.dotglob = enable,
            _ => {} // Ignore unknown options
        }
    }

    fn get_current_shopt_state(&self) -> TestModifiers {
        self.shopt_state.to_owned()
    }
}

// Re-export the main parsing function
pub fn parse(input: &str) -> Result<Vec<Command>, ParserError> {
    let mut parser = Parser::new(input);
    parser.parse()
}

