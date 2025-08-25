use crate::lexer::LexerError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("Lexer error: {0}")]
    Lexer(#[from] LexerError),
    #[error("Unexpected token: {token:?} at {line}:{col}")]
    UnexpectedToken { token: crate::lexer::Token, line: usize, col: usize },
    #[error("Expected token: {0:?}")]
    _ExpectedToken(crate::lexer::Token), // Unused variant, prefixed with underscore
    #[error("Unexpected end of input")]
    UnexpectedEOF,
    #[error("Invalid syntax: {0}")]
    InvalidSyntax(String),
}

