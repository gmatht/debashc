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
            // Check if we're at EOF after skipping whitespace and comments
            if self.lexer.is_eof() {
                break;
            }
            
            // Check if we're at a newline/semicolon/& (empty command or separator)
            if let Some(token) = self.lexer.peek() {
                match token {
                    Token::Newline | Token::Semicolon | Token::CarriageReturn | Token::Background => {
                        self.lexer.next(); // consume the separator
                        self.skip_whitespace_and_comments();
                        continue;
                    }
                    _ => {}
                }
            }
            
            let command = self.parse_command()?;
            commands.push(command);
            
            // Handle semicolons, newlines, and background '&'
            while let Some(token) = self.lexer.peek() {
                match token {
                    Token::Semicolon | Token::Newline | Token::Background => {
                        self.lexer.next();
                    }
                    _ => break,
                }
            }
            
            // Skip whitespace and comments before next command
            self.skip_whitespace_and_comments();
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
        
        while let Some(token) = self.lexer.peek() {
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
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
        
        // Parse command name
        let name = if let Some(token) = self.lexer.peek() {
            match token {
                Token::Identifier => {
                    self.get_identifier_text()?
                }
                Token::TestBracket => {
                    self.lexer.next(); // consume the [
                    "[".to_string()
                }
                Token::True => {
                    self.lexer.next(); // consume true
                    "true".to_string()
                }
                Token::False => {
                    self.lexer.next(); // consume false
                    "false".to_string()
                }
                Token::Dollar | Token::DollarBrace | Token::DollarParen => {
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
        
        // Parse arguments and redirects
        while let Some(token) = self.lexer.peek() {
            match token {
                Token::Identifier | Token::Number | Token::DoubleQuotedString | Token::SingleQuotedString | Token::SourceDot => {
                    args.push(self.parse_word()?);
                }
                Token::Dollar | Token::DollarBrace | Token::DollarParen => {
                    args.push(self.parse_variable_expansion()?);
                }
                Token::Minus => {
                    // Handle arguments starting with minus (like -la, -v, etc.)
                    let token_clone = token.clone();
                    self.lexer.next(); // consume the minus
                    if let Some(Token::Identifier) = self.lexer.peek() {
                        let arg = format!("-{}", self.get_identifier_text()?);
                        args.push(arg);
                    } else {
                        return Err(ParserError::UnexpectedToken { token: token_clone, line: 1, col: 1 });
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
                // Test comparison operators
                Token::Eq => { self.lexer.next(); args.push("-eq".to_string()); }
                Token::Ne => { self.lexer.next(); args.push("-ne".to_string()); }
                Token::Lt => { self.lexer.next(); args.push("-lt".to_string()); }
                Token::Le => { self.lexer.next(); args.push("-le".to_string()); }
                Token::Gt => { self.lexer.next(); args.push("-gt".to_string()); }
                Token::Ge => { self.lexer.next(); args.push("-ge".to_string()); }
                Token::Zero => { self.lexer.next(); args.push("-z".to_string()); }
                Token::RedirectIn | Token::RedirectOut | Token::RedirectAppend |
                Token::RedirectInOut | Token::Heredoc | Token::HeredocTabs => {
                    redirects.push(self.parse_redirect()?);
                }
                Token::Newline | Token::Semicolon => {
                    // Stop parsing arguments when we hit a command separator
                    break;
                }
                Token::TestBracketClose => {
                    // Handle closing bracket for test commands
                    self.lexer.next(); // consume the ]
                    args.push("]".to_string());
                    break;
                }
                Token::Space | Token::Tab => {
                    // Skip whitespace but continue parsing arguments
                    self.lexer.next();
                }
                _ => break,
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
                    let _ = self.parse_command()?;
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
                    let _ = self.parse_command()?;
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
                    _ => { let _ = self.parse_command()?; }
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
        
        // Parse first command inside subshell
        let command = Box::new(self.parse_command()?);

        // Allow multiple commands inside subshell; parse and discard until ')'
        loop {
            // Skip separators within subshell body
            while matches!(
                self.lexer.peek(),
                Some(Token::Space | Token::Tab | Token::Comment | Token::Newline | Token::Semicolon | Token::CarriageReturn | Token::Background)
            ) {
                self.lexer.next();
            }
            match self.lexer.peek() {
                Some(Token::ParenClose) | None => break,
                _ => {
                    // Parse and ignore additional commands inside the subshell
                    let _ = self.parse_command()?;
                }
            }
        }

        self.lexer.consume(Token::ParenClose)?;
        
        Ok(Command::Subshell(command))
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
        
        let target = self.parse_word()?;
        
        Ok(Redirect {
            fd,
            operator,
            target,
        })
    }

    fn parse_word(&mut self) -> Result<String, ParserError> {
        let result = match self.lexer.peek() {
            Some(Token::Identifier) => Ok(self.get_identifier_text()?),
            Some(Token::Number) => Ok(self.get_number_text()?),
            Some(Token::DoubleQuotedString) => Ok(self.get_string_text()?),
            Some(Token::SingleQuotedString) => Ok(self.get_string_text()?),
            Some(Token::BacktickString) => Ok(self.get_raw_token_text()?),
            Some(Token::BraceOpen) => Ok(self.parse_brace_word()?),
            Some(Token::SourceDot) => {
                // Treat standalone '.' as a normal word (e.g., `find . -name ...`)
                self.lexer.next();
                Ok(".".to_string())
            }
            Some(Token::Dollar) => Ok(self.parse_variable_expansion()?),
            Some(Token::DollarBrace) => Ok(self.parse_variable_expansion()?),
            Some(Token::DollarParen) => Ok(self.parse_variable_expansion()?),
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
            Some(Token::DollarBrace) => {
                self.lexer.next();
                if let Some(Token::Identifier) = self.lexer.peek() {
                    result.push_str(&format!("${{{}}}", self.get_identifier_text()?));
                    self.lexer.consume(Token::BraceClose)?;
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
                Token::DollarParen | Token::BraceOpen | Token::BacktickString => {
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
        let mut last_end = start;

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
                last_end = end;

                if depth == 0 {
                    break;
                }
            } else {
                return Err(ParserError::UnexpectedEOF);
            }
        }

        // Return substring including the braces
        Ok(self.lexer.get_text(start, last_end))
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