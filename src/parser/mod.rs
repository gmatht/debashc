use crate::ast::*;
use crate::lexer::{Lexer, Token, LexerError};
use thiserror::Error;
use std::collections::HashMap;

// Re-export all parser modules
pub mod commands;
pub mod control_flow;
pub mod words;
pub mod redirects;
pub mod assignments;
pub mod utilities;
pub mod errors;

// Re-export the main Parser struct and error types
pub use errors::ParserError;
pub use commands::Parser;

// Re-export the main parsing function
pub use commands::parse;

