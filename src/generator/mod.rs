use crate::ast::*;
use std::collections::HashSet;

pub mod commands;
pub mod control_flow;
pub mod words;
pub mod expansions;
pub mod redirects;
pub mod test_expressions;
pub mod utils;

pub struct Generator {
    pub indent_level: usize,
    pub declared_locals: HashSet<String>,
    pub declared_functions: HashSet<String>,
    pub file_handle_counter: usize,
}

impl Generator {
    pub fn new() -> Self {
        Self {
            indent_level: 0,
            declared_locals: HashSet::new(),
            declared_functions: HashSet::new(),
            file_handle_counter: 0,
        }
    }

    pub fn generate(&mut self, ast: &[Command]) -> String {
        let mut output = String::new();
        
        for command in ast {
            // Reset indentation level for each top-level command to prevent staircase effect
            self.indent_level = 0;
            output.push_str(&self.generate_command(command));
        }
        
        output
    }

    pub fn generate_command(&mut self, command: &Command) -> String {
        commands::generate_command_impl(self, command)
    }

    // Delegate to submodules
    pub fn generate_simple_command(&mut self, cmd: &SimpleCommand) -> String {
        commands::generate_simple_command_impl(self, cmd)
    }

    pub fn generate_pipeline(&mut self, pipeline: &Pipeline) -> String {
        commands::generate_pipeline_impl(self, pipeline)
    }

    pub fn generate_subshell(&mut self, command: &Command) -> String {
        commands::generate_subshell_impl(self, command)
    }

    pub fn generate_background(&mut self, command: &Command) -> String {
        commands::generate_background_impl(self, command)
    }

    pub fn generate_command_string_for_system(&mut self, cmd: &Command) -> String {
        commands::generate_command_string_for_system_impl(self, cmd)
    }

    pub fn generate_if_statement(&mut self, if_stmt: &IfStatement) -> String {
        control_flow::generate_if_statement_impl(self, if_stmt)
    }

    pub fn generate_case_statement(&mut self, case_stmt: &CaseStatement) -> String {
        control_flow::generate_case_statement_impl(self, case_stmt)
    }

    pub fn generate_while_loop(&mut self, while_loop: &WhileLoop) -> String {
        control_flow::generate_while_loop_impl(self, while_loop)
    }

    pub fn generate_for_loop(&mut self, for_loop: &ForLoop) -> String {
        control_flow::generate_for_loop_impl(self, for_loop)
    }

    pub fn generate_function(&mut self, func: &Function) -> String {
        control_flow::generate_function_impl(self, func)
    }

    pub fn generate_block(&mut self, block: &Block) -> String {
        control_flow::generate_block_impl(self, block)
    }

    pub fn generate_break_statement(&mut self, level: &Option<String>) -> String {
        control_flow::generate_break_statement_impl(self, level)
    }

    pub fn generate_continue_statement(&mut self, level: &Option<String>) -> String {
        control_flow::generate_continue_statement_impl(self, level)
    }

    pub fn generate_return_statement(&mut self, value: &Option<Word>) -> String {
        control_flow::generate_return_statement_impl(self, value)
    }

    pub fn generate_test_expression(&mut self, test_expr: &TestExpression) -> String {
        test_expressions::generate_test_expression_impl(self, test_expr)
    }

    pub fn generate_test_command(&mut self, cmd: &SimpleCommand, output: &mut String) {
        test_expressions::generate_test_command_impl(self, cmd, output)
    }

    pub fn generate_shopt_command(&mut self, cmd: &ShoptCommand) -> String {
        redirects::generate_shopt_command_impl(self, cmd)
    }

    pub fn generate_builtin_command(&mut self, cmd: &BuiltinCommand) -> String {
        redirects::generate_builtin_command_impl(self, cmd)
    }

    pub fn generate_redirect(&mut self, redirect: &Redirect) -> String {
        redirects::generate_redirect_impl(self, redirect)
    }

    pub fn word_to_perl(&mut self, word: &Word) -> String {
        words::word_to_perl_impl(self, word)
    }

    pub fn word_to_perl_for_test(&mut self, word: &Word) -> String {
        words::word_to_perl_for_test_impl(self, word)
    }

    pub fn generate_parameter_expansion(&mut self, pe: &ParameterExpansion) -> String {
        expansions::generate_parameter_expansion_impl(self, pe)
    }

    pub fn extract_array_key(&self, var: &str) -> Option<(String, String)> {
        utils::extract_array_key_impl(var)
    }

    pub fn extract_array_elements(&self, value: &str) -> Option<Vec<String>> {
        utils::extract_array_elements_impl(value)
    }

    pub fn perl_string_literal(&mut self, word: &Word) -> String {
        utils::perl_string_literal_impl(self, word)
    }

    pub fn get_unique_file_handle(&mut self) -> String {
        utils::get_unique_file_handle_impl(self)
    }

    // Additional helper methods that are needed
    pub fn handle_range_expansion(&self, s: &str) -> String {
        words::handle_range_expansion_impl(self, s)
    }

    pub fn handle_comma_expansion(&self, s: &str) -> String {
        words::handle_comma_expansion_impl(self, s)
    }

    pub fn handle_brace_expansion(&mut self, expansion: &BraceExpansion) -> String {
        words::handle_brace_expansion_impl(self, expansion)
    }

    pub fn brace_item_to_word(&self, item: &BraceItem) -> Word {
        words::brace_item_to_word_impl(self, item)
    }

    pub fn convert_string_interpolation_to_perl(&self, interp: &StringInterpolation) -> String {
        words::convert_string_interpolation_to_perl_impl(self, interp)
    }

    pub fn convert_arithmetic_to_perl(&self, expr: &str) -> String {
        words::convert_arithmetic_to_perl_impl(self, expr)
    }

    pub fn convert_extglob_to_perl_regex(&self, pattern: &str) -> String {
        test_expressions::convert_extglob_to_perl_regex_impl(self, pattern)
    }

    pub fn convert_glob_to_regex(&self, pattern: &str) -> String {
        test_expressions::convert_glob_to_regex_impl(self, pattern)
    }

    pub fn convert_test_args_to_expression(&self, args: &[Word]) -> TestExpression {
        test_expressions::convert_test_args_to_expression_impl(self, args)
    }

    pub fn indent(&self) -> String {
        control_flow::indent_impl(self)
    }

    pub fn generate_block_commands(&mut self, block: &Block) -> String {
        control_flow::generate_block_commands_impl(self, block)
    }

    pub fn escape_perl_string(&self, s: &str) -> String {
        commands::utilities::escape_perl_string(s)
    }
}
