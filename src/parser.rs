use crate::ast::*;
use crate::lexer::{Lexer, Token, LexerError};
use thiserror::Error;
use std::collections::HashMap;

// Use the debug macros from the root of the crate
use crate::{debug_println, debug_eprintln};

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("Lexer error: {0}")]
    Lexer(#[from] LexerError),
    #[error("Unexpected token: {token:?} at {line}:{col}")]
    UnexpectedToken { token: Token, line: usize, col: usize },
    #[error("Expected token: {0:?}")]
    ExpectedToken(Token),
    #[error("Unexpected end of input")]
    UnexpectedEOF,
    #[error("Invalid syntax: {0}")]
    InvalidSyntax(String),
}

pub struct Parser {
    lexer: Lexer,
    shopt_state: TestModifiers,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        Self {
            lexer: Lexer::new(input),
            shopt_state: TestModifiers::default(),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Command>, ParserError> {
        let mut commands = Vec::new();
        
        // Skip initial whitespace and comments
        self.skip_whitespace_and_comments();
        
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
                        self.skip_whitespace_and_comments();
                        continue;
                    }
                    Token::Semicolon | Token::CarriageReturn | Token::Background => {
                        self.lexer.next();
                        self.skip_whitespace_and_comments();
                        continue;
                    }
                    _ => {}
                }
            }
            
            let mut command = self.parse_command()?;

            // Check if this command is followed by a pipeline
            if let Some(Token::Pipe) = self.lexer.peek() {
                debug_eprintln!("DEBUG: Found pipe after command, parsing pipeline");
                // Parse the pipeline starting from the current command
                command = self.parse_pipeline_from_command(command)?;
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
            self.skip_whitespace_and_comments();

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
            // Bash arithmetic evaluation: (( ... ))
            Some(Token::ParenOpen) if matches!(self.lexer.peek_n(1), Some(Token::ParenOpen)) => {
                self.parse_double_paren_command()
            }
            Some(Token::ParenOpen) => self.parse_subshell(),
            Some(Token::Semicolon) | Some(Token::Newline) | Some(Token::CarriageReturn) => {
                // Skip semicolon and continue parsing
                self.lexer.next();
                self.parse_command()
            }
            _ => self.parse_pipeline(),
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
            self.skip_whitespace_and_comments();
            let Some(token) = self.lexer.peek() else { break; };
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
                Token::Semicolon | Token::Newline => {
                    // Stop parsing pipeline when we hit a command separator
                    // Don't consume the semicolon/newline - let the main parsing loop handle it
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
        let mut args = Vec::new();
        let mut redirects = Vec::new();
        let mut env_vars = HashMap::new();
        
        // Parse environment variable-style assignments at the start
        while let Some(token) = self.lexer.peek() {
            match token {
                Token::Identifier => {
                    if let Some(Token::Assign) = self.lexer.peek_n(1) {
                        let var_name = self.get_identifier_text()?;
                        self.lexer.next(); // consume =
                        // Parse the value as a Word (which can be a string, arithmetic expression, etc.)
                        let value_word = self.parse_environment_variable_value()?;
                        env_vars.insert(var_name, value_word);
                        
                        // Skip whitespace after the environment variable
                        self.skip_whitespace_and_comments();
                    } else if matches!(self.lexer.peek_n(1), Some(Token::TestBracket)) {
                        // Handle associative/indexed array assignment like: map[foo]=bar
                        let (start, _) = self.lexer.get_span().ok_or(ParserError::UnexpectedEOF)?;
                        let mut bracket_depth: i32 = 0;
                        loop {
                            if let Some((_, end)) = self.lexer.get_span() {
                                match self.lexer.peek() {
                                    Some(Token::TestBracket) => { bracket_depth += 1; }
                                    Some(Token::TestBracketClose) => { bracket_depth -= 1; }
                                    _ => {}
                                }
                                let done = bracket_depth == 0 && matches!(self.lexer.peek_n(1), Some(Token::Assign));
                                self.lexer.next();
                                if done {
                                    let name_text = self.lexer.get_text(start, end);
                                    let _eq = self.lexer.next();
                                    self.skip_whitespace_and_comments();
                                    let value_word = self.parse_environment_variable_value()?;
                                    env_vars.insert(name_text, value_word);
                                    self.skip_whitespace_and_comments();
                                    break;
                                }
                            } else { return Err(ParserError::UnexpectedEOF); }
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
                    Word::Literal(self.get_identifier_text()?)
                }
                Token::Set | Token::Export | Token::Readonly | Token::Local | Token::Declare | Token::Typeset |
                Token::Unset | Token::Shift | Token::Eval | Token::Exec | Token::Source | Token::Trap | Token::Wait | Token::Shopt => {
                    Word::Literal(self.get_raw_token_text()?) // keep exact text for builtin/keyword-as-command
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
                _ => {
                    // If we have parsed environment-style assignments and hit a separator,
                    // treat as an assignment-only command (no external program), using "true" as no-op.
                    match token {
                        Token::Semicolon | Token::Newline | Token::Done | Token::Fi | Token::Then | Token::Else | Token::ParenClose | Token::BraceClose => {
                            Word::Literal("true".to_string())
                        }
                        _ => {
                            let (line, col) = self
                                .lexer
                                .get_span()
                                .map(|(s, _)| self.lexer.offset_to_line_col(s))
                                .unwrap_or((1, 1));
                            return Err(ParserError::UnexpectedToken { token: token.clone(), line, col });
                        }
                    }
                }
            }
        } else {
            return Err(ParserError::UnexpectedEOF);
        };
        
        // Skip whitespace before parsing arguments
        self.skip_whitespace_and_comments();
        
        // Special handling for Bash double-bracket test: capture everything until closing ']]'
        if is_double_bracket {
            let expr = self.capture_double_bracket_expression()?;
            debug_eprintln!("DEBUG: Creating TestExpression with expression: '{}'", expr);
            return Ok(Command::TestExpression(TestExpression {
                expression: expr,
                modifiers: self.get_current_shopt_state(),
            }));
        }

        // Parse arguments and redirects
        while let Some(token) = self.lexer.peek() {
            match token {
                Token::Identifier | Token::Number | Token::DoubleQuotedString | Token::SingleQuotedString | Token::SourceDot | Token::BraceOpen | Token::BacktickString | Token::DollarSingleQuotedString | Token::DollarDoubleQuotedString | Token::Star => {
                    args.push(self.parse_word()?);
                }
                Token::Dollar | Token::DollarBrace | Token::DollarParen | Token::DollarHashSimple | Token::DollarAtSimple | Token::DollarStarSimple
                | Token::DollarBraceHash | Token::DollarBraceBang | Token::DollarBraceStar | Token::DollarBraceAt
                | Token::DollarBraceHashStar | Token::DollarBraceHashAt | Token::DollarBraceBangStar | Token::DollarBraceBangAt => {
                    args.push(self.parse_variable_expansion()?);
                }
                Token::Arithmetic => {
                    args.push(self.parse_arithmetic_expression()?);
                }
                Token::Minus => {
                    // Handle arguments starting with minus (like -la, -v, etc.)
                    let token_clone = token.clone();
                    self.lexer.next(); // consume the minus
                    
                    // Check if we have a specific shell option token next
                    if let Some(token_after_minus) = self.lexer.peek() {
                        match token_after_minus {
                            Token::Exists | Token::Readable | Token::Writable | Token::Executable |
                            Token::Size | Token::Symlink | Token::SymlinkH | Token::PipeFile |
                            Token::Socket | Token::Block | Token::Character | Token::SetGid |
                            Token::Sticky | Token::SetUid | Token::Owned | Token::GroupOwned |
                            Token::Modified | Token::NonZero | Token::File | Token::Directory => {
                                // Handle specific shell option tokens
                                let option_text = self.get_raw_token_text()?;
                                args.push(Word::Literal(option_text));
                            }
                            Token::Identifier => {
                                // Handle generic identifier after minus
                                let arg = Word::Literal(format!("-{}", self.get_identifier_text()?));
                                args.push(arg);
                            }
                            Token::Number => {
                                // Handle number after minus
                                let num = self.get_number_text()?;
                                args.push(Word::Literal(format!("-{}", num)));
                            }
                            _ => {
                                // Get the actual line and column for better error reporting
                                let (line, col) = self
                                    .lexer
                                    .get_span()
                                    .map(|(s, _)| self.lexer.offset_to_line_col(s))
                                    .unwrap_or((1, 1));
                                return Err(ParserError::UnexpectedToken { token: token_clone, line, col });
                            }
                        }
                    } else {
                        // Handle standalone minus
                        args.push(Word::Literal("-".to_string()));
                    }
                }
                // Process substitution as args: <(cmd) or >(cmd)
                Token::RedirectIn => {
                    if matches!(self.lexer.peek_n(1), Some(Token::ParenOpen)) {
                        self.lexer.next(); // consume '<'
                        let inner = self.capture_parenthetical_text()?;
                        args.push(Word::Literal(format!("<{}", inner)));
                    } else {
                        redirects.push(self.parse_redirect()?);
                    }
                }
                Token::RedirectOut => {
                    if matches!(self.lexer.peek_n(1), Some(Token::ParenOpen)) {
                        self.lexer.next(); // consume '>'
                        let inner = self.capture_parenthetical_text()?;
                        args.push(Word::Literal(format!(">{}", inner)));
                    } else {
                        redirects.push(self.parse_redirect()?);
                    }
                }
                Token::NonZero => {
                    // Handle -n argument
                    self.lexer.next(); // consume the NonZero token
                    if let Some(Token::Identifier) = self.lexer.peek() {
                        let arg = Word::Literal(format!("-n{}", self.get_identifier_text()?));
                        args.push(arg);
                    } else {
                        args.push(Word::Literal("-n".to_string()));
                    }
                }
                Token::Character => {
                    // Handle -c argument
                    self.lexer.next(); // consume the Character token
                    if let Some(Token::Identifier) = self.lexer.peek() {
                        let arg = Word::Literal(format!("-c{}", self.get_identifier_text()?));
                        args.push(arg);
                    } else {
                        args.push(Word::Literal("-c".to_string()));
                    }
                }
                Token::File => {
                    // Handle -f argument
                    self.lexer.next(); // consume the File token
                    args.push(Word::Literal("-f".to_string()));
                }
                Token::Directory => {
                    // Handle -d argument
                    self.lexer.next(); // consume the Directory token
                    args.push(Word::Literal("-d".to_string()));
                }
                Token::Exists => {
                    // Handle -e argument
                    self.lexer.next(); // consume the Exists token
                    args.push(Word::Literal("-e".to_string()));
                }
                Token::Readable => {
                    // Handle -r argument
                    self.lexer.next(); // consume the Readable token
                    args.push(Word::Literal("-r".to_string()));
                }
                Token::Writable => {
                    // Handle -w argument
                    self.lexer.next(); // consume the Writable token
                    args.push(Word::Literal("-w".to_string()));
                }
                Token::Executable => {
                    // Handle -x argument
                    self.lexer.next(); // consume the Executable token
                    args.push(Word::Literal("-x".to_string()));
                }
                Token::Size => { self.lexer.next(); args.push(Word::Literal("-s".to_string())); }
                Token::Symlink => { self.lexer.next(); args.push(Word::Literal("-L".to_string())); }
                Token::SymlinkH => { self.lexer.next(); args.push(Word::Literal("-h".to_string())); }
                Token::PipeFile => { self.lexer.next(); args.push(Word::Literal("-p".to_string())); }
                Token::Socket => { self.lexer.next(); args.push(Word::Literal("-S".to_string())); }
                Token::Block => { self.lexer.next(); args.push(Word::Literal("-b".to_string())); }
                Token::SetGid => { self.lexer.next(); args.push(Word::Literal("-g".to_string())); }
                Token::Sticky => { self.lexer.next(); args.push(Word::Literal("-k".to_string())); }
                Token::SetUid => { self.lexer.next(); args.push(Word::Literal("-u".to_string())); }
                Token::Owned => { self.lexer.next(); args.push(Word::Literal("-O".to_string())); }
                Token::GroupOwned => { self.lexer.next(); args.push(Word::Literal("-G".to_string())); }
                Token::Modified => { self.lexer.next(); args.push(Word::Literal("-N".to_string())); }
                // Test comparison operators
                Token::Eq => { self.lexer.next(); args.push(Word::Literal("-eq".to_string())); }
                Token::Ne => { self.lexer.next(); args.push(Word::Literal("-ne".to_string())); }
                Token::Lt => { self.lexer.next(); args.push(Word::Literal("-lt".to_string())); }
                Token::Le => { self.lexer.next(); args.push(Word::Literal("-le".to_string())); }
                Token::Gt => { self.lexer.next(); args.push(Word::Literal("-gt".to_string())); }
                Token::Ge => { self.lexer.next(); args.push(Word::Literal("-ge".to_string())); }
                Token::NewerThan => { self.lexer.next(); args.push(Word::Literal("-nt".to_string())); }
                Token::OlderThan => { self.lexer.next(); args.push(Word::Literal("-ot".to_string())); }
                Token::SameFile => { self.lexer.next(); args.push(Word::Literal("-ef".to_string())); }
                Token::Zero => { self.lexer.next(); args.push(Word::Literal("-z".to_string())); }
                Token::RedirectAppend | Token::RedirectInOut | Token::Heredoc | Token::HeredocTabs | Token::HereString => {
                    redirects.push(self.parse_redirect()?);
                }
                Token::Newline | Token::Semicolon | Token::CarriageReturn => {
                    // Stop parsing arguments when we hit a command separator
                    break;
                }
                Token::TestBracketClose => {
                    // Handle closing bracket for test commands
                    self.lexer.next(); // consume the ]
                    if is_double_bracket {
                        if let Some(Token::TestBracketClose) = self.lexer.peek() { self.lexer.next(); }
                    }
                    // Don't add "]" to args - it's just a syntax marker
                    break;
                }
                Token::Space | Token::Tab => {
                    // Skip whitespace but continue parsing arguments
                    self.lexer.next();
                }
                _ => break,
            }
        }

        // If this was a [[ ... ]] and nothing captured into args, greedily capture raw text
        if is_double_bracket && args.is_empty() {
            let mut expr = String::new();
            loop {
                match self.lexer.peek() {
                    Some(Token::TestBracketClose) if matches!(self.lexer.peek_n(1), Some(Token::TestBracketClose)) => {
                        self.lexer.next();
                        self.lexer.next();
                        break;
                    }
                    Some(_) => {
                        if let Some((s, e)) = self.lexer.get_span() {
                            expr.push_str(&self.lexer.get_text(s, e));
                        }
                        self.lexer.next();
                    }
                    None => break,
                }
            }
            let trimmed = expr.trim().to_string();
            if !trimmed.is_empty() { args.push(Word::Literal(trimmed)); }
        }
        
        // Postprocess: convert shopt commands to ShoptCommand AST nodes
        if name == "shopt" && args.len() >= 2 {
            if let (Some(flag), Some(option)) = (args.get(0), args.get(1)) {
                if let (Word::Literal(flag_str), Word::Literal(option_str)) = (flag, option) {
                    match flag_str.as_str() {
                        "-s" => {
                            // Update parser state
                            self.update_shopt_state(option_str, true);
                            return Ok(Command::ShoptCommand(ShoptCommand {
                                option: option_str.clone(),
                                enable: true,
                            }));
                        }
                        "-u" => {
                            // Update parser state
                            self.update_shopt_state(option_str, false);
                            return Ok(Command::ShoptCommand(ShoptCommand {
                                option: option_str.clone(),
                                enable: false,
                            }));
                        }
                        _ => {}
                    }
                }
            }
        }
        
        // Postprocess: convert builtin commands to BuiltinCommand AST nodes
        if let Word::Literal(name_str) = &name {
            match name_str.as_str() {
                "set" | "export" | "readonly" | "local" | "declare" | "typeset" |
                "unset" | "shift" | "eval" | "exec" | "source" | "trap" | "wait" => {
                    return Ok(Command::BuiltinCommand(BuiltinCommand {
                        name: name_str.clone(),
                        args: args,
                        redirects: redirects,
                        env_vars: env_vars,
                    }));
                }
                _ => {}
            }
        }
        
        // Postprocess: convert [[ ... ]] test expressions to TestExpression AST nodes
        if name == "[[".to_string() && !args.is_empty() {
            if let Some(Word::Literal(expr)) = args.first() {
                return Ok(Command::TestExpression(TestExpression {
                    expression: expr.clone(),
                    modifiers: self.get_current_shopt_state(),
                }));
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
        
        // Consume optional separator (semicolon or newline) after condition
        match self.lexer.peek() {
            Some(Token::Semicolon) | Some(Token::Newline) => { self.lexer.next(); },
            _ => {}
        }
        
        // Skip whitespace/newlines before then
        while matches!(self.lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline)) {
            self.lexer.next();
        }
        
        self.lexer.consume(Token::Then)?;
        // Allow newline/whitespace after 'then'
        while matches!(self.lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline)) {
            self.lexer.next();
        }
        let then_branch = Box::new(self.parse_command()?);
        
        // Skip whitespace/newlines before checking for separator
        while matches!(self.lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline)) {
            self.lexer.next();
        }
        
        // Consume optional separator (semicolon or newline) after then branch
        match self.lexer.peek() {
            Some(Token::Semicolon) | Some(Token::Newline) => {
                self.lexer.next();
                while matches!(self.lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline)) {
                    self.lexer.next();
                }
            },
            _ => {}
        }
        
        let else_branch = if let Some(Token::Else) = self.lexer.peek() {
            self.lexer.next();
            // Allow newline/whitespace after 'else'
            while matches!(self.lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline)) {
                self.lexer.next();
            }
            Some(Box::new(self.parse_command()?))
        } else {
            None
        };
        
        // Skip whitespace/newlines before fi
        while matches!(self.lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline)) {
            self.lexer.next();
        }
        
        self.lexer.consume(Token::Fi)?;
        
        Ok(Command::If(IfStatement {
            condition,
            then_branch,
            else_branch,
        }))
    }

    fn parse_while_loop(&mut self) -> Result<Command, ParserError> {
        self.lexer.consume(Token::While)?;
        // Parse condition
        let condition = Box::new(self.parse_command()?);

        // Optional separator after condition (semicolon or newline) and skip whitespace
        match self.lexer.peek() {
            Some(Token::Semicolon) | Some(Token::Newline) | Some(Token::CarriageReturn) => { self.lexer.next(); },
            _ => {}
        }
        while matches!(self.lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline | Token::CarriageReturn)) {
            self.lexer.next();
        }

        // Expect 'do'
        self.lexer.consume(Token::Do)?;

        // Allow newline/whitespace after 'do'
        while matches!(self.lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline | Token::CarriageReturn)) {
            self.lexer.next();
        }

        // Parse body commands into a Block
        let mut body_commands = Vec::new();
        
        // Parse first command
        body_commands.push(self.parse_command()?);

        // Parse additional commands in body until 'done'
        loop {
            // Skip separators
            while matches!(self.lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline | Token::CarriageReturn | Token::Semicolon)) {
                self.lexer.next();
            }
            match self.lexer.peek() {
                Some(Token::Done) | None => break,
                _ => {
                    // Parse and add command to body
                    let pre_pos = self.lexer.current_position();
                    let command = self.parse_command()?;
                    body_commands.push(command);
                    if self.lexer.current_position() == pre_pos {
                        if self.lexer.next().is_none() { break; }
                    }
                }
            }
        }

        // Allow optional separator after body before 'done'
        loop {
            match self.lexer.peek() {
                Some(Token::Space) | Some(Token::Tab) | Some(Token::Comment) | Some(Token::Newline | Token::CarriageReturn) => {
                    self.lexer.next();
                    continue;
                }
                Some(Token::Semicolon) => {
                    self.lexer.next();
                    // consume any following whitespace/newlines as well
                    continue;
                }
                _ => {}
            }
            break;
        }

        self.lexer.consume(Token::Done)?;
        
        let body = Block { commands: body_commands };
        Ok(Command::While(WhileLoop { condition, body }))
    }

    fn parse_for_loop(&mut self) -> Result<Command, ParserError> {
        self.lexer.consume(Token::For)?;
        // Allow whitespace/comments after 'for'
        self.skip_whitespace_and_comments();

        // Variable name
        let variable = match self.lexer.peek() {
            Some(Token::Identifier) => self.get_identifier_text()?,
            Some(t) => return Err(ParserError::UnexpectedToken { token: t.clone(), line: 1, col: 1 }),
            None => return Err(ParserError::UnexpectedEOF),
        };

        // Allow whitespace/comments after variable
        self.skip_whitespace_and_comments();

        // Optional 'in' list
        let items = if let Some(Token::In) = self.lexer.peek() {
            self.lexer.next();
            // Allow whitespace/comments after 'in'
            self.skip_whitespace_and_comments();
            let words = self.parse_word_list()?;
            // Optional separator before 'do'
            while matches!(self.lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::CarriageReturn)) {
                self.lexer.next();
            }
            match self.lexer.peek() {
                Some(Token::Semicolon) | Some(Token::Newline) | Some(Token::CarriageReturn) => {
                    self.lexer.next();
                }
                _ => {}
            }
            words
        } else {
            // No 'in' list; optional separator before 'do'
            while matches!(self.lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::CarriageReturn)) {
                self.lexer.next();
            }
            match self.lexer.peek() {
                Some(Token::Semicolon) | Some(Token::Newline) | Some(Token::CarriageReturn) => {
                    self.lexer.next();
                }
                _ => {}
            }
            Vec::new()
        };

        // Allow whitespace/newlines/comments before 'do'
        while matches!(self.lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline | Token::CarriageReturn)) {
            self.lexer.next();
        }
        self.lexer.consume(Token::Do)?;
        
        // Parse body commands into a Block
        let mut body_commands = Vec::new();
        
        // Parse first command
        body_commands.push(self.parse_command()?);

        // Parse additional commands in body until 'done'
        loop {
            // Skip separators
            while matches!(self.lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline | Token::CarriageReturn)) {
                self.lexer.next();
            }
            
            // Check for 'done' first
            if let Some(Token::Done) = self.lexer.peek() {
                break;
            }
            
            // Check for semicolon - this should separate commands in the loop body
            if let Some(Token::Semicolon) = self.lexer.peek() {
                self.lexer.next(); // consume semicolon
                // Skip whitespace after semicolon
                self.skip_whitespace_and_comments();
                
                // Check if the next token is 'done'
                if let Some(Token::Done) = self.lexer.peek() {
                    break;
                }
                
                // Continue parsing the next command in the loop body
                continue;
            }
            
            // Parse additional command in body
            let pre_pos = self.lexer.current_position();
            let command = self.parse_command()?;
            body_commands.push(command);
            if self.lexer.current_position() == pre_pos {
                if self.lexer.next().is_none() { break; }
            }
        }

        // Allow optional separator after body before 'done'
        loop {
            match self.lexer.peek() {
                Some(Token::Space) | Some(Token::Tab) | Some(Token::Comment) | Some(Token::Newline | Token::CarriageReturn) => {
                    self.lexer.next();
                    continue;
                }
                Some(Token::Semicolon) => {
                    self.lexer.next();
                    // consume any following whitespace/newlines as well
                    continue;
                }
                _ => {}
            }
            break;
        }

        self.lexer.consume(Token::Done)?;
        
        // Skip whitespace after 'done' before checking for pipe
        self.skip_whitespace_and_comments();
        
        // Check if there's a pipeline after the for loop
        let mut final_command = Command::For(ForLoop {
            variable,
            items,
            body: Block { commands: body_commands },
        });
        
        // If there's a pipe after 'done', parse the pipeline
        if let Some(Token::Pipe) = self.lexer.peek() {
            final_command = self.parse_pipeline_from_command(final_command)?;
        }
        
        Ok(final_command)
    }

    fn parse_function(&mut self) -> Result<Command, ParserError> {
        self.lexer.consume(Token::Function)?;
        // Allow whitespace between 'function' and name
        self.skip_whitespace_and_comments();

        let name = match self.lexer.peek() {
            Some(Token::Identifier) => self.get_identifier_text()?,
            Some(t) => {
                let (line, col) = self
                    .lexer
                    .get_span()
                    .map(|(s, _)| self.lexer.offset_to_line_col(s))
                    .unwrap_or((1, 1));
                return Err(ParserError::UnexpectedToken { token: t.clone(), line, col });
            }
            None => return Err(ParserError::UnexpectedEOF),
        };

        // Skip whitespace after name
        self.skip_whitespace_and_comments();

        // Optional empty parentheses after function name: function name()
        if let Some(Token::ParenOpen) = self.lexer.peek() {
            // Consume () if present
            self.lexer.next();
            if let Some(Token::ParenClose) = self.lexer.peek() { self.lexer.next(); }
            // Skip whitespace/newlines
            while matches!(self.lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline)) {
                self.lexer.next();
            }
        }

        // Brace-wrapped function body: { ... }
        let body = if let Some(Token::BraceOpen) = self.lexer.peek() {
            // Consume '{'
            self.lexer.next();
            // Allow whitespace/newlines
            while matches!(self.lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline)) {
                self.lexer.next();
            }
            
            // Parse body commands into a Block
            let mut body_commands = Vec::new();
            
            // Parse first command
            body_commands.push(self.parse_command()?);

            // Parse additional commands inside the block
            loop {
                // Skip separators
                while matches!(self.lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline | Token::Semicolon)) {
                    self.lexer.next();
                }
                match self.lexer.peek() {
                    Some(Token::BraceClose) | None => break,
                    _ => {
                        let pre_pos = self.lexer.current_position();
                        let command = self.parse_command()?;
                        body_commands.push(command);
                        if self.lexer.current_position() == pre_pos {
                            if self.lexer.next().is_none() { break; }
                        }
                    }
                }
            }

            // Expect closing '}'
            self.lexer.consume(Token::BraceClose)?;
            Block { commands: body_commands }
        } else {
            // Fallback: parse next as a single command body
            let command = self.parse_command()?;
            Block { commands: vec![command] }
        };
        
        Ok(Command::Function(Function { name, body }))
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
            Some(Token::HereString) => RedirectOperator::HereString,
            _ => return Err(ParserError::InvalidSyntax("Invalid redirect operator".to_string())),
        };
        // Here-string: '<<< word' often lexes as '<<' '<' then word; accept optional extra '<'
        if matches!(operator, RedirectOperator::Heredoc) {
            if let Some(Token::RedirectIn) = self.lexer.peek() { self.lexer.next(); }
        }
        // Skip whitespace before target
        self.skip_whitespace_and_comments();

        // Process substitution as redirect target, allowing an optional extra '<' before '('
        // For here-strings, we don't need a target - the string content follows immediately
        let target = if matches!(operator, RedirectOperator::HereString) {
            // For here-strings, create a placeholder target since we'll get the content from heredoc_body
            Word::Literal("".to_string())
        } else if matches!(self.lexer.peek(), Some(Token::RedirectIn)) && matches!(self.lexer.peek_n(1), Some(Token::ParenOpen)) {
            // consume the extra '<' and capture ( ... )
            self.lexer.next();
            Word::Literal(self.capture_parenthetical_text()?)
        } else if matches!(self.lexer.peek(), Some(Token::ParenOpen)) {
            Word::Literal(self.capture_parenthetical_text()?)
        } else {
            self.parse_word()?
        };
        // If this is a heredoc, capture lines until the delimiter is found at start of line
        // If this is a here-string, the target is the string content
        let heredoc_body = match operator {
            RedirectOperator::Heredoc | RedirectOperator::HeredocTabs => {
                let delim = match &target {
                    Word::Literal(s) => s.clone(),
                    _ => return Err(ParserError::InvalidSyntax("Heredoc delimiter must be a literal string".to_string())),
                };
                let mut body = String::new();
                let mut found_delim = false;

                // Skip to the next newline token
                while let Some(token) = self.lexer.peek() {
                    match token {
                        Token::Newline => {
                            self.lexer.next(); // consume the newline
                            break;
                        }
                        _ => {
                            self.lexer.next(); // consume other tokens
                        }
                    }
                }

                // Collect lines until we find the delimiter at start of line
                while let Some(token) = self.lexer.peek() {
                    match token {
                        Token::Newline => {
                            self.lexer.next(); // consume the newline
                            // Don't add newline if this is the last line (before delimiter)
                            if let Some(Token::Identifier) = self.lexer.peek() {
                                let next_word = self.lexer.get_current_text().unwrap_or_default();
                                if next_word != delim {
                                    body.push('\n');
                                }
                            } else {
                                body.push('\n');
                            }
                        }
                        Token::Identifier => {
                            let word = self.get_identifier_text()?;
                            if word == delim {
                                found_delim = true;
                                break;
                            }
                            body.push_str(&word);
                        }
                        _ => {
                            // For any other token, just consume it and add to body
                            let text = self.lexer.get_current_text().unwrap_or_default();
                            body.push_str(&text);
                            self.lexer.next();
                        }
                    }
                }

                if found_delim {
                    Some(body)
                } else {
                    Some(String::new())
                }
            }
            RedirectOperator::HereString => {
                // For here-strings, the target is the string content
                // We need to extract the string content from the target
                match &target {
                    Word::Literal(s) => Some(s.clone()),
                    Word::StringInterpolation(interp) => {
                        // Convert string interpolation to a string
                        let mut result = String::new();
                        for part in &interp.parts {
                            match part {
                                &StringPart::Literal(ref s) => result.push_str(s),
                                &StringPart::Variable(ref var) => result.push_str(&format!("${}", var)),
                                &StringPart::MapAccess(ref map_name, ref key) => result.push_str(&format!("${{{}}}[{}]", map_name, key)),
                                &StringPart::MapKeys(ref map_name) => result.push_str(&format!("${{!{}}}[@]", map_name)),
                                &StringPart::MapLength(ref map_name) => result.push_str(&format!("${{#{}}}[@]", map_name)),
                                &StringPart::Arithmetic(ref expr) => result.push_str(&expr.expression),
                                &StringPart::CommandSubstitution(_) => result.push_str("$(...)"),
                                &StringPart::ParameterExpansion(ref pe) => result.push_str(&pe.to_string()),
                            }
                        }
                        Some(result)
                    }
                    _ => None,
                }
            }
            _ => None,
        };

        Ok(Redirect { fd, operator, target, heredoc_body })
    }

    fn parse_word(&mut self) -> Result<Word, ParserError> {
        let result = match self.lexer.peek() {
            Some(Token::Identifier) => Ok(Word::Literal(self.get_identifier_text()?)),
            Some(Token::Number) => Ok(Word::Literal(self.get_number_text()?)),
            Some(Token::DoubleQuotedString) => Ok(self.parse_string_interpolation()?),
            Some(Token::SingleQuotedString) => Ok(Word::Literal(self.get_string_text()?)),
            Some(Token::BacktickString) => Ok(Word::Literal(self.get_raw_token_text()?)),
            Some(Token::DollarSingleQuotedString) => Ok(self.parse_ansic_quoted_string()?),
            Some(Token::DollarDoubleQuotedString) => Ok(self.parse_string_interpolation()?),
            Some(Token::BraceOpen) => Ok(self.parse_brace_expansion()?),
            Some(Token::SourceDot) => {
                // Treat standalone '.' as a normal word (e.g., `find . -name ...`)
                self.lexer.next();
                Ok(Word::Literal(".".to_string()))
            }
            Some(Token::Dollar) => Ok(self.parse_variable_expansion()?),
            Some(Token::DollarBrace) | Some(Token::DollarParen) | Some(Token::DollarHashSimple) | Some(Token::DollarAtSimple) | Some(Token::DollarStarSimple)
            | Some(Token::DollarBraceHash) | Some(Token::DollarBraceBang) | Some(Token::DollarBraceStar) | Some(Token::DollarBraceAt)
            | Some(Token::DollarBraceHashStar) | Some(Token::DollarBraceHashAt) | Some(Token::DollarBraceBangStar) | Some(Token::DollarBraceBangAt)
                => Ok(self.parse_variable_expansion()?),
            Some(Token::Arithmetic) => Ok(self.parse_arithmetic_expression()?),
            _ => {
                let (line, col) = self
                    .lexer
                    .get_span()
                    .map(|(s, _)| self.lexer.offset_to_line_col(s))
                    .unwrap_or((1, 1));
                Err(ParserError::UnexpectedToken { token: Token::Identifier, line, col })
            }
        };
        
        // Skip whitespace after consuming the word
        self.skip_whitespace_and_comments();
        
        result
    }

    fn parse_variable_expansion(&mut self) -> Result<Word, ParserError> {
        match self.lexer.peek() {
            Some(Token::Dollar) => {
                self.lexer.next();
                if let Some(Token::Identifier) = self.lexer.peek() {
                    let var_name = self.get_identifier_text()?;
                    Ok(Word::Variable(var_name))
                } else {
                    Err(ParserError::InvalidSyntax("Expected identifier after $".to_string()))
                }
            }
            Some(Token::DollarHashSimple) => { 
                self.lexer.next(); 
                Ok(Word::Variable("#".to_string()))
            }
            Some(Token::DollarAtSimple) => { 
                self.lexer.next(); 
                Ok(Word::Variable("@".to_string()))
            }
            Some(Token::DollarStarSimple) => { 
                self.lexer.next(); 
                Ok(Word::Variable("*".to_string()))
            }
            Some(Token::DollarBrace) => {
                // Parse ${...} expansions
                self.lexer.next(); // consume the token
                
                // Try to parse as a parameter expansion first
                if let Ok(pe) = self.parse_parameter_expansion() {
                    Ok(pe)
                } else {
                    // Fall back to the old method
                    let var_name = self.parse_braced_variable_name()?;
                    
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
                                    Ok(Word::MapAccess(map_name.to_string(), key.to_string()))
                                } else {
                                    Ok(Word::Variable(var_name))
                                }
                            } else {
                                Ok(Word::Variable(var_name))
                            }
                        } else {
                            Ok(Word::Variable(var_name))
                        }
                    }
                    } else if var_name.contains(":-") {
                        let parts: Vec<&str> = var_name.split(":-").collect();
                        if parts.len() == 2 {
                            Ok(Word::ParameterExpansion(ParameterExpansion {
                                variable: parts[0].to_string(),
                                operator: ParameterExpansionOperator::DefaultValue(parts[1].to_string()),
                            }))
                        } else {
                            // Check if this is a map access pattern like map[foo]
                            if var_name.contains('[') && var_name.contains(']') {
                                if let Some(bracket_start) = var_name.find('[') {
                                    if let Some(bracket_end) = var_name.rfind(']') {
                                        let map_name = &var_name[..bracket_start];
                                        let key = &var_name[bracket_start + 1..bracket_end];
                                        Ok(Word::MapAccess(map_name.to_string(), key.to_string()))
                                    } else {
                                        Ok(Word::Variable(var_name))
                                    }
                                } else {
                                    Ok(Word::Variable(var_name))
                                }
                            } else {
                                Ok(Word::Variable(var_name))
                            }
                        }
                        } else if var_name.contains(":=") {
                        let parts: Vec<&str> = var_name.split(":=").collect();
                        if parts.len() == 2 {
                            Ok(Word::ParameterExpansion(ParameterExpansion {
                                variable: parts[0].to_string(),
                                operator: ParameterExpansionOperator::AssignDefault(parts[1].to_string()),
                            }))
                        } else {
                            // Check if this is a map access pattern like map[foo]
                            if var_name.contains('[') && var_name.contains(']') {
                                if let Some(bracket_start) = var_name.find('[') {
                                    if let Some(bracket_end) = var_name.rfind(']') {
                                        let map_name = &var_name[..bracket_start];
                                        let key = &var_name[bracket_start + 1..bracket_end];
                                        Ok(Word::MapAccess(map_name.to_string(), key.to_string()))
                                    } else {
                                        Ok(Word::Variable(var_name))
                                    }
                                } else {
                                    Ok(Word::Variable(var_name))
                                }
                            } else {
                                Ok(Word::Variable(var_name))
                            }
                        }
                    } else if var_name.contains(":?") {
                        let parts: Vec<&str> = var_name.split(":?").collect();
                        if parts.len() == 2 {
                            Ok(Word::ParameterExpansion(ParameterExpansion {
                                variable: parts[0].to_string(),
                                operator: ParameterExpansionOperator::ErrorIfUnset(parts[1].to_string()),
                            }))
                        } else {
                            // Check if this is a map access pattern like map[foo]
                            if var_name.contains('[') && var_name.contains(']') {
                                if let Some(bracket_start) = var_name.find('[') {
                                    if let Some(bracket_end) = var_name.rfind(']') {
                                        let map_name = &var_name[..bracket_start];
                                        let key = &var_name[bracket_start + 1..bracket_end];
                                        Ok(Word::MapAccess(map_name.to_string(), key.to_string()))
                                    } else {
                                        Ok(Word::Variable(var_name))
                                    }
                                } else {
                                    Ok(Word::Variable(var_name))
                                }
                            } else {
                                Ok(Word::Variable(var_name))
                            }
                        }
                        } else if var_name.contains("#") && !var_name.starts_with('#') {
                        let parts: Vec<&str> = var_name.split("#").collect();
                        if parts.len() == 2 {
                            Ok(Word::ParameterExpansion(ParameterExpansion {
                                variable: parts[0].to_string(),
                                operator: ParameterExpansionOperator::RemoveShortestPrefix(parts[1].to_string()),
                            }))
                        } else {
                            // Check if this is a map access pattern like map[foo]
                            if var_name.contains('[') && var_name.contains(']') {
                                if let Some(bracket_start) = var_name.find('[') {
                                    if let Some(bracket_end) = var_name.rfind(']') {
                                        let map_name = &var_name[..bracket_start];
                                        let key = &var_name[bracket_start + 1..bracket_end];
                                        Ok(Word::MapAccess(map_name.to_string(), key.to_string()))
                                    } else {
                                        Ok(Word::Variable(var_name))
                                    }
                                } else {
                                    Ok(Word::Variable(var_name))
                                }
                            } else {
                                Ok(Word::Variable(var_name))
                            }
                        }
                    } else if var_name.contains("%") && !var_name.starts_with('%') {
                        let parts: Vec<&str> = var_name.split("%").collect();
                        if parts.len() == 2 {
                            Ok(Word::ParameterExpansion(ParameterExpansion {
                                variable: parts[0].to_string(),
                                operator: ParameterExpansionOperator::RemoveShortestSuffix(parts[1].to_string()),
                            }))
                        } else {
                            // Check if this is a map access pattern like map[foo]
                            if var_name.contains('[') && var_name.contains(']') {
                                if let Some(bracket_start) = var_name.find('[') {
                                    if let Some(bracket_end) = var_name.rfind(']') {
                                        let map_name = &var_name[..bracket_start];
                                        let key = &var_name[bracket_start + 1..bracket_end];
                                        Ok(Word::MapAccess(map_name.to_string(), key.to_string()))
                                    } else {
                                        Ok(Word::Variable(var_name))
                                    }
                                } else {
                                    Ok(Word::Variable(var_name))
                                }
                            } else {
                                Ok(Word::Variable(var_name))
                            }
                        }
                    } else {
                        // Check if this is a map access pattern like map[foo]
                        if var_name.contains('[') && var_name.contains(']') {
                            if let Some(bracket_start) = var_name.find('[') {
                                if let Some(bracket_end) = var_name.rfind(']') {
                                    let map_name = &var_name[..bracket_start];
                                    let key = &var_name[bracket_start + 1..bracket_end];
                                    Ok(Word::MapAccess(map_name.to_string(), key.to_string()))
                                } else {
                                    Ok(Word::Variable(var_name))
                                }
                            } else {
                                Ok(Word::Variable(var_name))
                            }
                        } else {
                            Ok(Word::Variable(var_name))
                        }
                    }
                }
            }
            Some(Token::DollarBraceHash) => {
                // Parse ${#...} expansions (array length)
                self.lexer.next(); // consume the token
                let var_name = self.parse_braced_variable_name()?;
                Ok(Word::MapLength(var_name))
            }
            Some(Token::DollarBraceBang) => {
                // Parse ${!...} expansions (associative array keys)
                self.lexer.next(); // consume the token
                let var_name = self.parse_braced_variable_name()?;
                Ok(Word::MapKeys(var_name))
            }
            Some(Token::DollarBraceStar) => {
                // Parse ${*...} expansions
                self.lexer.next(); // consume the token
                let var_name = self.parse_braced_variable_name()?;
                Ok(Word::Variable(format!("*{}", var_name)))
            }
            Some(Token::DollarBraceAt) => {
                // Parse ${@...} expansions
                self.lexer.next(); // consume the token
                let var_name = self.parse_braced_variable_name()?;
                Ok(Word::Variable(format!("@{}", var_name)))
            }
            Some(Token::DollarBraceHashStar) => {
                // Parse ${#*...} expansions
                self.lexer.next(); // consume the token
                let var_name = self.parse_braced_variable_name()?;
                Ok(Word::Variable(format!("#*{}", var_name)))
            }
            Some(Token::DollarBraceHashAt) => {
                // Parse ${#@...} expansions
                self.lexer.next(); // consume the token
                let var_name = self.parse_braced_variable_name()?;
                Ok(Word::Variable(format!("#@{}", var_name)))
            }
            Some(Token::DollarBraceBangStar) => {
                // Parse ${!*...} expansions
                self.lexer.next(); // consume the token
                let var_name = self.parse_braced_variable_name()?;
                Ok(Word::Variable(format!("!*{}", var_name)))
            }
            Some(Token::DollarBraceBangAt) => {
                // Parse ${!@...} expansions
                self.lexer.next(); // consume the token
                let var_name = self.parse_braced_variable_name()?;
                Ok(Word::Variable(format!("!@{}", var_name)))
            }
            Some(Token::DollarParen) => {
                // Parse command substitution: capture until matching ')'
                self.parse_command_substitution()
            }
            _ => {
                let (line, col) = self
                    .lexer
                    .get_span()
                    .map(|(s, _)| self.lexer.offset_to_line_col(s))
                    .unwrap_or((1, 1));
                return Err(ParserError::UnexpectedToken { token: Token::Identifier, line, col });
            }
        }
    }

    fn parse_word_list(&mut self) -> Result<Vec<Word>, ParserError> {
        let mut words = Vec::new();
        
        while let Some(token) = self.lexer.peek() {
            match token {
                Token::Identifier | Token::Number | Token::DoubleQuotedString |
                Token::SingleQuotedString | Token::Dollar | Token::DollarBrace |
                Token::DollarParen | Token::BraceOpen | Token::BacktickString | Token::Arithmetic | Token::DollarHashSimple | Token::DollarAtSimple | Token::DollarStarSimple
                | Token::DollarBraceHash | Token::DollarBraceBang | Token::DollarBraceStar | Token::DollarBraceAt
                | Token::DollarBraceHashStar | Token::DollarBraceHashAt | Token::DollarBraceBangStar | Token::DollarBraceBangAt | Token::Star => {
                    words.push(self.parse_word()?);
                }
                _ => break,
            }
        }
        
        Ok(words)
    }

    fn parse_brace_word(&mut self) -> Result<String, ParserError> {
        // Capture from '{' to matching '}' and return the exact substring
        let (start, _end) = self
            .lexer
            .get_span()
            .ok_or(ParserError::UnexpectedEOF)?;

        // consume '{'
        self.lexer.next();
        let mut depth: i32 = 1;
        loop {
            if self.lexer.is_eof() {
                return Err(ParserError::InvalidSyntax(
                    "Unterminated brace expansion".to_string(),
                ));
            }

            // capture current token span, then consume
            if let Some((_, end)) = self.lexer.get_span() {
                match self.lexer.peek() {
                    Some(Token::BraceOpen) => depth += 1,
                    Some(Token::BraceClose) => depth -= 1,
                    _ => {}
                }
                // consume current token
                self.lexer.next();

                if depth == 0 {
                    return Ok(self.lexer.get_text(start, end));
                }
            } else {
                return Err(ParserError::UnexpectedEOF);
            }
        }
    }

    fn parse_brace_expansion(&mut self) -> Result<Word, ParserError> {
        // Parse brace expansions like {1..5}, {a,b,c}, {prefix,{1..3},suffix}
        let (start, _) = self.lexer.get_span().ok_or(ParserError::UnexpectedEOF)?;
        
        // consume '{'
        self.lexer.next();
        
        let mut items = Vec::new();
        let mut current_item = String::new();
        let mut depth = 1;
        
        while !self.lexer.is_eof() && depth > 0 {
            if let Some((_, end)) = self.lexer.get_span() {
                match self.lexer.peek() {
                    Some(Token::BraceOpen) => {
                        depth += 1;
                        current_item.push_str(&self.lexer.get_text(start, end));
                        self.lexer.next();
                    }
                    Some(Token::BraceClose) => {
                        depth -= 1;
                        if depth == 0 {
                            // End of brace expansion
                            if !current_item.is_empty() {
                                // Check if current_item contains a range pattern like "a..c"
                                if current_item.contains("..") {
                                    let parts: Vec<&str> = current_item.split("..").collect();
                                    if parts.len() == 2 {
                                        // Check if this is a character range like "a..c"
                                        if let (Some(start_char), Some(end_char)) = (parts[0].chars().next(), parts[1].chars().next()) {
                                            if start_char.is_ascii_lowercase() && end_char.is_ascii_lowercase() {
                                                // This is a character range
                                                items.push(BraceItem::Range(BraceRange {
                                                    start: parts[0].to_string(),
                                                    end: parts[1].to_string(),
                                                    step: None,
                                                    format: None,
                                                }));
                                            } else {
                                                // This is a numeric range
                                                items.push(BraceItem::Range(BraceRange {
                                                    start: parts[0].to_string(),
                                                    end: parts[1].to_string(),
                                                    step: None,
                                                    format: None,
                                                }));
                                            }
                                        } else {
                                            // This is a numeric range
                                            items.push(BraceItem::Range(BraceRange {
                                                start: parts[0].to_string(),
                                                end: parts[1].to_string(),
                                                step: None,
                                                format: None,
                                            }));
                                        }
                                    } else if parts.len() == 3 {
                                        // This is a range with step like "00..04..2"
                                        items.push(BraceItem::Range(BraceRange {
                                            start: parts[0].to_string(),
                                            end: parts[1].to_string(),
                                            step: Some(parts[2].to_string()),
                                            format: None,
                                        }));
                                    } else {
                                        items.push(BraceItem::Literal(current_item.clone()));
                                    }
                                } else {
                                    items.push(BraceItem::Literal(current_item.clone()));
                                }
                            }
                            self.lexer.next();
                            break;
                        } else {
                            current_item.push_str(&self.lexer.get_text(start, end));
                            self.lexer.next();
                        }
                    }
                    Some(Token::Comma) if depth == 1 => {
                        // Comma separator at top level
                        if !current_item.is_empty() {
                            // Check if current_item contains a range pattern like "a..c"
                            if current_item.contains("..") {
                                let parts: Vec<&str> = current_item.split("..").collect();
                                if parts.len() == 2 {
                                    // Check if this is a character range like "a..c"
                                    if let (Some(start_char), Some(end_char)) = (parts[0].chars().next(), parts[1].chars().next()) {
                                        if start_char.is_ascii_lowercase() && end_char.is_ascii_lowercase() {
                                            // This is a character range
                                            items.push(BraceItem::Range(BraceRange {
                                                start: parts[0].to_string(),
                                                end: parts[1].to_string(),
                                                step: None,
                                                format: None,
                                            }));
                                        } else {
                                            // This is a numeric range
                                            items.push(BraceItem::Range(BraceRange {
                                                start: parts[0].to_string(),
                                                end: parts[1].to_string(),
                                                step: None,
                                                format: None,
                                            }));
                                        }
                                    } else {
                                        // This is a numeric range
                                        items.push(BraceItem::Range(BraceRange {
                                            start: parts[0].to_string(),
                                            end: parts[1].to_string(),
                                            step: None,
                                            format: None,
                                        }));
                                    }
                                } else if parts.len() == 3 {
                                    // This is a range with step like "00..04..2"
                                    items.push(BraceItem::Range(BraceRange {
                                        start: parts[0].to_string(),
                                        end: parts[1].to_string(),
                                        step: Some(parts[2].to_string()),
                                        format: None,
                                    }));
                                } else {
                                    items.push(BraceItem::Literal(current_item.clone()));
                                }
                            } else {
                                items.push(BraceItem::Literal(current_item.clone()));
                            }
                            current_item.clear();
                        }
                        self.lexer.next();
                    }
                    Some(Token::Range) if depth == 1 => {
                        // This is a range separator (e.g., 1..5 or 00..04..2)
                        if !current_item.is_empty() {
                            // This is the start of the range
                            let start_val = current_item.clone();
                            current_item.clear();
                            
                            // Consume the range token
                            self.lexer.next();
                            
                            let mut end_val = String::new();
                            while let Some(token) = self.lexer.peek() {
                                match token {
                                    Token::Comma | Token::BraceClose => break,
                                    Token::Range => {
                                        // Found another range token - this indicates a step value
                                        break;
                                    }
                                    _ => {
                                        if let Some((s, e)) = self.lexer.get_span() {
                                            end_val.push_str(&self.lexer.get_text(s, e));
                                        }
                                        self.lexer.next();
                                    }
                                }
                            }
                            
                            // Check for step value (e.g., 1..5..2)
                            let step = if let Some(Token::Range) = self.lexer.peek() {
                                self.lexer.next(); // consume the range token
                                let mut step_val = String::new();
                                while let Some(token) = self.lexer.peek() {
                                    match token {
                                        Token::Comma | Token::BraceClose => break,
                                        _ => {
                                            if let Some((s, e)) = self.lexer.get_span() {
                                                step_val.push_str(&self.lexer.get_text(s, e));
                                            }
                                            self.lexer.next();
                                        }
                                    }
                                }
                                if !step_val.is_empty() { Some(step_val) } else { None }
                            } else {
                                None
                            };
                            
                            items.push(BraceItem::Range(BraceRange {
                                start: start_val,
                                end: end_val,
                                step,
                                format: None,
                            }));
                            continue;
                        }
                    }
                    _ => {
                        if let Some((s, e)) = self.lexer.get_span() {
                            current_item.push_str(&self.lexer.get_text(s, e));
                        }
                        self.lexer.next();
                    }
                }
            } else {
                break;
            }
        }
        
        // Create the brace expansion node
        let brace_expansion = BraceExpansion {
            prefix: None,
            items,
            suffix: None,
        };
        
        Ok(Word::BraceExpansion(brace_expansion))
    }

    fn parse_arithmetic_expression(&mut self) -> Result<Word, ParserError> {
        // Parse arithmetic expressions like $((i+1))
        let mut inner_expression = String::new();
        let mut tokens = Vec::new();
        
        // We're at the Arithmetic token which is $((, consume it
        if let Some((_start, _end)) = self.lexer.get_span() {
            self.lexer.next();
        }
        
        let mut depth = 2; // We start with two open parentheses
        
        while !self.lexer.is_eof() && depth > 0 {
            if let Some((start, end)) = self.lexer.get_span() {
                let text = self.lexer.get_text(start, end);
                match self.lexer.peek() {
                    Some(Token::Arithmetic) => {
                        depth += 2;
                        tokens.push(ArithmeticToken::ParenOpen);
                        tokens.push(ArithmeticToken::ParenOpen);
                        inner_expression.push_str(&text);
                    }
                    Some(Token::ParenClose) => {
                        depth -= 1;
                        tokens.push(ArithmeticToken::ParenClose);
                        // Only include the text if we're not at the final closing parenthesis
                        // We need depth > 1 because we start with depth = 2 for $(( and need to
                        // include the first ) but not the second )
                        if depth > 1 {
                            inner_expression.push_str(&text);
                        }
                        // When depth becomes 1 or 0, we've reached the end of the arithmetic expression
                        // Don't include the final closing parentheses in the expression
                    }
                    Some(Token::Number) => {
                        tokens.push(ArithmeticToken::Number(text.clone()));
                        inner_expression.push_str(&text);
                    }
                    Some(Token::Identifier) => {
                        tokens.push(ArithmeticToken::Variable(text.clone()));
                        inner_expression.push_str(&text);
                    }
                    Some(Token::Plus) | Some(Token::Minus) | Some(Token::Star) | Some(Token::Slash) 
                    | Some(Token::Percent) | Some(Token::Caret) => {
                        tokens.push(ArithmeticToken::Operator(text.clone()));
                        inner_expression.push_str(&text);
                    }
                    _ => {
                        inner_expression.push_str(&text);
                    }
                }
                
                self.lexer.next();
            } else {
                break;
            }
        }
        
        Ok(Word::Arithmetic(ArithmeticExpression {
            expression: inner_expression,
            tokens,
        }))
    }

    fn parse_string_interpolation(&mut self) -> Result<Word, ParserError> {
        // Parse string interpolation like "I is $i" or "Result: $((i+1))"
        let mut parts = Vec::new();
        
        // Get the quoted string content
        let quoted_content = if let Some((start, end)) = self.lexer.get_span() {
            let text = self.lexer.get_text(start, end);
            self.lexer.next();
            // Strip the quotes
            if text.len() >= 2 {
                text[1..text.len()-1].to_string()
            } else {
                text
            }
        } else {
            return Err(ParserError::UnexpectedEOF);
        };
        
        // Parse the string content for variables and arithmetic
        let mut current_literal = String::new();
        let mut i = 0;
        
        while i < quoted_content.len() {
            // Use find() to locate the next $ character more efficiently
            if let Some(dollar_pos) = quoted_content[i..].find('$') {
                // Add any literal content before the $ to parts
                if dollar_pos > 0 {
                    current_literal.push_str(&quoted_content[i..i + dollar_pos]);
                }
                
                // Flush current literal if any
                if !current_literal.is_empty() {
                    parts.push(StringPart::Literal(current_literal.clone()));
                    current_literal.clear();
                }
                
                // Update position to the $ character
                i += dollar_pos;
                
                // Parse variable or arithmetic
                if quoted_content[i..].starts_with("$((") {
                    // Arithmetic expression
                    let mut depth = 2;
                    let mut expr = String::new();
                    i += 3; // Skip $(( 
                    
                    while i < quoted_content.len() && depth > 0 {
                        if quoted_content[i..].starts_with("((") {
                            depth += 2;
                            expr.push_str("((");
                            i += 2;
                        } else if quoted_content[i..].starts_with(')') {
                            depth -= 1;
                            expr.push(')');
                            i += 1;
                        } else {
                            expr.push(quoted_content.chars().nth(i).unwrap_or(' '));
                            i += 1;
                        }
                    }
                    
                    // Create a simple arithmetic expression for now
                    parts.push(StringPart::Arithmetic(ArithmeticExpression {
                        expression: format!("$(({}))", expr),
                        tokens: Vec::new(), // Simplified for now
                    }));
                } else if quoted_content[i..].starts_with('$') && i + 1 < quoted_content.len() {
                    // Variable
                    i += 1; // Skip $
                    let mut var_name = String::new();
                    
                    // Check for braced variable syntax like ${arr[1]} or ${#arr[@]}
                    if quoted_content[i..].starts_with('{') {
                        i += 1; // Skip {
                        let mut brace_depth = 1;
                        let mut brace_content = String::new();
                        
                        while i < quoted_content.len() && brace_depth > 0 {
                            let ch = quoted_content.chars().nth(i).unwrap_or(' ');
                            if ch == '{' {
                                brace_depth += 1;
                            } else if ch == '}' {
                                brace_depth -= 1;
                            }
                            if brace_depth > 0 {
                                brace_content.push(ch);
                            }
                            i += 1;
                        }
                        
                        // Check if this is a special shell array syntax like #arr[@] or !map[@]
                        if brace_content.starts_with('#') && brace_content.contains('[') {
                            // This is ${#arr[@]} - array length
                            if let Some(bracket_start) = brace_content.find('[') {
                                if let Some(_bracket_end) = brace_content.rfind(']') {
                                    let array_name = &brace_content[1..bracket_start]; // Remove # prefix
                                    // Create a MapLength StringPart
                                    parts.push(StringPart::MapLength(array_name.to_string()));
                                    continue; // Skip the regular variable handling
                                }
                            }
                            // Fallback to regular variable if parsing fails
                            var_name = brace_content;
                        } else if brace_content.starts_with('!') && brace_content.contains('[') {
                            // This is ${!map[@]} - get keys of associative array
                            if let Some(bracket_start) = brace_content.find('[') {
                                if let Some(_bracket_end) = brace_content.rfind(']') {
                                    let map_name = &brace_content[1..bracket_start]; // Remove ! prefix
                                    // Create a MapKeys StringPart
                                    parts.push(StringPart::MapKeys(map_name.to_string()));
                                    continue; // Skip the regular variable handling
                                }
                            }
                            // Fallback to regular variable if parsing fails
                            var_name = brace_content;
                        } else if brace_content.contains('[') && brace_content.contains(']') {
                            // This is a map/array access like ${map[foo]} or ${arr[1]}
                            // Parse it as a MapAccess
                            if let Some(bracket_start) = brace_content.find('[') {
                                if let Some(_bracket_end) = brace_content.rfind(']') {
                                    let map_name = &brace_content[..bracket_start];
                                    let key = &brace_content[bracket_start + 1.._bracket_end];
                                    // Create a MapAccess StringPart
                                    parts.push(StringPart::MapAccess(map_name.to_string(), key.to_string()));
                                    continue; // Skip the regular variable handling
                                }
                            }
                            // Fallback to regular variable if parsing fails
                            var_name = brace_content;
                        } else {
                            // Regular braced variable like ${arr[1]}
                            var_name = brace_content;
                        }
                    } else {
                        // Handle special shell variables like $# and $@
                        if i < quoted_content.len() {
                            let next_char = quoted_content.chars().nth(i).unwrap_or(' ');
                            if next_char == '#' || next_char == '@' {
                                // Special shell variables
                                var_name.push(next_char);
                                i += 1;
                            } else {
                                // Regular alphanumeric variables
                                while i < quoted_content.len() && quoted_content.chars().nth(i).unwrap_or(' ').is_alphanumeric() {
                                    var_name.push(quoted_content.chars().nth(i).unwrap_or(' '));
                                    i += 1;
                                }
                            }
                        }
                    }
                    
                    // Check if this is a parameter expansion with operators
                    if var_name.ends_with("^^") {
                        let base_var = var_name.trim_end_matches("^^");
                        parts.push(StringPart::ParameterExpansion(ParameterExpansion {
                            variable: base_var.to_string(),
                            operator: ParameterExpansionOperator::UppercaseAll,
                        }));
                    } else if var_name.ends_with(",,") {
                        let base_var = var_name.trim_end_matches(",,");
                        parts.push(StringPart::ParameterExpansion(ParameterExpansion {
                            variable: base_var.to_string(),
                            operator: ParameterExpansionOperator::LowercaseAll,
                        }));
                    } else if var_name.ends_with("^") && !var_name.ends_with("^^") {
                        let base_var = var_name.trim_end_matches("^");
                        parts.push(StringPart::ParameterExpansion(ParameterExpansion {
                            variable: base_var.to_string(),
                            operator: ParameterExpansionOperator::UppercaseFirst,
                        }));
                    } else if var_name.ends_with("##*/") {
                        let base_var = var_name.trim_end_matches("##*/");
                        parts.push(StringPart::ParameterExpansion(ParameterExpansion {
                            variable: base_var.to_string(),
                            operator: ParameterExpansionOperator::Basename,
                        }));
                    } else if var_name.ends_with("%/*") {
                        let base_var = var_name.trim_end_matches("%/*");
                        parts.push(StringPart::ParameterExpansion(ParameterExpansion {
                            variable: base_var.to_string(),
                            operator: ParameterExpansionOperator::Dirname,
                        }));
                    } else if var_name.contains("##") && !var_name.ends_with("##*/") {
                        let parts_split: Vec<&str> = var_name.split("##").collect();
                        if parts_split.len() == 2 {
                            parts.push(StringPart::ParameterExpansion(ParameterExpansion {
                                variable: parts_split[0].to_string(),
                                operator: ParameterExpansionOperator::RemoveLongestPrefix(parts_split[1].to_string()),
                            }));
                        } else {
                            parts.push(StringPart::Variable(var_name));
                        }
                    } else if var_name.contains("%%") && !var_name.ends_with("%/*") {
                        let parts_split: Vec<&str> = var_name.split("%%").collect();
                        if parts_split.len() == 2 {
                            parts.push(StringPart::ParameterExpansion(ParameterExpansion {
                                variable: parts_split[0].to_string(),
                                operator: ParameterExpansionOperator::RemoveLongestSuffix(parts_split[1].to_string()),
                            }));
                        } else {
                            parts.push(StringPart::Variable(var_name));
                        }
                    } else if var_name.contains("//") {
                        // For pattern substitution, we need to split on all // occurrences
                        // The format is variable//pattern/replacement
                        if let Some(first_slash) = var_name.find("//") {
                            let variable = &var_name[..first_slash];
                            let rest = &var_name[first_slash + 2..];
                            if let Some(second_slash) = rest.find('/') {
                                let pattern = &rest[..second_slash];
                                let replacement = &rest[second_slash + 1..];
                                parts.push(StringPart::ParameterExpansion(ParameterExpansion {
                                    variable: variable.to_string(),
                                    operator: ParameterExpansionOperator::SubstituteAll(pattern.to_string(), replacement.to_string()),
                                }));
                            } else {
                                parts.push(StringPart::Variable(var_name));
                            }
                        } else {
                            parts.push(StringPart::Variable(var_name));
                        }
                    } else if var_name.contains(":-") {
                        let parts_split: Vec<&str> = var_name.split(":-").collect();
                        if parts_split.len() == 2 {
                            parts.push(StringPart::ParameterExpansion(ParameterExpansion {
                                variable: parts_split[0].to_string(),
                                operator: ParameterExpansionOperator::DefaultValue(parts_split[1].to_string()),
                            }));
                        } else {
                            parts.push(StringPart::Variable(var_name));
                        }
                    } else if var_name.contains(":=") {
                        let parts_split: Vec<&str> = var_name.split(":=").collect();
                        if parts_split.len() == 2 {
                            parts.push(StringPart::ParameterExpansion(ParameterExpansion {
                                variable: parts_split[0].to_string(),
                                operator: ParameterExpansionOperator::AssignDefault(parts_split[1].to_string()),
                            }));
                        } else {
                            parts.push(StringPart::Variable(var_name));
                        }
                    } else if var_name.contains(":?") {
                        let parts_split: Vec<&str> = var_name.split(":?").collect();
                        if parts_split.len() == 2 {
                            parts.push(StringPart::ParameterExpansion(ParameterExpansion {
                                variable: parts_split[0].to_string(),
                                operator: ParameterExpansionOperator::ErrorIfUnset(parts_split[1].to_string()),
                            }));
                        } else {
                            parts.push(StringPart::Variable(var_name));
                        }
                    } else if var_name.contains("#") && !var_name.starts_with('#') {
                        let parts_split: Vec<&str> = var_name.split("#").collect();
                        if parts_split.len() == 2 {
                            parts.push(StringPart::ParameterExpansion(ParameterExpansion {
                                variable: parts_split[0].to_string(),
                                operator: ParameterExpansionOperator::RemoveShortestPrefix(parts_split[1].to_string()),
                            }));
                        } else {
                            parts.push(StringPart::Variable(var_name));
                        }
                    } else if var_name.contains("%") && !var_name.starts_with('%') {
                        let parts_split: Vec<&str> = var_name.split("%").collect();
                        if parts_split.len() == 2 {
                            parts.push(StringPart::ParameterExpansion(ParameterExpansion {
                                variable: parts_split[0].to_string(),
                                operator: ParameterExpansionOperator::RemoveShortestSuffix(parts_split[1].to_string()),
                            }));
                        } else {
                            parts.push(StringPart::Variable(var_name));
                        }
                    } else {
                        parts.push(StringPart::Variable(var_name));
                    }
                } else {
                    // Just a literal $
                    current_literal.push('$');
                    i += 1;
                }
            } else {
                // No more $ characters found, add remaining content as literal
                current_literal.push_str(&quoted_content[i..]);
                break;
            }
        }
        
        // Flush remaining literal
        if !current_literal.is_empty() {
            parts.push(StringPart::Literal(current_literal));
        }
        
        Ok(Word::StringInterpolation(StringInterpolation { parts }))
    }

    fn parse_braced_variable_name(&mut self) -> Result<String, ParserError> {
        // Parse the variable name inside ${...}
        // We need to capture the raw text from the input to preserve special characters
        let mut var_name = String::new();
        let mut depth = 1; // We start after the opening {
        

        
        // Skip the opening brace
        self.lexer.next();
        
        while !self.lexer.is_eof() && depth > 0 {
            if let Some((start, end)) = self.lexer.get_span() {
                let token = self.lexer.peek();
                
                match token {
                    Some(Token::BraceOpen) => {
                        depth += 1;
                        let text = self.lexer.get_text(start, end);
                        var_name.push_str(&text);
                        self.lexer.next();
                    }
                    Some(Token::BraceClose) => {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        } else {
                            let text = self.lexer.get_text(start, end);
                            var_name.push_str(&text);
                            self.lexer.next();
                        }
                    }
                    _ => {
                        let text = self.lexer.get_text(start, end);
                        var_name.push_str(&text);
                        self.lexer.next();
                    }
                }
            } else {
                break;
            }
        }
        
        Ok(var_name)
    }
    
    fn parse_parameter_expansion(&mut self) -> Result<Word, ParserError> {
        // Parse parameter expansion like ${name^^}, ${name,,}, etc.
        // We're already past the ${, so we need to parse the content
        

        
        // First, try to get an identifier (the variable name)
        if let Some(Token::Identifier) = self.lexer.peek() {
            let var_name = self.get_identifier_text()?;

            
            // Now check for operators
            if let Some(Token::Caret) = self.lexer.peek() {
                debug_println!("DEBUG: parse_parameter_expansion: Found first Caret");
                self.lexer.next(); // consume first ^
                if let Some(Token::Caret) = self.lexer.peek() {
                    // This is ^^ (uppercase all)
                    debug_println!("DEBUG: parse_parameter_expansion: Found second Caret, this is ^^");
                    self.lexer.next(); // consume second ^
                    
                    // Expect closing brace
                    if let Some(Token::BraceClose) = self.lexer.peek() {
                        debug_println!("DEBUG: parse_parameter_expansion: Found BraceClose, returning UppercaseAll");
                        self.lexer.next(); // consume }
                        return Ok(Word::ParameterExpansion(ParameterExpansion {
                            variable: var_name,
                            operator: ParameterExpansionOperator::UppercaseAll,
                        }));
                    } else {
                        debug_println!("DEBUG: parse_parameter_expansion: Expected BraceClose but got: {:?}", self.lexer.peek());
                    }
                } else {
                    // This is ^ (uppercase first)
                    debug_println!("DEBUG: parse_parameter_expansion: Only one Caret, this is ^");
                    // Expect closing brace
                    if let Some(Token::BraceClose) = self.lexer.peek() {
                        debug_println!("DEBUG: parse_parameter_expansion: Found BraceClose, returning UppercaseFirst");
                        self.lexer.next(); // consume }
                        return Ok(Word::ParameterExpansion(ParameterExpansion {
                            variable: var_name,
                            operator: ParameterExpansionOperator::UppercaseFirst,
                        }));
                    } else {
                        debug_println!("DEBUG: parse_parameter_expansion: Expected BraceClose but got: {:?}", self.lexer.peek());
                    }
                }
            } else if let Some(Token::Comma) = self.lexer.peek() {
                debug_println!("DEBUG: parse_parameter_expansion: Found first Comma");
                self.lexer.next(); // consume first ,
                if let Some(Token::Comma) = self.lexer.peek() {
                    // This is ,, (lowercase all)
                    debug_println!("DEBUG: parse_parameter_expansion: Found second Comma, this is ,,");
                    self.lexer.next(); // consume second ,
                    
                    // Expect closing brace
                    if let Some(Token::BraceClose) = self.lexer.peek() {
                        debug_println!("DEBUG: parse_parameter_expansion: Found BraceClose, returning LowercaseAll");
                        self.lexer.next(); // consume }
                        return Ok(Word::ParameterExpansion(ParameterExpansion {
                            variable: var_name,
                            operator: ParameterExpansionOperator::LowercaseAll,
                        }));
                    } else {
                        debug_println!("DEBUG: parse_parameter_expansion: Expected BraceClose but got: {:?}", self.lexer.peek());
                    }
                }
            } else {
                debug_println!("DEBUG: parse_parameter_expansion: No operator found, got: {:?}", self.lexer.peek());
            }
        } else {
            debug_println!("DEBUG: parse_parameter_expansion: No Identifier found, got: {:?}", self.lexer.peek());
        }
        
        // If we get here, it's not a simple parameter expansion
        debug_println!("DEBUG: parse_parameter_expansion: Returning error");
        Err(ParserError::InvalidSyntax("Not a valid parameter expansion".to_string()))
    }

    fn parse_command_substitution(&mut self) -> Result<Word, ParserError> {
        // Parse command substitution $(...)
        // We're already at the DollarParen token, so consume it
        self.lexer.next();
        
        let mut depth = 1; // We start after the opening (
        let mut command_text = String::new();
        
        while !self.lexer.is_eof() && depth > 0 {
            if let Some((start, end)) = self.lexer.get_span() {
                match self.lexer.peek() {
                    Some(Token::DollarParen) => {
                        depth += 1;
                        command_text.push_str(&self.lexer.get_text(start, end));
                    }
                    Some(Token::ParenClose) => {
                        depth -= 1;
                        if depth == 0 {
                            // Consume the closing parenthesis before breaking
                            self.lexer.next();
                            break;
                        } else {
                            command_text.push_str(&self.lexer.get_text(start, end));
                        }
                    }
                    _ => {
                        command_text.push_str(&self.lexer.get_text(start, end));
                    }
                }
                self.lexer.next();
            } else {
                break;
            }
        }
        
        // Create a simple command from the captured text and wrap it in CommandSubstitution
        let command = Command::Simple(SimpleCommand {
            name: Word::Literal("command_substitution".to_string()),
            args: vec![Word::Literal(command_text)],
            redirects: Vec::new(),
            env_vars: HashMap::new(),
        });
        
        Ok(Word::CommandSubstitution(Box::new(command)))
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

    fn get_raw_token_text(&mut self) -> Result<String, ParserError> {
        if let Some(span) = self.lexer.get_span() {
            let text = self.lexer.get_text(span.0, span.1);
            self.lexer.next(); // consume the token
            Ok(text)
        } else {
            Err(ParserError::UnexpectedEOF)
        }
    }

    fn capture_arithmetic_text(&mut self) -> Result<String, ParserError> {
        // We are at $((' start. Capture until matching '))'
        // The lexer token for start is DollarParen for $( and Arithmetic for $((
        // Our lexer provides Arithmetic for '$((' specifically.
        let mut text = String::new();
        if let Some((start, end)) = self.lexer.get_span() {
            text.push_str(&self.lexer.get_text(start, end));
            self.lexer.next(); // consume $(('
        } else {
            return Err(ParserError::UnexpectedEOF);
        }

        // Arithmetic token corresponds to '$((' which is two opening parens
        let mut depth: i32 = 2;
        while !self.lexer.is_eof() && depth > 0 {
            if let Some((start, end)) = self.lexer.get_span() {
                match self.lexer.peek() {
                    Some(Token::Arithmetic) => { 
                        depth += 2; 
                        text.push_str(&self.lexer.get_text(start, end));
                    }
                    Some(Token::ParenClose) => {
                        depth -= 1;
                        text.push_str(&self.lexer.get_text(start, end));
                    }
                    _ => {
                        text.push_str(&self.lexer.get_text(start, end));
                    }
                }
                self.lexer.next();
            } else {
                break;
            }
        }
        Ok(text)
    }

    fn parse_ansic_quoted_string(&mut self) -> Result<Word, ParserError> {
        // Parse ANSI-C quoting strings like $'line1\nline2\tTabbed'
        let raw_text = self.get_raw_token_text()?;
        
        // Remove the $' and ' wrapper
        if raw_text.len() >= 3 && raw_text.starts_with("$'") && raw_text.ends_with("'") {
            let content = &raw_text[2..raw_text.len()-1];
            
            // Process ANSI-C escape sequences
            let mut result = String::new();
            let mut chars = content.chars().peekable();
            
            while let Some(ch) = chars.next() {
                if ch == '\\' {
                    if let Some(next_ch) = chars.next() {
                        match next_ch {
                            'a' => result.push('\x07'), // bell
                            'b' => result.push('\x08'), // backspace
                            'f' => result.push('\x0c'), // formfeed
                            'n' => result.push('\n'),   // newline
                            'r' => result.push('\r'),   // carriage return
                            't' => result.push('\t'),   // tab
                            'v' => result.push('\x0b'), // vertical tab
                            '\\' => result.push('\\'),  // backslash
                            '\'' => result.push('\''),  // single quote
                            '?' => result.push('?'),    // question mark
                            'x' => {
                                // Hex escape: \xHH
                                let mut hex = String::new();
                                for _ in 0..2 {
                                    if let Some(hex_ch) = chars.next() {
                                        if hex_ch.is_ascii_hexdigit() {
                                            hex.push(hex_ch);
                                        } else {
                                            break;
                                        }
                                    } else {
                                        break;
                                    }
                                }
                                if hex.len() == 2 {
                                    if let Ok(byte_val) = u8::from_str_radix(&hex, 16) {
                                        result.push(byte_val as char);
                                    } else {
                                        result.push_str(&format!("\\x{}", hex));
                                    }
                                } else {
                                    result.push_str(&format!("\\x{}", hex));
                                }
                            }
                            'u' => {
                                // Unicode escape: \uHHHH
                                let mut hex = String::new();
                                for _ in 0..4 {
                                    if let Some(hex_ch) = chars.next() {
                                        if hex_ch.is_ascii_hexdigit() {
                                            hex.push(hex_ch);
                                        } else {
                                            break;
                                        }
                                    } else {
                                        break;
                                    }
                                }
                                if hex.len() == 4 {
                                    if let Ok(unicode_val) = u32::from_str_radix(&hex, 16) {
                                        if let Some(unicode_char) = char::from_u32(unicode_val) {
                                            result.push(unicode_char);
                                        } else {
                                            result.push_str(&format!("\\u{}", hex));
                                        }
                                    } else {
                                        result.push_str(&format!("\\u{}", hex));
                                    }
                                } else {
                                    result.push_str(&format!("\\u{}", hex));
                                }
                            }
                            _ => {
                                // Unknown escape sequence, treat as literal
                                result.push('\\');
                                result.push(next_ch);
                            }
                        }
                    } else {
                        // Trailing backslash
                        result.push('\\');
                    }
                } else {
                    result.push(ch);
                }
            }
            
            Ok(Word::Literal(result))
        } else {
            // Fallback: treat as literal if format is unexpected
            Ok(Word::Literal(raw_text))
        }
    }

    fn capture_parenthetical_text(&mut self) -> Result<String, ParserError> {
        // Assumes current token is '(' or we are right before it (when called after consuming '<' or '>')
        if !matches!(self.lexer.peek(), Some(Token::ParenOpen)) {
            // If not at '(', just parse a word
            let word = self.parse_word()?;
            return Ok(word.to_string());
        }
        let (start, _end) = self.lexer.get_span().ok_or(ParserError::UnexpectedEOF)?;
        let mut depth: i32 = 0;
        loop {
            if let Some((_, end)) = self.lexer.get_span() {
                match self.lexer.peek() {
                    Some(Token::ParenOpen) => depth += 1,
                    Some(Token::ParenClose) => depth -= 1,
                    _ => {}
                }
                self.lexer.next();
                if depth == 0 { return Ok(self.lexer.get_text(start, end)); }
            } else { return Err(ParserError::UnexpectedEOF); }
        }
    }

    fn capture_double_bracket_expression(&mut self) -> Result<String, ParserError> {
        // Capture raw text until we encounter a closing ']]'.
        let mut expr = String::new();
        loop {
            match self.lexer.peek() {
                Some(Token::TestBracketClose) if matches!(self.lexer.peek_n(1), Some(Token::TestBracketClose)) => {
                    // consume the closing ']]' and stop
                    self.lexer.next();
                    self.lexer.next();
                    break;
                }
                Some(_) => {
                    if let Some((s, e)) = self.lexer.get_span() {
                        expr.push_str(&self.lexer.get_text(s, e));
                    }
                    self.lexer.next();
                }
                None => {
                    // Unterminated [[ ...  ; treat as whatever we collected
                    break;
                }
            }
        }
        Ok(expr.trim().to_string())
    }

    fn parse_double_paren_command(&mut self) -> Result<Command, ParserError> {
        // Consume two opening parens
        self.lexer.consume(Token::ParenOpen)?;
        self.lexer.consume(Token::ParenOpen)?;
        // Capture until matching '))'
        let mut depth: i32 = 2;
        let mut expr = String::new();
        while !self.lexer.is_eof() && depth > 0 {
            if let Some((start, end)) = self.lexer.get_span() {
                match self.lexer.peek() {
                    Some(Token::ParenOpen) => { depth += 1; }
                    Some(Token::ParenClose) => { depth -= 1; }
                    _ => {}
                }
                let seg = self.lexer.get_text(start, end);
                self.lexer.next();
                if depth >= 0 { expr.push_str(&seg); }
            } else { break; }
        }
                 Ok(Command::Simple(SimpleCommand { name: Word::Literal("((".to_string()), args: vec![Word::Literal(expr.trim().to_string())], redirects: Vec::new(), env_vars: HashMap::new() }))
    }

    fn skip_whitespace_and_comments(&mut self) {
        while let Some(token) = self.lexer.peek() {
            match token {
                Token::Space | Token::Tab | Token::Comment => {
                    self.lexer.next();
                }
                _ => break,
            }
        }
    }

    fn parse_environment_variable_value(&mut self) -> Result<Word, ParserError> {
        if let Some(tok) = self.lexer.peek() {
            match tok {
                Token::Arithmetic => {
                    // Parse arithmetic expression properly
                    self.parse_arithmetic_expression()
                }
                Token::DollarParen => {
                    // Parse variable expansion
                    self.parse_variable_expansion()
                }
                Token::ParenOpen => {
                    // Parse parenthetical text as a literal
                    let text = self.capture_parenthetical_text()?;
                    Ok(Word::Literal(text))
                }
                Token::DoubleQuotedString | Token::SingleQuotedString => {
                    // Parse quoted string as a literal
                    let text = self.get_string_text()?;
                    Ok(Word::Literal(text))
                }
                Token::BacktickString => {
                    // Parse backtick string as a literal
                    let text = self.get_raw_token_text()?;
                    Ok(Word::Literal(text))
                }
                _ => {
                    // Parse as a literal string until separator
                    let mut value = String::new();
                    loop {
                        match self.lexer.peek() {
                            Some(Token::Space) | Some(Token::Tab) | Some(Token::Newline) | Some(Token::Semicolon) | None => break,
                            Some(Token::Arithmetic) => {
                                // Parse arithmetic expression properly
                                return self.parse_arithmetic_expression();
                            }
                            Some(Token::DollarParen) => {
                                // Parse variable expansion
                                return self.parse_variable_expansion();
                            }
                            Some(Token::ParenOpen) => {
                                // Parse parenthetical text as a literal
                                let text = self.capture_parenthetical_text()?;
                                value.push_str(&text);
                            }
                            _ => {
                                if let Some((start, end)) = self.lexer.get_span() {
                                    value.push_str(&self.lexer.get_text(start, end));
                                    self.lexer.next();
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
        self.shopt_state.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_command() {
        let mut parser = Parser::new("echo hello world");
        let commands = parser.parse().unwrap();
        assert_eq!(commands.len(), 1);
        if let Command::Simple(cmd) = &commands[0] {
            assert!(matches!(&cmd.name, Word::Literal(name) if name == "echo"));
            assert_eq!(cmd.args.len(), 2);
            assert!(matches!(&cmd.args[0], Word::Literal(arg) if arg == "hello"));
            assert!(matches!(&cmd.args[1], Word::Literal(arg) if arg == "world"));
        } else {
            panic!("Expected Simple command");
        }
    }

    #[test]
    fn test_parse_pipeline() {
        let mut parser = Parser::new("ls | grep test");
        let commands = parser.parse().unwrap();
        assert_eq!(commands.len(), 1);
        if let Command::Pipeline(pipeline) = &commands[0] {
            assert_eq!(pipeline.commands.len(), 2);
            assert_eq!(pipeline.operators.len(), 1);
        } else {
            panic!("Expected Pipeline command");
        }
    }

    #[test]
    fn test_parse_if_statement() {
        let mut parser = Parser::new("if [ -f file ]; then echo exists; fi");
        let commands = parser.parse().unwrap();
        assert_eq!(commands.len(), 1);
        if let Command::If(_) = &commands[0] {
            // Successfully parsed if statement
        } else {
            panic!("Expected If command");
        }
    }

    #[test]
    fn test_parse_brace_expansion() {
        let mut parser = Parser::new("echo {1..5}");
        let commands = parser.parse().unwrap();
        assert_eq!(commands.len(), 1);
        if let Command::Simple(cmd) = &commands[0] {
            assert!(matches!(&cmd.name, Word::Literal(name) if name == "echo"));
            assert_eq!(cmd.args.len(), 1);
            if let Word::BraceExpansion(expansion) = &cmd.args[0] {
                assert_eq!(expansion.items.len(), 1);
                if let BraceItem::Range(range) = &expansion.items[0] {
                    assert_eq!(range.start, "1");
                    assert_eq!(range.end, "5");
                    assert!(range.step.is_none());
                } else {
                    panic!("Expected Range item");
                }
            } else {
                panic!("Expected BraceExpansion word");
            }
        } else {
            panic!("Expected Simple command");
        }
    }

    #[test]
    fn test_parse_arithmetic_expression() {
        let mut parser = Parser::new("echo $((i+1))");
        let commands = parser.parse().unwrap();
        assert_eq!(commands.len(), 1);
        if let Command::Simple(cmd) = &commands[0] {
            assert!(matches!(&cmd.name, Word::Literal(name) if name == "echo"));
            assert_eq!(cmd.args.len(), 1);
            if let Word::Arithmetic(arithmetic) = &cmd.args[0] {
                assert!(arithmetic.expression.contains("i+1"));
                assert!(!arithmetic.tokens.is_empty());
            } else {
                panic!("Expected Arithmetic word");
            }
        } else {
            panic!("Expected Simple command");
        }
    }

    #[test]
    fn test_parse_string_interpolation() {
        let mut parser = Parser::new("echo \"I is $i\"");
        let commands = parser.parse().unwrap();
        assert_eq!(commands.len(), 1);
        if let Command::Simple(cmd) = &commands[0] {
            assert!(matches!(&cmd.name, Word::Literal(name) if name == "echo"));
            assert_eq!(cmd.args.len(), 1);
            if let Word::StringInterpolation(interpolation) = &cmd.args[0] {
                assert_eq!(interpolation.parts.len(), 2);
                assert!(matches!(&interpolation.parts[0], StringPart::Literal(s) if s == "I is "));
                assert!(matches!(&interpolation.parts[1], StringPart::Variable(s) if s == "i"));
            } else {
                panic!("Expected StringInterpolation word");
            }
        } else {
            panic!("Expected Simple command");
        }
    }

    #[test]
    fn test_parse_variable_expansion() {
        let mut parser = Parser::new("echo $HOME");
        let commands = parser.parse().unwrap();
        assert_eq!(commands.len(), 1);
        if let Command::Simple(cmd) = &commands[0] {
            assert!(matches!(&cmd.name, Word::Literal(name) if name == "echo"));
            assert_eq!(cmd.args.len(), 1);
            if let Word::Variable(var) = &cmd.args[0] {
                assert_eq!(var, "HOME");
            } else {
                panic!("Expected Variable word");
            }
        } else {
            panic!("Expected Simple command");
        }
    }

    #[test]
    fn test_parse_command_substitution() {
        let mut parser = Parser::new("echo $(ls)");
        let commands = parser.parse().unwrap();
        assert_eq!(commands.len(), 1);
        if let Command::Simple(cmd) = &commands[0] {
            assert!(matches!(&cmd.name, Word::Literal(name) if name == "echo"));
            assert_eq!(cmd.args.len(), 1);
            if let Word::CommandSubstitution(_) = &cmd.args[0] {
                // Successfully parsed command substitution
            } else {
                panic!("Expected CommandSubstitution word");
            }
        } else {
            panic!("Expected Simple command");
        }
    }

    #[test]
    fn test_debug_simple_command_substitution() {
        let input = "echo $(ls)";
        println!("Input: '{}'", input);
        println!("Input length: {}", input.len());
        println!("Input bytes: {:?}", input.as_bytes());
        
        let mut parser = Parser::new(input);
        let commands = parser.parse().unwrap();
        
        println!("Number of commands: {}", commands.len());
        for (i, cmd) in commands.iter().enumerate() {
            println!("Command {}: {:?}", i, cmd);
        }
        
        assert_eq!(commands.len(), 1);
    }
} 