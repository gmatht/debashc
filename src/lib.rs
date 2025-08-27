pub mod lexer;
pub mod parser;
pub mod ast;
pub mod debug;
pub mod shared_utils;
pub mod generator;
pub mod wasm;
pub mod cmd;

// Only export the main types to avoid conflicts
pub use lexer::{Lexer, Token};
pub use parser::commands::Parser;
pub use parser::utilities::ParserUtilities;
pub use ast::*;
pub use generator::Generator;
