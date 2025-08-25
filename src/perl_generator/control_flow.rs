use crate::ast::*;
use super::Generator;

pub fn generate_if_statement_impl(generator: &mut Generator, if_stmt: &IfStatement) -> String {
    let mut output = String::new();
    
    // Generate condition
    output.push_str("if (");
    match &*if_stmt.condition {
        Command::Simple(cmd) if cmd.name == "[" || cmd.name == "test" => {
            generator.generate_test_command(cmd, &mut output);
        }
        _ => {
            output.push_str(&generator.generate_command(&if_stmt.condition));
        }
    }
    output.push_str(") {\n");
    
    // Generate then branch
    generator.indent_level += 1;
    output.push_str(&generator.indent());
    output.push_str(&generator.generate_command(&if_stmt.then_branch));
    generator.indent_level -= 1;
    
    // Generate else branch if present
    if let Some(else_branch) = &if_stmt.else_branch {
        output.push_str(&generator.indent());
        output.push_str("} else {\n");
        generator.indent_level += 1;
        output.push_str(&generator.indent());
        output.push_str(&generator.generate_command(else_branch));
        generator.indent_level -= 1;
    }
    
    output.push_str(&generator.indent());
    output.push_str("}\n");
    
    output
}

pub fn generate_case_statement_impl(generator: &mut Generator, case_stmt: &CaseStatement) -> String {
    let mut output = String::new();
    
    // Convert bash case statement to Perl if/elsif/else
    let mut first_case = true;
    
    for case_clause in &case_stmt.cases {
        if first_case {
            // First case becomes 'if'
            output.push_str("if (");
            first_case = false;
        } else {
            // Subsequent cases become 'elsif'
            output.push_str(&generator.indent());
            output.push_str("} elsif (");
        }
        
        // Handle multiple patterns in a single case clause
        let mut pattern_conditions = Vec::new();
        for pattern in &case_clause.patterns {
            let pattern_str = generator.perl_string_literal(pattern);
            if pattern_str == "\"*\"" {
                // Default case - this should be the last one
                pattern_conditions.push("1".to_string()); // Always true
            } else {
                // Convert bash glob patterns to Perl regex
                let mut perl_pattern = pattern_str.trim_matches('"').to_string();
                perl_pattern = perl_pattern.replace("*", ".*");
                perl_pattern = perl_pattern.replace("?", ".");
                perl_pattern = perl_pattern.replace("[", "\\[");
                perl_pattern = perl_pattern.replace("]", "\\]");
                
                // Create condition: $operation =~ /^pattern$/
                let word_str = generator.word_to_perl(&case_stmt.word);
                
                // Handle positional parameters in case statements
                let processed_word = if word_str.contains("$1") || word_str.contains("$2") || word_str.contains("$3") {
                    // Replace positional parameters with generic names that will be replaced later
                    word_str.replace("$1", "$arg1").replace("$2", "$arg2").replace("$3", "$arg3")
                } else if word_str.contains("$name") {
                    // The word_to_perl converted $1 to $name, but we need $arg1 for parameter replacement
                    word_str.replace("$name", "$arg1")
                } else {
                    word_str
                };
                
                pattern_conditions.push(format!("{} =~ /^{}$/", processed_word, perl_pattern));
            }
        }
        
        // Join multiple patterns with 'or'
        output.push_str(&pattern_conditions.join(" or "));
        output.push_str(") {\n");
        
        generator.indent_level += 1;
        // Generate body commands
        for command in &case_clause.body {
            output.push_str(&generator.indent());
            output.push_str(&generator.generate_command(command));
        }
        generator.indent_level -= 1;
    }
    
    // Close the if/elsif chain
    output.push_str(&generator.indent());
    output.push_str("}\n");
    
    output
}

pub fn generate_while_loop_impl(generator: &mut Generator, while_loop: &WhileLoop) -> String {
    let mut output = String::new();
    
    // Generate while loop
    output.push_str("while (");
    match &*while_loop.condition {
        Command::Simple(cmd) if cmd.name == "[" || cmd.name == "test" => {
            generator.generate_test_command(cmd, &mut output);
        }
        _ => {
            output.push_str(&generator.generate_command(&while_loop.condition));
        }
    }
    output.push_str(") {\n");
    
    // Generate body
    generator.indent_level += 1;
    output.push_str(&generator.indent());
    output.push_str(&generator.generate_block_commands(&while_loop.body));
    generator.indent_level -= 1;
    
    output.push_str(&generator.indent());
    output.push_str("}\n");
    
    output
}

pub fn generate_for_loop_impl(generator: &mut Generator, for_loop: &ForLoop) -> String {
    let mut output = String::new();
    
    // Generate for loop
    output.push_str("for my $item (");
    
    // Handle different types of for loop items
    let items: Vec<String> = for_loop.items.iter()
        .map(|word| generator.word_to_perl(word))
        .collect();
    output.push_str(&items.join(", "));
    
    output.push_str(") {\n");
    
    // Generate body
    generator.indent_level += 1;
    output.push_str(&generator.indent());
    output.push_str(&generator.generate_block_commands(&for_loop.body));
    generator.indent_level -= 1;
    
    output.push_str(&generator.indent());
    output.push_str("}\n");
    
    output
}

pub fn generate_function_impl(generator: &mut Generator, func: &Function) -> String {
    let mut output = String::new();
    
    // Generate function definition
    output.push_str(&format!("sub {} {{\n", func.name));
    
    // Handle function parameters
    if !func.parameters.is_empty() {
        generator.indent_level += 1;
        output.push_str(&generator.indent());
        output.push_str("my (");
        let params: Vec<String> = func.parameters.iter()
            .map(|param| format!("${}", param))
            .collect();
        output.push_str(&params.join(", "));
        output.push_str(") = @_;\n");
        
        // Generate function body
        output.push_str(&generator.indent());
        output.push_str(&generator.generate_block_commands(&func.body));
        
        generator.indent_level -= 1;
    } else {
        // No parameters
        generator.indent_level += 1;
        output.push_str(&generator.indent());
        output.push_str(&generator.generate_block_commands(&func.body));
        generator.indent_level -= 1;
    }
    
    output.push_str("}\n");
    
    // Mark function as declared
    generator.declared_functions.insert(func.name.clone());
    
    output
}

pub fn generate_block_impl(generator: &mut Generator, block: &Block) -> String {
    let mut output = String::new();
    
    // Generate block
    output.push_str("{\n");
    
    generator.indent_level += 1;
    output.push_str(&generator.indent());
    output.push_str(&generator.generate_block_commands(block));
    generator.indent_level -= 1;
    
    output.push_str(&generator.indent());
    output.push_str("}\n");
    
    output
}

pub fn generate_break_statement_impl(generator: &Generator, level: &Option<String>) -> String {
    match level {
        Some(level_str) => format!("last LABEL{};", level_str),
        None => "last;".to_string(),
    }
}

pub fn generate_continue_statement_impl(generator: &Generator, level: &Option<String>) -> String {
    match level {
        Some(level_str) => format!("next LABEL{};", level_str),
        None => "next;".to_string(),
    }
}

pub fn generate_return_statement_impl(generator: &mut Generator, value: &Option<Word>) -> String {
    match value {
        Some(word) => {
            let perl_value = generator.perl_string_literal(word);
            format!("return {};", perl_value)
        }
        None => "return;".to_string(),
    }
}

// Helper method for indentation
pub fn indent_impl(generator: &Generator) -> String {
    "    ".repeat(generator.indent_level)
}

pub fn generate_block_commands_impl(generator: &mut Generator, block: &Block) -> String {
    let mut output = String::new();
    for command in &block.commands {
        output.push_str(&generator.indent());
        output.push_str(&generator.generate_command(command));
        if !output.ends_with('\n') {
            output.push('\n');
        }
    }
    output
}
