use crate::ast::*;
use crate::lexer::{Lexer, Token, LexerError};
use thiserror::Error;
use std::collections::HashMap;

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
}

impl Parser {
    pub fn new(input: &str) -> Self {
        Self {
            lexer: Lexer::new(input),
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
        let mut commands = Vec::new();
        let mut operators = Vec::new();
        
        commands.push(self.parse_simple_command()?);
        // Allow whitespace/comments between command and pipeline operators
        self.skip_whitespace_and_comments();
        
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
                    break;
                }
                _ => break,
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
                        // Capture value tokens until a separator (space, tab, newline, semicolon)
                        // Special cases: arithmetic $((...)), quoted strings, backticks
                        let mut value = String::new();
                        if let Some(tok) = self.lexer.peek() {
                            match tok {
                                Token::Arithmetic => {
                                    value.push_str(&self.capture_arithmetic_text()?);
                                }
                                Token::DollarParen => {
                                    value.push_str(&self.parse_variable_expansion()?);
                                }
                                Token::ParenOpen => {
                                    value.push_str(&self.capture_parenthetical_text()?);
                                }
                                Token::DoubleQuotedString | Token::SingleQuotedString => {
                                    value.push_str(&self.get_string_text()?);
                                }
                                Token::BacktickString => {
                                    value.push_str(&self.get_raw_token_text()?);
                                }
                                _ => {
                                    loop {
                                        match self.lexer.peek() {
                                            Some(Token::Space) | Some(Token::Tab) | Some(Token::Newline) | Some(Token::Semicolon) | None => break,
                                            Some(Token::Arithmetic) => {
                                                value.push_str(&self.capture_arithmetic_text()?);
                                            }
                                            Some(Token::DollarParen) => {
                                                value.push_str(&self.parse_variable_expansion()?);
                                            }
                                            Some(Token::ParenOpen) => {
                                                value.push_str(&self.capture_parenthetical_text()?);
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
                                }
                            }
                        }
                        env_vars.insert(var_name, value);
                        
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
                                    let mut value = String::new();
                                    self.skip_whitespace_and_comments();
                                    if let Some(tok) = self.lexer.peek() {
                                        match tok {
                                            Token::Arithmetic => { value.push_str(&self.capture_arithmetic_text()?); }
                                            Token::DollarParen => { value.push_str(&self.parse_variable_expansion()?); }
                                            Token::ParenOpen => { value.push_str(&self.capture_parenthetical_text()?); }
                                            Token::DoubleQuotedString | Token::SingleQuotedString => { value.push_str(&self.get_string_text()?); }
                                            Token::BacktickString => { value.push_str(&self.get_raw_token_text()?); }
                                            _ => {
                                                loop {
                                                    match self.lexer.peek() {
                                                        Some(Token::Space) | Some(Token::Tab) | Some(Token::Newline) | Some(Token::Semicolon) | None => break,
                                                        Some(Token::Arithmetic) => { value.push_str(&self.capture_arithmetic_text()?); }
                                                        Some(Token::DollarParen) => { value.push_str(&self.parse_variable_expansion()?); }
                                                        Some(Token::ParenOpen) => { value.push_str(&self.capture_parenthetical_text()?); }
                                                        _ => {
                                                            if let Some((s2, e2)) = self.lexer.get_span() {
                                                                value.push_str(&self.lexer.get_text(s2, e2));
                                                                self.lexer.next();
                                                            } else { break; }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    env_vars.insert(name_text, value);
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
                    self.get_identifier_text()?
                }
                Token::Set | Token::Export | Token::Readonly | Token::Local | Token::Declare | Token::Typeset |
                Token::Unset | Token::Shift | Token::Eval | Token::Exec | Token::Source | Token::Trap | Token::Wait => {
                    self.get_raw_token_text()? // keep exact text for builtin/keyword-as-command
                }
                Token::TestBracket => {
                    self.lexer.next(); // consume the first [
                    if let Some(Token::TestBracket) = self.lexer.peek() {
                        self.lexer.next(); // consume the second [
                        is_double_bracket = true;
                        "[[".to_string()
                    } else {
                        "[".to_string()
                    }
                }
                Token::True => {
                    self.lexer.next(); // consume true
                    "true".to_string()
                }
                Token::False => {
                    self.lexer.next(); // consume false
                    "false".to_string()
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
                            "true".to_string()
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
            return Ok(Command::Simple(SimpleCommand {
                name,
                args: if expr.is_empty() { Vec::new() } else { vec![expr] },
                redirects,
                env_vars,
            }));
        }

        // Parse arguments and redirects
        while let Some(token) = self.lexer.peek() {
            match token {
                Token::Identifier | Token::Number | Token::DoubleQuotedString | Token::SingleQuotedString | Token::SourceDot | Token::BraceOpen | Token::BacktickString | Token::DollarSingleQuotedString | Token::DollarDoubleQuotedString => {
                    args.push(self.parse_word()?);
                }
                Token::Dollar | Token::DollarBrace | Token::DollarParen | Token::DollarHashSimple | Token::DollarAtSimple | Token::DollarStarSimple
                | Token::DollarBraceHash | Token::DollarBraceBang | Token::DollarBraceStar | Token::DollarBraceAt
                | Token::DollarBraceHashStar | Token::DollarBraceHashAt | Token::DollarBraceBangStar | Token::DollarBraceBangAt => {
                    args.push(self.parse_variable_expansion()?);
                }
                Token::Arithmetic => {
                    let txt = self.capture_arithmetic_text()?;
                    args.push(txt);
                }
                Token::Minus => {
                    // Handle arguments starting with minus (like -la, -v, etc.)
                    let token_clone = token.clone();
                    self.lexer.next(); // consume the minus
                    if let Some(Token::Identifier) = self.lexer.peek() {
                        let arg = format!("-{}", self.get_identifier_text()?);
                        args.push(arg);
                    } else if let Some(Token::Number) = self.lexer.peek() {
                        let num = self.get_number_text()?;
                        args.push(format!("-{}", num));
                    } else {
                        return Err(ParserError::UnexpectedToken { token: token_clone, line: 1, col: 1 });
                    }
                }
                // Process substitution as args: <(cmd) or >(cmd)
                Token::RedirectIn => {
                    if matches!(self.lexer.peek_n(1), Some(Token::ParenOpen)) {
                        self.lexer.next(); // consume '<'
                        let inner = self.capture_parenthetical_text()?;
                        args.push(format!("<{}", inner));
                    } else {
                        redirects.push(self.parse_redirect()?);
                    }
                }
                Token::RedirectOut => {
                    if matches!(self.lexer.peek_n(1), Some(Token::ParenOpen)) {
                        self.lexer.next(); // consume '>'
                        let inner = self.capture_parenthetical_text()?;
                        args.push(format!(">{}", inner));
                    } else {
                        redirects.push(self.parse_redirect()?);
                    }
                }
                Token::NonZero => {
                    // Handle -n argument
                    self.lexer.next(); // consume the NonZero token
                    if let Some(Token::Identifier) = self.lexer.peek() {
                        let arg = format!("-n{}", self.get_identifier_text()?);
                        args.push(arg);
                    } else {
                        args.push("-n".to_string());
                    }
                }
                Token::Character => {
                    // Handle -c argument
                    self.lexer.next(); // consume the Character token
                    if let Some(Token::Identifier) = self.lexer.peek() {
                        let arg = format!("-c{}", self.get_identifier_text()?);
                        args.push(arg);
                    } else {
                        args.push("-c".to_string());
                    }
                }
                Token::File => {
                    // Handle -f argument
                    self.lexer.next(); // consume the File token
                    args.push("-f".to_string());
                }
                Token::Directory => {
                    // Handle -d argument
                    self.lexer.next(); // consume the Directory token
                    args.push("-d".to_string());
                }
                Token::Exists => {
                    // Handle -e argument
                    self.lexer.next(); // consume the Exists token
                    args.push("-e".to_string());
                }
                Token::Readable => {
                    // Handle -r argument
                    self.lexer.next(); // consume the Readable token
                    args.push("-r".to_string());
                }
                Token::Writable => {
                    // Handle -w argument
                    self.lexer.next(); // consume the Writable token
                    args.push("-w".to_string());
                }
                Token::Executable => {
                    // Handle -x argument
                    self.lexer.next(); // consume the Executable token
                    args.push("-x".to_string());
                }
                Token::Size => { self.lexer.next(); args.push("-s".to_string()); }
                Token::Symlink => { self.lexer.next(); args.push("-L".to_string()); }
                Token::SymlinkH => { self.lexer.next(); args.push("-h".to_string()); }
                Token::PipeFile => { self.lexer.next(); args.push("-p".to_string()); }
                Token::Socket => { self.lexer.next(); args.push("-S".to_string()); }
                Token::Block => { self.lexer.next(); args.push("-b".to_string()); }
                Token::SetGid => { self.lexer.next(); args.push("-g".to_string()); }
                Token::Sticky => { self.lexer.next(); args.push("-k".to_string()); }
                Token::SetUid => { self.lexer.next(); args.push("-u".to_string()); }
                Token::Owned => { self.lexer.next(); args.push("-O".to_string()); }
                Token::GroupOwned => { self.lexer.next(); args.push("-G".to_string()); }
                Token::Modified => { self.lexer.next(); args.push("-N".to_string()); }
                // Test comparison operators
                Token::Eq => { self.lexer.next(); args.push("-eq".to_string()); }
                Token::Ne => { self.lexer.next(); args.push("-ne".to_string()); }
                Token::Lt => { self.lexer.next(); args.push("-lt".to_string()); }
                Token::Le => { self.lexer.next(); args.push("-le".to_string()); }
                Token::Gt => { self.lexer.next(); args.push("-gt".to_string()); }
                Token::Ge => { self.lexer.next(); args.push("-ge".to_string()); }
                Token::NewerThan => { self.lexer.next(); args.push("-nt".to_string()); }
                Token::OlderThan => { self.lexer.next(); args.push("-ot".to_string()); }
                Token::SameFile => { self.lexer.next(); args.push("-ef".to_string()); }
                Token::Zero => { self.lexer.next(); args.push("-z".to_string()); }
                Token::RedirectAppend | Token::RedirectInOut | Token::Heredoc | Token::HeredocTabs => {
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
            if !trimmed.is_empty() { args.push(trimmed); }
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

        // Parse body (first command)
        let body = Box::new(self.parse_command()?);

        // Consume additional commands in body until 'done' (discard for now)
        loop {
            // Skip separators
            while matches!(self.lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline | Token::CarriageReturn | Token::Semicolon)) {
                self.lexer.next();
            }
            match self.lexer.peek() {
                Some(Token::Done) | None => break,
                _ => {
                    // Parse and discard extra commands in the loop body
                    let pre_pos = self.lexer.current_position();
                    let _ = self.parse_command()?;
                    // Progress guard
                    if self.lexer.current_position() == pre_pos {
                        // Consume one token to avoid infinite loop
                        if self.lexer.next().is_none() { break; }
                    }
                }
            }
        }

        // Allow optional separator after body before 'done'
        loop {
            match self.lexer.peek() {
                Some(Token::Space) | Some(Token::Tab) | Some(Token::Comment) | Some(Token::Newline) | Some(Token::CarriageReturn) => { self.lexer.next(); continue; }
                Some(Token::Semicolon) => { self.lexer.next(); continue; }
                _ => {}
            }
            break;
        }

        // Expect 'done'
        self.lexer.consume(Token::Done)?;
        
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
        let body = Box::new(self.parse_command()?);

        // Consume additional commands in body until 'done' (discard for now)
        loop {
            // Skip separators
            while matches!(self.lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline | Token::CarriageReturn | Token::Semicolon)) {
                self.lexer.next();
            }
            match self.lexer.peek() {
                Some(Token::Done) | None => break,
                _ => {
                    // Parse and discard extra commands in the loop body
                    let pre_pos = self.lexer.current_position();
                    let _ = self.parse_command()?;
                    if self.lexer.current_position() == pre_pos {
                        if self.lexer.next().is_none() { break; }
                    }
                }
            }
        }

        // Allow optional separator after body before 'done'
        loop {
            match self.lexer.peek() {
                Some(Token::Space) | Some(Token::Tab) | Some(Token::Comment) | Some(Token::Newline) | Some(Token::CarriageReturn) => {
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
        
        Ok(Command::For(ForLoop {
            variable,
            items,
            body,
        }))
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
            // Parse first command as the body node
            let first = Box::new(self.parse_command()?);

            // Consume any additional commands inside the block (ignored for now)
        loop {
                // Skip separators
                while matches!(self.lexer.peek(), Some(Token::Space | Token::Tab | Token::Comment | Token::Newline | Token::Semicolon)) {
                    self.lexer.next();
                }
                match self.lexer.peek() {
                    Some(Token::BraceClose) | None => break,
                _ => {
                    let pre_pos = self.lexer.current_position();
                    let _ = self.parse_command()?;
                    if self.lexer.current_position() == pre_pos {
                        if self.lexer.next().is_none() { break; }
                    }
                }
                }
            }

            // Expect closing '}'
            self.lexer.consume(Token::BraceClose)?;
            first
        } else {
            // Fallback: parse next as a single command body
            Box::new(self.parse_command()?)
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
            _ => return Err(ParserError::InvalidSyntax("Invalid redirect operator".to_string())),
        };
        // Here-string: '<<< word' often lexes as '<<' '<' then word; accept optional extra '<'
        if matches!(operator, RedirectOperator::Heredoc) {
            if let Some(Token::RedirectIn) = self.lexer.peek() { self.lexer.next(); }
        }
        // Skip whitespace before target
        self.skip_whitespace_and_comments();

        // Process substitution as redirect target, allowing an optional extra '<' before '('
        let target = if matches!(self.lexer.peek(), Some(Token::RedirectIn)) && matches!(self.lexer.peek_n(1), Some(Token::ParenOpen)) {
            // consume the extra '<' and capture ( ... )
            self.lexer.next();
            self.capture_parenthetical_text()?
        } else if matches!(self.lexer.peek(), Some(Token::ParenOpen)) {
            self.capture_parenthetical_text()?
        } else {
            self.parse_word()?
        };
        // If this is a heredoc, capture lines until the delimiter is found at start of line
        let heredoc_body = match operator {
            RedirectOperator::Heredoc | RedirectOperator::HeredocTabs => {
                let delim = target.clone();
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
            _ => None,
        };

        Ok(Redirect { fd, operator, target, heredoc_body })
    }

    fn parse_word(&mut self) -> Result<String, ParserError> {
        let result = match self.lexer.peek() {
            Some(Token::Identifier) => Ok(self.get_identifier_text()?),
            Some(Token::Number) => Ok(self.get_number_text()?),
            Some(Token::DoubleQuotedString) => Ok(self.get_string_text()?),
            Some(Token::SingleQuotedString) => Ok(self.get_string_text()?),
            Some(Token::BacktickString) => Ok(self.get_raw_token_text()?),
            Some(Token::DollarSingleQuotedString) => Ok(self.get_raw_token_text()?),
            Some(Token::DollarDoubleQuotedString) => Ok(self.get_raw_token_text()?),
            Some(Token::BraceOpen) => Ok(self.parse_brace_word()?),
            Some(Token::SourceDot) => {
                // Treat standalone '.' as a normal word (e.g., `find . -name ...`)
                self.lexer.next();
                Ok(".".to_string())
            }
            Some(Token::Dollar) => Ok(self.parse_variable_expansion()?),
            Some(Token::DollarBrace) | Some(Token::DollarParen) | Some(Token::DollarHashSimple) | Some(Token::DollarAtSimple) | Some(Token::DollarStarSimple)
            | Some(Token::DollarBraceHash) | Some(Token::DollarBraceBang) | Some(Token::DollarBraceStar) | Some(Token::DollarBraceAt)
            | Some(Token::DollarBraceHashStar) | Some(Token::DollarBraceHashAt) | Some(Token::DollarBraceBangStar) | Some(Token::DollarBraceBangAt)
                => Ok(self.parse_variable_expansion()?),
            Some(Token::Arithmetic) => Ok(self.capture_arithmetic_text()?),
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

    fn parse_variable_expansion(&mut self) -> Result<String, ParserError> {
        // This is a simplified version - in a real implementation,
        // you'd need to handle the full complexity of shell variable expansion
        let mut result = String::new();
        
        match self.lexer.peek() {
            Some(Token::Dollar) => {
                self.lexer.next();
                if let Some(Token::Identifier) = self.lexer.peek() {
                    result.push_str(&format!("${}", self.get_identifier_text()?));
                }
            }
            Some(Token::DollarHashSimple) => { self.lexer.next(); result.push_str("$#"); }
            Some(Token::DollarAtSimple) => { self.lexer.next(); result.push_str("$@"); }
            Some(Token::DollarStarSimple) => { self.lexer.next(); result.push_str("$*"); }
            Some(Token::DollarBrace)
            | Some(Token::DollarBraceHash) | Some(Token::DollarBraceBang) | Some(Token::DollarBraceStar) | Some(Token::DollarBraceAt)
            | Some(Token::DollarBraceHashStar) | Some(Token::DollarBraceHashAt) | Some(Token::DollarBraceBangStar) | Some(Token::DollarBraceBangAt) => {
                // Capture raw from '${' to matching '}' inclusively
                if let Some((start, _)) = self.lexer.get_span() {
                    let mut depth: i32 = 0;
                    loop {
                        if let Some((_, end)) = self.lexer.get_span() {
                            match self.lexer.peek() {
                                Some(Token::DollarBrace)
                                | Some(Token::DollarBraceHash) | Some(Token::DollarBraceBang) | Some(Token::DollarBraceStar) | Some(Token::DollarBraceAt)
                                | Some(Token::DollarBraceHashStar) | Some(Token::DollarBraceHashAt) | Some(Token::DollarBraceBangStar) | Some(Token::DollarBraceBangAt)
                                | Some(Token::BraceOpen) => { depth += 1; }
                                Some(Token::BraceClose) => { depth -= 1; }
                                _ => {}
                            }
                            let seg = self.lexer.get_text(start, end);
                            self.lexer.next();
                            result.push_str(&seg);
                            if depth == 0 { break; }
                        } else { break; }
                    }
                }
            }
            Some(Token::DollarParen) => {
                self.lexer.next();
                // Parse command substitution: capture until matching ')'
                result.push_str("$(");
                let mut depth: i32 = 1;
                while !self.lexer.is_eof() && depth > 0 {
                    if let Some((start, end)) = self.lexer.get_span() {
                        match self.lexer.peek() {
                            Some(Token::DollarParen) => { depth += 1; }
                            Some(Token::ParenClose) => { depth -= 1; }
                            _ => {}
                        }
                        let text = self.lexer.get_text(start, end);
                        self.lexer.next();
                        if depth >= 0 {
                            result.push_str(&text);
                        }
                    } else {
                        break;
                    }
                }
                result.push(')');
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
        
        Ok(result)
    }

    fn parse_word_list(&mut self) -> Result<Vec<String>, ParserError> {
        let mut words = Vec::new();
        
        while let Some(token) = self.lexer.peek() {
            match token {
                Token::Identifier | Token::Number | Token::DoubleQuotedString |
                Token::SingleQuotedString | Token::Dollar | Token::DollarBrace |
                Token::DollarParen | Token::BraceOpen | Token::BacktickString | Token::Arithmetic | Token::DollarHashSimple | Token::DollarAtSimple | Token::DollarStarSimple
                | Token::DollarBraceHash | Token::DollarBraceBang | Token::DollarBraceStar | Token::DollarBraceAt
                | Token::DollarBraceHashStar | Token::DollarBraceHashAt | Token::DollarBraceBangStar | Token::DollarBraceBangAt => {
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
                    Some(Token::Arithmetic) => { depth += 2; }
                    Some(Token::ParenClose) => {
                        // Could be one or two parens; capture text and adjust after
                    }
                    _ => {}
                }
                let seg = self.lexer.get_text(start, end);
                self.lexer.next();
                // Adjust depth when we see '))'
                if seg == ")" { depth -= 1; }
                text.push_str(&seg);
            } else {
                break;
            }
        }
        Ok(text)
    }

    fn capture_parenthetical_text(&mut self) -> Result<String, ParserError> {
        // Assumes current token is '(' or we are right before it (when called after consuming '<' or '>')
        if !matches!(self.lexer.peek(), Some(Token::ParenOpen)) {
            // If not at '(', just parse a word
            return self.parse_word();
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
        Ok(Command::Simple(SimpleCommand { name: "((".to_string(), args: vec![expr.trim().to_string()], redirects: Vec::new(), env_vars: HashMap::new() }))
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_command() {
        let input = "echo hello world";
        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_pipeline() {
        let input = "ls | grep test";
        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_if_statement() {
        let input = "if true; then echo hello; fi";
        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_ok());
    }
} 