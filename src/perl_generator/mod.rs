use crate::ast::*;
use crate::shared_utils::SharedUtils;
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};

// Static counter for generating unique temp file names
static TEMP_FILE_COUNTER: AtomicUsize = AtomicUsize::new(0);

// Re-export submodules
pub mod commands;
pub mod control_flow;
pub mod words;
pub mod expansions;
pub mod redirects;
pub mod test_expressions;
pub mod utils;

pub struct PerlGenerator {
    indent_level: usize,
    declared_locals: HashSet<String>,
    declared_functions: HashSet<String>,
    subshell_depth: usize,
    file_handle_counter: usize,
    pipeline_counter: usize,
    needs_file_find: bool,
}

impl PerlGenerator {
    pub fn new() -> Self {
        Self {
            indent_level: 0,
            declared_locals: HashSet::new(),
            declared_functions: HashSet::new(),
            subshell_depth: 0,
            file_handle_counter: 0,
            pipeline_counter: 0,
            needs_file_find: false,
        }
    }

    // Add the missing generate method that wasm.rs expects
    pub fn generate(&mut self, commands: &[Command]) -> String {
        let mut output = String::new();
        for command in commands {
            output.push_str(&self.generate_command(command));
        }
        output
    }

    // This method is needed by the generate method above
    pub fn generate_command(&mut self, command: &Command) -> String {
        commands::generate_command_impl(self, command)
    }
}
