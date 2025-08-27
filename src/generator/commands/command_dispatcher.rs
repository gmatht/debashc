use crate::ast::*;
use crate::generator::Generator;
use super::cat::generate_cat_command;

pub fn generate_command_impl(generator: &mut Generator, command: &Command) -> String {
    match command {
        Command::Simple(cmd) => generator.generate_simple_command(cmd),
        Command::ShoptCommand(cmd) => generator.generate_shopt_command(cmd),
        Command::TestExpression(test_expr) => {
            generator.generate_test_expression(test_expr)
        },
        Command::Pipeline(pipeline) => generator.generate_pipeline(pipeline),
        Command::If(if_stmt) => generator.generate_if_statement(if_stmt),
        Command::Case(case_stmt) => generator.generate_case_statement(case_stmt),
        Command::While(while_loop) => generator.generate_while_loop(while_loop),
        Command::For(for_loop) => generator.generate_for_loop(for_loop),
        Command::Function(func) => generator.generate_function(func),
        Command::Subshell(cmd) => generator.generate_subshell(cmd),
        Command::Background(cmd) => generator.generate_background(cmd),
        Command::Block(block) => generator.generate_block(block),
        Command::BuiltinCommand(cmd) => generator.generate_builtin_command(cmd),
        Command::Break(level) => generator.generate_break_statement(level),
        Command::Continue(level) => generator.generate_continue_statement(level),
        Command::Return(value) => generator.generate_return_statement(value),
        Command::BlankLine => "\n".to_string(),
        Command::Redirect(redirect_cmd) => {
            // Check if this is a cat command with heredocs
            if let Command::Simple(cat_cmd) = &*redirect_cmd.command {
                if let Word::Literal(cmd_name) = &cat_cmd.name {
                    if cmd_name == "cat" {
                        // Check if any of the redirects are heredocs
                        let has_heredoc = redirect_cmd.redirects.iter().any(|r| {
                            matches!(r.operator, RedirectOperator::Heredoc | RedirectOperator::HeredocTabs)
                        });
                        
                        if has_heredoc {
                            // Use the dedicated cat command generator for heredocs
                            return generate_cat_command(generator, cat_cmd, &redirect_cmd.redirects);
                        }
                    }
                }
            }
            
            // Default redirect handling for other commands
            let mut result = generate_command_impl(generator, &redirect_cmd.command);
            for redirect in &redirect_cmd.redirects {
                result.push_str(&generator.generate_redirect(redirect));
            }
            result
        }
    }
}
