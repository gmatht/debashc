use crate::ast::*;
use std::collections::HashSet;
// HashMap import removed as it's not used

// Use the debug macros from the root of the crate
use crate::debug_eprintln;

pub struct PerlGenerator {
    indent_level: usize,
    declared_locals: HashSet<String>,
    declared_functions: HashSet<String>,
    subshell_depth: usize,
    file_handle_counter: usize,
    pipeline_counter: usize,
    used_modules: HashSet<String>,
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
            used_modules: HashSet::new(),
            needs_file_find: false,
        }
    }

    fn get_unique_file_handle(&mut self) -> String {
        self.file_handle_counter += 1;
        format!("$fh_{}", self.file_handle_counter)
    }

    fn get_unique_dir_handle(&mut self) -> String {
        self.file_handle_counter += 1;
        format!("$dh_{}", self.file_handle_counter)
    }

    pub fn generate(&mut self, commands: &[Command]) -> String {
        // First pass: scan all commands to determine if File::Find is needed
        self.scan_for_file_find_usage(commands);
        
        let mut output = String::new();
        output.push_str("#!/usr/bin/env perl\n");
        output.push_str("use strict;\n");
        output.push_str("use warnings;\n");
        output.push_str("use File::Basename;\n");
        
        // Add File::Find if needed
        debug_eprintln!("DEBUG: needs_file_find = {}", self.needs_file_find);
        if self.needs_file_find {
            output.push_str("use File::Find;\n");
        }
        
        output.push_str("\n");

        for command in commands {
            output.push_str(&self.generate_command(command));
        }
        // Remove all trailing newlines
        while output.ends_with('\n') { output.pop(); }
        output
    }

    fn scan_for_file_find_usage(&mut self, commands: &[Command]) {
        for command in commands {
            match command {
                Command::Pipeline(pipeline) => {
                    for cmd in &pipeline.commands {
                        if let Command::Simple(simple_cmd) = cmd {
                            if simple_cmd.name == "find" {
                                self.needs_file_find = true;
                                return; // Early return once we find a find command
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn generate_command(&mut self, command: &Command) -> String {
        debug_eprintln!("DEBUG: generate_command called with: {:?}", command);
        match command {
            Command::Simple(cmd) => self.generate_simple_command(cmd),
            Command::ShoptCommand(cmd) => self.generate_shopt_command(cmd),
            Command::TestExpression(test_expr) => {
                debug_eprintln!("DEBUG: Routing to generate_test_expression");
                self.generate_test_expression(test_expr)
            },
            Command::Pipeline(pipeline) => self.generate_pipeline(pipeline),
            Command::If(if_stmt) => self.generate_if_statement(if_stmt),
            Command::While(while_loop) => self.generate_while_loop(while_loop),
            Command::For(for_loop) => self.generate_for_loop(for_loop),
            Command::Function(func) => self.generate_function(func),
            Command::Subshell(cmd) => self.generate_subshell(cmd),
            Command::Background(cmd) => self.generate_background(cmd),
            Command::Block(block) => self.generate_block(block),
            Command::BuiltinCommand(cmd) => self.generate_builtin_command(cmd),
            Command::BlankLine => "\n".to_string(),
        }
    }

    fn generate_simple_command(&mut self, cmd: &SimpleCommand) -> String {
        let mut output = String::new();
        let has_env = !cmd.env_vars.is_empty() && cmd.name != "true";
        if has_env {
            output.push_str("{\n");
            for (var, value) in &cmd.env_vars {
                let val = self.perl_string_literal(value);
                output.push_str(&format!("local $ENV{{{}}} = {};;\n", var, val));
            }
        }

        // Generate the command
        if cmd.name == "((" {
            // Handle arithmetic expressions like ((i++))
            if let Some(expr) = cmd.args.first() {
                // Convert shell arithmetic to Perl
                let perl_expr = self.convert_arithmetic_to_perl(expr);
                output.push_str(&format!("{}\\n", perl_expr));
            }
        } else if cmd.name == "true" && !cmd.env_vars.is_empty() && cmd.args.is_empty() {
            // Assignment-only shell locals: e.g., a=1
            for (var, value) in &cmd.env_vars {
                if let Word::Array(_, elements) = value {
                    // Handle array assignment: arr=(one two three) -> @arr = ("one", "two", "three")
                    let elements_perl: Vec<String> = elements.iter()
                        .map(|s| self.perl_string_literal(s))
                        .collect();
                    if self.subshell_depth > 0 || !self.declared_locals.contains(var) {
                        output.push_str(&format!("my @{} = ({});\n", var, elements_perl.join(", ")));
                    } else {
                        output.push_str(&format!("@{} = ({});\n", var, elements_perl.join(", ")));
                    }
                    if self.subshell_depth == 0 {
                        self.declared_locals.insert(var.clone());
                    }
                    continue; // Skip the regular assignment below
                } else if let Word::Literal(literal) = value {
                    if literal.starts_with("(") && literal.ends_with(")") {
                        // Handle array assignment: arr=(one two three) -> @arr = ("one", "two", "three")
                        let content = &literal[1..literal.len()-1];
                        let elements: Vec<String> = content.split_whitespace()
                            .map(|s| self.perl_string_literal(s))
                            .collect();
                        if self.subshell_depth > 0 || !self.declared_locals.contains(var) {
                            output.push_str(&format!("my @{} = ({});\n", var, elements.join(", ")));
                        } else {
                            output.push_str(&format!("@{} = ({});\n", var, elements.join(", ")));
                        }
                        if self.subshell_depth == 0 {
                            self.declared_locals.insert(var.clone());
                        }
                        continue; // Skip the regular assignment below
                    }
                }
                
                // Check if this is an array assignment like map[foo]=bar
                if var.contains('[') && var.ends_with(']') {
                    if let Some(bracket_start) = var.find('[') {
                        let array_name = &var[..bracket_start];
                        let key = &var[bracket_start + 1..var.len() - 1];
                        let val = match value {
                            Word::Literal(literal) => self.perl_string_literal(literal),
                            _ => self.word_to_perl(value),
                        };
                        if self.subshell_depth > 0 || !self.declared_locals.contains(array_name) {
                            output.push_str(&format!("my %{} = ();\n", array_name));
                            self.declared_locals.insert(array_name.to_string());
                        }
                        output.push_str(&format!("${}{{{}}} = {};\n", array_name, key, val));
                        continue; // Skip the regular assignment below
                    }
                }
                
                let val = match value {
                    Word::Arithmetic(arithmetic) => {
                        // Handle shell arithmetic: $((i + 1)) -> $i + 1
                        self.convert_arithmetic_to_perl(&arithmetic.expression)
                    }
                    Word::Literal(literal) => {
                        if literal.starts_with("$(") && literal.ends_with(")") {
                            // Handle command substitution: $(command) -> `command`
                            let cmd = &literal[2..literal.len()-1];
                            format!("`{}`", cmd)
                        } else {
                            self.perl_string_literal(literal)
                        }
                    }
                    Word::Variable(var_name) => {
                        // Handle variable references
                        format!("${}", var_name)
                    }
                    _ => {
                        // Handle other Word types by converting to string
                        self.word_to_perl(value)
                    }
                };
                
                if self.subshell_depth > 0 || !self.declared_locals.contains(var) {
                    output.push_str(&format!("my ${} = {};\n", var, val));
                } else {
                    output.push_str(&format!("${} = {};\n", var, val));
                }
                if self.subshell_depth == 0 {
                    self.declared_locals.insert(var.clone());
                }
            }
        } else if cmd.name == "true" {
            // Builtin true: successful no-op
            output.push_str("1;\n");
        } else if cmd.name == "false" {
            // Builtin false: no-op; semantic failure not modeled in this simplified generator
            output.push_str("0;\n");
        } else if cmd.name == "printf" {
            // Handle printf command
            if cmd.args.is_empty() {
                output.push_str("printf(\"\\n\");\n");
            } else {
                let format_str = &cmd.args[0];
                let args = &cmd.args[1..];
                if args.is_empty() {
                    output.push_str(&format!("printf({});\n", self.perl_string_literal(format_str)));
                } else {
                    // For printf, the format string should be properly quoted
                    let format_str_perl = match format_str {
                        Word::StringInterpolation(interp) => {
                            // Convert string interpolation and wrap in quotes for printf
                            // For printf format strings, preserve array length expressions
                            let content = self.convert_string_interpolation_to_perl_for_printf(interp);
                            format!("\"{}\"", content)
                        }
                        _ => self.perl_string_literal(format_str)
                    };
                    let perl_args = args.iter().map(|arg| {
                        // For printf arguments, we need to ensure proper quoting
                        match arg {
                            Word::Variable(var) => format!("${}", var),
                            Word::StringInterpolation(interp) => {
                                // For printf arguments, if it's just a single literal part, quote it
                                if interp.parts.len() == 1 {
                                    if let StringPart::Literal(s) = &interp.parts[0] {
                                        return format!("\"{}\"", self.escape_perl_string(s));
                                    }
                                }
                                // For more complex interpolations, wrap the result in quotes
                                let content = self.convert_string_interpolation_to_perl_for_printf(interp);
                                format!("\"{}\"", content)
                            }
                            _ => self.word_to_perl(arg)
                        }
                    }).collect::<Vec<_>>();
                    output.push_str(&format!("printf({}, {});\n", 
                        format_str_perl, 
                        perl_args.join(", ")));
                }
            }
        } else if cmd.name == "echo" {
            // Special handling for echo
            if cmd.args.is_empty() {
                output.push_str("print(\"\\n\");\n");
            } else if cmd.args.len() == 1 {
                // Handle single argument
                let arg = &cmd.args[0];
                if matches!(arg, Word::Variable(var) if var == "#") {
                    output.push_str("print(scalar(@ARGV) . \"\\n\");\n");
                } else if matches!(arg, Word::Variable(var) if var == "@") {
                    output.push_str("print(join(\" \", @ARGV) . \"\\n\");\n");
                } else if let Word::Variable(var) = arg {
                    // Handle special array and hash operations
                    if var.starts_with('#') && var.ends_with("[@]") {
                        // This is #arr[@] - convert to scalar(@arr) and concatenate with newline
                        let array_name = &var[1..var.len()-3]; // Remove # prefix and [@] suffix
                        output.push_str(&format!("print(scalar(@{}) . \"\\n\");\n", array_name));
                    } else if var.starts_with('#') && var.ends_with("[*]") {
                        // This is #arr[*] - convert to scalar(@arr) and concatenate with newline
                        let array_name = &var[1..var.len()-3]; // Remove # prefix and [*] suffix
                        output.push_str(&format!("print(scalar(@{}) . \"\\n\");\n", array_name));
                    } else if var.starts_with('!') && var.ends_with("[@]") {
                        // This is !map[@] - convert to keys(%map) and concatenate with newline
                        let array_name = &var[1..var.len()-3]; // Remove ! prefix and [@] suffix
                        output.push_str(&format!("print(join(\" \", keys(%{})) . \"\\n\");\n", array_name));
                    } else if var.starts_with('!') && var.ends_with("[*]") {
                        // This is !map[*] - convert to keys(%map) and concatenate with newline
                        let array_name = &var[1..var.len()-3]; // Remove ! prefix and [*] suffix
                        output.push_str(&format!("print(join(\" \", keys(%{})) . \"\\n\");\n", array_name));
                    } else {
                        // Handle other variables
                        output.push_str(&format!("print(\"${}\\n\");\n", var));
                    }
                } else if let Word::BraceExpansion(expansion) = arg {
                    // Handle brace expansion
                    if expansion.items.len() == 1 {
                        match &expansion.items[0] {
                            BraceItem::Range(range) => {
                                // Check if this is a character range first
                                if let (Some(start_char), Some(end_char)) = (range.start.chars().next(), range.end.chars().next()) {
                                    if start_char.is_ascii_lowercase() && end_char.is_ascii_lowercase() {
                                        // This is a character range
                                        let start = start_char as u8;
                                        let end = end_char as u8;
                                        if start <= end {
                                            let step = range.step.as_ref().and_then(|s| s.parse::<usize>().ok()).unwrap_or(1);
                                            let values: Vec<String> = (start..=end)
                                                .step_by(step)
                                                .map(|c| char::from(c).to_string())
                                                .collect();
                                            output.push_str(&format!("print(\"{}\\n\");\n", values.join(" ")));
                                        } else {
                                            // Reverse range
                                            let step = range.step.as_ref().and_then(|s| s.parse::<usize>().ok()).unwrap_or(1);
                                            let values: Vec<String> = (end..=start)
                                                .rev()
                                                .step_by(step)
                                                .map(|c| char::from(c).to_string())
                                                .collect();
                                            output.push_str(&format!("print(\"{}\\n\");\n", values.join(" ")));
                                        }
                                    } else {
                                        // Try numeric range
                                        if let (Ok(start), Ok(end)) = (range.start.parse::<i64>(), range.end.parse::<i64>()) {
                                            let step = range.step.as_ref().and_then(|s| s.parse::<i64>().ok()).unwrap_or(1);
                                            let values: Vec<String> = if step > 0 {
                                                (start..=end).step_by(step as usize).map(|i| {
                                                    // Preserve leading zeros by formatting with the same width as the original
                                                    if range.start.starts_with('0') && range.start.len() > 1 {
                                                        format!("{:0width$}", i, width = range.start.len())
                                                    } else {
                                                        i.to_string()
                                                    }
                                                }).collect()
                                            } else {
                                                (end..=start).rev().step_by((-step) as usize).map(|i| {
                                                    if range.start.starts_with('0') && range.start.len() > 1 {
                                                        format!("{:0width$}", i, width = range.start.len())
                                                    } else {
                                                        i.to_string()
                                                    }
                                                }).collect()
                                            };
                                            output.push_str(&format!("print(\"{}\\n\");\n", values.join(" ")));
                                        } else {
                                            // Fallback for non-numeric ranges
                                            output.push_str(&format!("print(\"{{{}}}..{{{}}}\\n\");\n", range.start, range.end));
                                        }
                                    }
                                } else {
                                    // Try numeric range
                                    if let (Ok(start), Ok(end)) = (range.start.parse::<i64>(), range.end.parse::<i64>()) {
                                        let step = range.step.as_ref().and_then(|s| s.parse::<i64>().ok()).unwrap_or(1);
                                        let values: Vec<String> = if step > 0 {
                                            (start..=end).step_by(step as usize).map(|i| {
                                                // Preserve leading zeros by formatting with the same width as the original
                                                if range.start.starts_with('0') && range.start.len() > 1 {
                                                    format!("{:0width$}", i, width = range.start.len())
                                                } else {
                                                    i.to_string()
                                                }
                                            }).collect()
                                        } else {
                                            (end..=start).rev().step_by((-step) as usize).map(|i| {
                                                if range.start.starts_with('0') && range.start.len() > 1 {
                                                    format!("{:0width$}", i, width = range.start.len())
                                                } else {
                                                    i.to_string()
                                                }
                                            }).collect()
                                        };
                                        output.push_str(&format!("print(\"{}\\n\");\n", values.join(" ")));
                                    } else {
                                        // Fallback for non-numeric ranges
                                        output.push_str(&format!("print(\"{{{}}}..{{{}}}\\n\");\n", range.start, range.end));
                                    }
                                }
                            }
                            BraceItem::Literal(s) => {
                                // Single literal item
                                if s.contains("..") {
                                    // Handle ranges like "a..c" or "00..04..2"
                                    let parts: Vec<&str> = s.split("..").collect();
                                    if parts.len() == 2 {
                                        if let (Some(start_char), Some(end_char)) = (parts[0].chars().next(), parts[1].chars().next()) {
                                            if start_char.is_ascii_lowercase() && end_char.is_ascii_lowercase() {
                                                let start = start_char as u8;
                                                let end = end_char as u8;
                                                if start <= end {
                                                    let values: Vec<String> = (start..=end)
                                                        .map(|c| char::from(c).to_string())
                                                        .collect();
                                                    output.push_str(&format!("print(\"{}\\n\");\n", values.join(" ")));
                                                } else {
                                                    output.push_str(&format!("print(\"{{{}}}\\n\");\n", s));
                                                }
                                            } else {
                                                output.push_str(&format!("print(\"{{{}}}\\n\");\n", s));
                                            }
                                        } else {
                                            output.push_str(&format!("print(\"{{{}}}\\n\");\n", s));
                                        }
                                    } else if parts.len() == 3 {
                                        // Range with step like "00..04..2"
                                        if let (Ok(start), Ok(end), Ok(step)) = (parts[0].parse::<i64>(), parts[1].parse::<i64>(), parts[2].parse::<i64>()) {
                                            let values: Vec<String> = (start..=end).step_by(step as usize).map(|i| i.to_string()).collect();
                                            output.push_str(&format!("print(\"{}\\n\");\n", values.join(" ")));
                                        } else {
                                            output.push_str(&format!("print(\"{{{}}}\\n\");\n", s));
                                        }
                                    } else {
                                        output.push_str(&format!("print(\"{{{}}}\\n\");\n", s));
                                    }
                                } else {
                                    output.push_str(&format!("print(\"{{{}}}\\n\");\n", s));
                                }
                            }
                            BraceItem::Sequence(seq) => {
                                // Convert {a,b,c} to a b c
                                output.push_str(&format!("print(\"{}\\n\");\n", seq.join(" ")));
                            }
                        }
                    } else {
                        // Multiple items - expand each one
                        let mut expanded_items = Vec::new();
                        for item in &expansion.items {
                            match item {
                                BraceItem::Literal(s) => expanded_items.push(s.clone()),
                                BraceItem::Range(range) => {
                                    if let (Ok(start), Ok(end)) = (range.start.parse::<i64>(), range.end.parse::<i64>()) {
                                        let step = range.step.as_ref().and_then(|s| s.parse::<i64>().ok()).unwrap_or(1);
                                        let values: Vec<String> = if step > 0 {
                                            (start..=end).step_by(step as usize).map(|i| i.to_string()).collect()
                                        } else {
                                            (end..=start).rev().step_by((-step) as usize).map(|i| i.to_string()).collect()
                                        };
                                        expanded_items.extend(values);
                                    } else {
                                        expanded_items.push(format!("{{{}}}..{{{}}}", range.start, range.end));
                                    }
                                }
                                BraceItem::Sequence(seq) => {
                                    expanded_items.extend(seq.iter().cloned());
                                }
                            }
                        }
                        // For multiple brace expansions like {a,b,c}{1,2,3}, we need to generate all combinations
                        if expansion.items.len() == 2 {
                            let mut combinations = Vec::new();
                            for item1 in &expanded_items {
                                for item2 in &expanded_items {
                                    combinations.push(format!("{}{}", item1, item2));
                                }
                            }
                            output.push_str(&format!("print(\"{}\\n\");\n", combinations.join(" ")));
                        } else {
                            output.push_str(&format!("print(\"{}\\n\");\n", expanded_items.join(" ")));
                        }
                    }
                } else if let Word::StringInterpolation(interp) = arg {
                    // Handle string interpolation like "$#"
                    if interp.parts.len() == 1 {
                        if let StringPart::Variable(var) = &interp.parts[0] {
                            if var == "#" {
                                output.push_str("print(scalar(@ARGV) . \"\\n\");\n");
                            } else if var == "@" {
                                output.push_str("print(join(\" \", @ARGV) . \"\\n\");\n");
                            } else if var.starts_with('#') && var.ends_with("[@]") {
                                // This is #arr[@] - convert to scalar(@arr) and print
                                let array_name = &var[1..var.len()-3]; // Remove # prefix and [@] suffix
                                output.push_str(&format!("print(scalar(@{}) . \"\\n\");\n", array_name));
                            } else if var.starts_with('#') && var.ends_with("[*]") {
                                // This is #arr[*] - convert to scalar(@arr) and print
                                let array_name = &var[1..var.len()-3]; // Remove # prefix and [*] suffix
                                output.push_str(&format!("print(scalar(@{}) . \"\\n\");\n", array_name));
                            } else if var.starts_with('!') && var.ends_with("[@]") {
                                // This is !map[@] - convert to keys(%map) and print
                                let array_name = &var[1..var.len()-3]; // Remove ! prefix and [@] suffix
                                output.push_str(&format!("print(join(\" \", keys(%{})) . \"\\n\");\n", array_name));
                            } else if var.starts_with('!') && var.ends_with("[*]") {
                                // This is !map[*] - convert to keys(%map) and print
                                let array_name = &var[1..var.len()-3]; // Remove ! prefix and [*] suffix
                                output.push_str(&format!("print(join(\" \", keys(%{})) . \"\\n\");\n", array_name));
                            } else {
                                output.push_str(&format!("print(\"${}\\n\");\n", var));
                            }
                        } else if let StringPart::MapLength(map_name) = &interp.parts[0] {
                            // This is ${#arr[@]} - convert to scalar(@arr) and print
                            output.push_str(&format!("print(scalar(@{}) . \"\\n\");\n", map_name));
                        } else if let StringPart::MapKeys(map_name) = &interp.parts[0] {
                            // This is ${!map[@]} - convert to keys(%map) and print
                            output.push_str(&format!("print(join(\" \", keys(%{})) . \"\\n\");\n", map_name));
                        } else if let StringPart::ParameterExpansion(pe) = &interp.parts[0] {
                            // This is a single parameter expansion - generate without quotes
                            let content = self.generate_parameter_expansion(pe);
                            output.push_str(&format!("print({} . \"\\n\");\n", content));
                        } else {
                            let content = self.convert_string_interpolation_to_perl(interp);
                            output.push_str(&format!("print(\"{}\\n\");\n", content));
                        }
                    } else {
                        let content = self.convert_string_interpolation_to_perl(interp);
                        output.push_str(&format!("print(\"{}\\n\");\n", content));
                    }
                } else {
                    // Handle other argument types
                    let content = self.word_to_perl(arg);
                    output.push_str(&format!("print(\"{}\\n\");\n", content));
                }
            } else {
                // Multiple arguments - use the new brace expansion combiner
                let expanded_args = self.combine_adjacent_brace_expansions(&cmd.args);
                output.push_str(&format!("print(\"{}\\n\");\n", expanded_args.join(" ")));
            }
        } else if cmd.name == "touch" {
            // Special handling for touch with brace expansion support
            if !cmd.args.is_empty() {
                // For touch, we need to reconstruct the full filename pattern and expand brace expansion
                let mut all_files = Vec::new();
                
                // Check if we have a pattern like "file_" + brace_expansion + ".txt"
                if cmd.args.len() >= 3 {
                    // Look for brace expansion in the middle
                    for i in 1..cmd.args.len()-1 {
                        if let Word::BraceExpansion(expansion) = &cmd.args[i] {
                            // Reconstruct the pattern: prefix + brace_expansion + suffix
                            let prefix = self.word_to_perl(&cmd.args[i-1]);
                            // Concatenate all remaining arguments after the brace expansion
                            let suffix: String = cmd.args.iter().skip(i+1).map(|arg| self.word_to_perl(arg)).collect();
                            
                            if expansion.items.len() == 1 {
                                match &expansion.items[0] {
                                    BraceItem::Range(range) => {
                                        if let (Ok(start), Ok(end)) = (range.start.parse::<i64>(), range.end.parse::<i64>()) {
                                            let step = range.step.as_ref().and_then(|s| s.parse::<i64>().ok()).unwrap_or(1);
                                            let values: Vec<String> = if step > 0 {
                                                (start..=end).step_by(step as usize).map(|i| {
                                                    // Preserve leading zeros by formatting with the same width as the original
                                                    if range.start.starts_with('0') && range.start.len() > 1 {
                                                        format!("{:0width$}", i, width = range.start.len())
                                                    } else {
                                                        i.to_string()
                                                    }
                                                }).collect()
                                            } else {
                                                (end..=start).rev().step_by((-step) as usize).map(|i| {
                                                    if range.start.starts_with('0') && range.start.len() > 1 {
                                                        format!("{:0width$}", i, width = range.start.len())
                                                    } else {
                                                        i.to_string()
                                                    }
                                                }).collect()
                                            };
                                            for value in values {
                                                all_files.push(format!("{}{}{}", prefix, value, suffix));
                                            }
                                        }
                                    }
                                    _ => {
                                        // For other brace items, just add the literal
                                        all_files.push(format!("{}{}{}", prefix, self.word_to_perl(&cmd.args[i]), suffix));
                                    }
                                }
                            } else {
                                // Multiple items - expand each one
                                for item in &expansion.items {
                                    match item {
                                        BraceItem::Literal(s) => all_files.push(format!("{}{}{}", prefix, s, suffix)),
                                        BraceItem::Range(range) => {
                                            if let (Ok(start), Ok(end)) = (range.start.parse::<i64>(), range.end.parse::<i64>()) {
                                                let step = range.step.as_ref().and_then(|s| s.parse::<i64>().ok()).unwrap_or(1);
                                                let values: Vec<String> = if step > 0 {
                                                    (start..=end).step_by(step as usize).map(|i| {
                                                        // Preserve leading zeros by formatting with the same width as the original
                                                        if range.start.starts_with('0') && range.start.len() > 1 {
                                                            format!("{:0width$}", i, width = range.start.len())
                                                        } else {
                                                            i.to_string()
                                                        }
                                                    }).collect()
                                                } else {
                                                    (end..=start).rev().step_by((-step) as usize).map(|i| {
                                                        if range.start.starts_with('0') && range.start.len() > 1 {
                                                            format!("{:0width$}", i, width = range.start.len())
                                                        } else {
                                                            i.to_string()
                                                        }
                                                    }).collect()
                                                };
                                                for value in values {
                                                    all_files.push(format!("{}{}{}", prefix, value, suffix));
                                                }
                                            }
                                        }
                                        BraceItem::Sequence(seq) => {
                                            for s in seq {
                                                all_files.push(format!("{}{}{}", prefix, s, suffix));
                                            }
                                        }
                                    }
                                }
                            }
                            break; // Only handle the first brace expansion
                        }
                    }
                }
                
                // If no brace expansion pattern was found, handle each argument normally
                if all_files.is_empty() {
                    for arg in &cmd.args {
                        if let Word::BraceExpansion(expansion) = arg {
                            // Handle brace expansion
                            if expansion.items.len() == 1 {
                                match &expansion.items[0] {
                                    BraceItem::Range(range) => {
                                        if let (Ok(start), Ok(end)) = (range.start.parse::<i64>(), range.end.parse::<i64>()) {
                                            let step = range.step.as_ref().and_then(|s| s.parse::<i64>().ok()).unwrap_or(1);
                                            let values: Vec<String> = if step > 0 {
                                                (start..=end).step_by(step as usize).map(|i| {
                                                    // Preserve leading zeros by formatting with the same width as the original
                                                    if range.start.starts_with('0') && range.start.len() > 1 {
                                                        format!("{:0width$}", i, width = range.start.len())
                                                    } else {
                                                        i.to_string()
                                                    }
                                                }).collect()
                                            } else {
                                                (end..=start).rev().step_by((-step) as usize).map(|i| {
                                                    if range.start.starts_with('0') && range.start.len() > 1 {
                                                        format!("{:0width$}", i, width = range.start.len())
                                                    } else {
                                                        i.to_string()
                                                    }
                                                }).collect()
                                            };
                                            for value in values {
                                                all_files.push(value);
                                            }
                                        }
                                    }
                                    _ => {
                                        // For other brace items, just add the literal
                                        all_files.push(self.word_to_perl(arg));
                                    }
                                }
                            } else {
                                // Multiple items - expand each one
                                for item in &expansion.items {
                                    match item {
                                        BraceItem::Literal(s) => all_files.push(s.clone()),
                                        BraceItem::Range(range) => {
                                            if let (Ok(start), Ok(end)) = (range.start.parse::<i64>(), range.end.parse::<i64>()) {
                                                let step = range.step.as_ref().and_then(|s| s.parse::<i64>().ok()).unwrap_or(1);
                                                let values: Vec<String> = if step > 0 {
                                                    (start..=end).step_by(step as usize).map(|i| {
                                                        // Preserve leading zeros by formatting with the same width as the original
                                                        if range.start.starts_with('0') && range.start.len() > 1 {
                                                            format!("{:0width$}", i, width = range.start.len())
                                                        } else {
                                                            i.to_string()
                                                        }
                                                    }).collect()
                                                } else {
                                                    (end..=start).rev().step_by((-step) as usize).map(|i| {
                                                        if range.start.starts_with('0') && range.start.len() > 1 {
                                                            format!("{:0width$}", i, width = range.start.len())
                                                        } else {
                                                            i.to_string()
                                                        }
                                                    }).collect()
                                                };
                                                for value in values {
                                                    all_files.push(value);
                                                }
                                            }
                                        }
                                        BraceItem::Sequence(seq) => {
                                            for s in seq {
                                                all_files.push(s.clone());
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            // Regular argument
                            let arg_str = self.word_to_perl(arg);
                            all_files.push(arg_str);
                        }
                    }
                }
                
                // Now create all the files
                for file in all_files {
                    let fh = self.get_unique_file_handle();
                    output.push_str(&format!("open(my {}, '>', '{}') or die \"Cannot create file: $!\\n\";\n", fh, file));
                    output.push_str(&format!("close({});\n", fh));
                }
            }
        } else if cmd.name == "cd" {
            // Special handling for cd
            let empty_word = Word::Literal("".to_string());
            let dir = cmd.args.first().unwrap_or(&empty_word);
            output.push_str(&format!("chdir('{}') or die \"Cannot change to directory: $!\\n\";\n", dir));
        } else if cmd.name == "rm" {
            // Special handling for rm with brace expansion support
            if !cmd.args.is_empty() {
                // Expand brace expansion in arguments
                let mut expanded_args = Vec::new();
                for arg in &cmd.args {
                    let expanded = self.word_to_perl(arg);
                    expanded_args.push(expanded);
                }
                
                // Remove each file
                for arg in expanded_args {
                    if arg.contains('*') {
                        // Handle glob patterns like file_*.txt
                        let dh = self.get_unique_dir_handle();
                        output.push_str(&format!("opendir(my {}, '.') or die \"Cannot open directory: $!\\n\";\n", dh));
                        output.push_str(&format!("while (my $file = readdir({})) {{\n", dh));
                        output.push_str(&format!("    if ($file =~ /^{}$/) {{\n", arg.replace('*', ".*")));
                        output.push_str("        unlink($file) or die \"Cannot remove file: $!\\n\";\n");
                        output.push_str("    }\n");
                        output.push_str("}\n");
                        output.push_str(&format!("closedir({});\n", dh));
                    } else {
                        // Regular file
                        output.push_str(&format!("unlink('{}') or die \"Cannot remove file: $!\\n\";\n", arg));
                    }
                }
            }
        } else if cmd.name == "ls" {
            // Special handling for ls with brace expansion support
            if cmd.args.is_empty() {
                // Default to current directory
                let dh = self.get_unique_dir_handle();
                output.push_str(&format!("opendir(my {}, '.') or die \"Cannot open directory: $!\\n\";\n", dh));
                output.push_str(&format!("while (my $file = readdir({})) {{\n", dh));
                output.push_str("    print(\"$file\\n\") unless $file =~ /^\\.\\.?$/;\n");
                output.push_str("}\n");
                output.push_str(&format!("closedir({});\n", dh));
            } else {
                // Handle arguments with potential brace expansion
                let mut expanded_args = Vec::new();
                for arg in &cmd.args {
                    if arg.starts_with('-') {
                        // Skip flags
                        continue;
                    }
                    // Expand brace expansion in the argument
                    let expanded = self.word_to_perl(arg);
                    expanded_args.push(expanded);
                }
                
                if expanded_args.is_empty() {
                    // No non-flag arguments, default to current directory
                    let dh = self.get_unique_dir_handle();
                    output.push_str(&format!("opendir(my {}, '.') or die \"Cannot open directory: $!\\n\";\n", dh));
                    output.push_str(&format!("while (my $file = readdir({})) {{\n", dh));
                    output.push_str("    print(\"$file\\n\") unless $file =~ /^\\.\\.?$/;\n");
                    output.push_str("}\n");
                    output.push_str(&format!("closedir({});\n", dh));
                } else {
                    // Handle each expanded argument
                    for arg in expanded_args {
                        if arg.contains('*') {
                            // Handle glob patterns like file_*.txt
                            let pattern = arg.replace('*', ".*");
                            let dh = self.get_unique_dir_handle();
                            output.push_str(&format!("opendir(my {}, '.') or die \"Cannot open directory: $!\\n\";\n", dh));
                            output.push_str(&format!("while (my $file = readdir({})) {{\n", dh));
                            output.push_str(&format!("    if ($file =~ /^{}$/) {{\n", pattern));
                            output.push_str("        print(\"$file\\n\");\n");
                            output.push_str("    }\n");
                            output.push_str("}\n");
                            output.push_str(&format!("closedir({});\n", dh));
                        } else {
                            // Regular directory/file
                            let dh = self.get_unique_dir_handle();
                            output.push_str(&format!("opendir(my {}, '{}') or die \"Cannot open directory: $!\\n\";\n", dh, arg));
                            output.push_str(&format!("while (my $file = readdir({})) {{\n", dh));
                            output.push_str("    print(\"$file\\n\") unless $file =~ /^\\.\\.?$/;\n");
                            output.push_str("}\n");
                            output.push_str(&format!("closedir({});\n", dh));
                        }
                    }
                }
            }
        } else if cmd.name == "grep" {
            // Special handling for grep
            if cmd.args.len() >= 1 {
                // Find the pattern (first non-flag argument)
                let mut pattern = None;
                let mut file = None;
                let mut flags = Vec::new();
                
                for arg in &cmd.args {
                    if arg.starts_with('-') {
                        flags.push(arg.as_str());
                    } else if pattern.is_none() {
                        pattern = Some(arg);
                    } else if file.is_none() {
                        file = Some(arg);
                    }
                }
                
                if let Some(pattern) = pattern {
                    let file = file.map_or("STDIN", |w| w.as_str());
                    
                    // Check for -o flag (only matching part)
                    let only_matching = flags.iter().any(|&flag| flag == "-o");
                    
                    if only_matching {
                        if file == "STDIN" {
                            output.push_str("while (my $line = <STDIN>) {\n");
                            output.push_str(&format!("    if ($line =~ /({})/g) {{\n", pattern));
                            output.push_str("        print \"$1\\n\";\n");
                            output.push_str("    }\n");
                            output.push_str("}\n");
                        } else {
                            let fh = self.get_unique_file_handle();
                            output.push_str(&format!("open(my {}, '<', '{}') or die \"Cannot open file: $!\\n\";\n", fh, file));
                            output.push_str(&format!("while (my $line = <{}) {{\n", fh));
                            output.push_str(&format!("    if ($line =~ /({})/g) {{\n", pattern));
                            output.push_str("        print \"$1\\n\";\n");
                            output.push_str("    }\n");
                            output.push_str("}\n");
                            output.push_str(&format!("close({});\n", fh));
                        }
                    } else {
                        if file == "STDIN" {
                            output.push_str("while (my $line = <STDIN>) {\n");
                            output.push_str(&format!("    print($line) if $line =~ /{}/;\n", pattern));
                            output.push_str("}\n");
                        } else {
                            let fh = self.get_unique_file_handle();
                            output.push_str(&format!("open(my {}, '<', '{}') or die \"Cannot open file: $!\\n\";\n", fh, file));
                            output.push_str(&format!("while (my $line = <{}) {{\n", fh));
                            output.push_str(&format!("    print($line) if $line =~ /{}/;\n", pattern));
                            output.push_str("}\n");
                            output.push_str(&format!("close({});\n", fh));
                        }
                    }
                }
            }
        } else if cmd.name == "cat" {
            // Special handling for cat including heredocs
            // If there are heredoc redirects attached, emit their bodies inline
            let mut printed_any = false;
            for redir in &cmd.redirects {
                if matches!(redir.operator, RedirectOperator::Heredoc | RedirectOperator::HeredocTabs) {
                    if let Some(body) = &redir.heredoc_body {
                        output.push_str(&format!("print <<'{}';\n{}\n{}\n;\n", redir.target, body, redir.target));
                        printed_any = true;
                    }
                }
            }
            if !printed_any {
                for arg in &cmd.args {
                    let fh = self.get_unique_file_handle();
                    output.push_str(&format!("open(my {}, '<', '{}') or die \"Cannot open file: $!\\n\";\n", fh, arg));
                    output.push_str(&format!("while (my $line = <{}) {{\n", fh));
                    output.push_str("    print($line);\n");
                    output.push_str("}\n");
                    output.push_str(&format!("close({});\n", fh));
                }
            }
        } else if cmd.name == "mkdir" {
            // Special handling for mkdir
            for arg in &cmd.args {
                output.push_str(&format!("mkdir('{}') or die \"Cannot create directory: $!\\n\";\n", arg));
            }
        } else if cmd.name == "rm" {
            // Special handling for rm
            for arg in &cmd.args {
                output.push_str(&format!("unlink('{}') or die \"Cannot remove file: $!\\n\";\n", arg));
            }
        } else if cmd.name == "mv" {
            // Special handling for mv
            if cmd.args.len() >= 2 {
                let src = &cmd.args[0];
                let dst = &cmd.args[1];
                output.push_str(&format!("rename('{}', '{}') or die \"Cannot move file: $!\\n\";\n", src, dst));
            }
        } else if cmd.name == "cp" {
            // Special handling for cp
            if cmd.args.len() >= 2 {
                let src = &cmd.args[0];
                let dst = &cmd.args[1];
                output.push_str(&format!("use File::Copy;\n"));
                output.push_str(&format!("copy('{}', '{}') or die \"Cannot copy file: $!\\n\";\n", src, dst));
            }
        } else if cmd.name == "mapfile" {
            // Handle mapfile command for reading lines into an array
            if cmd.args.len() >= 2 && cmd.args[0] == "-t" {
                let array_name = &cmd.args[1];
                output.push_str(&format!("my @{} = ();\n", array_name));
                output.push_str(&format!("while (my $line = <STDIN>) {{\n"));
                output.push_str(&format!("    chomp $line;\n"));
                output.push_str(&format!("    push @{}, $line;\n", array_name));
                output.push_str("}\n");
                if self.subshell_depth == 0 {
                    self.declared_locals.insert(array_name.to_string());
                }
            }
        } else if cmd.name == "comm" {
            // Handle comm command for comparing sorted files
            if cmd.args.len() >= 3 {
                let flag = &cmd.args[0];
                let file1 = &cmd.args[1];
                let file2 = &cmd.args[2];
                output.push_str(&format!("# comm {} {} {}\n", flag, file1, file2));
                output.push_str("system('comm', ");
                output.push_str(&format!("{}, {}, {});\n", 
                    self.perl_string_literal(flag),
                    self.perl_string_literal(file1),
                    self.perl_string_literal(file2)));
            }
        } else if cmd.name == "test" || cmd.name == "[" {
            // Special handling for test
            self.generate_test_command(cmd, &mut output);
        } else if cmd.name == "[[" {
            // Handle [[ ... ]] test command with pattern matching and regex
            if cmd.args.len() >= 3 {
                let left = &cmd.args[0];
                let operator = &cmd.args[1];
                let right = &cmd.args[2];
                
                match operator.as_str() {
                    "==" => {
                        // Pattern matching: [[ $var == pattern ]]
                        output.push_str(&format!("if (${} =~ /{}/) {{\n", left, right));
                        output.push_str("    # Pattern match succeeded\n");
                        output.push_str("}\n");
                    }
                    "=~" => {
                        // Regex matching: [[ $var =~ regex ]]
                        output.push_str(&format!("if (${} =~ /{}/) {{\n", left, right));
                        output.push_str("    # Regex match succeeded\n");
                        output.push_str("}\n");
                    }
                    _ => {
                        // Other operators not yet implemented
                        output.push_str(&format!("# [[ {} {} {} ]] not implemented\n", left, operator, right));
                        output.push_str("1;\n");
                    }
                }
            } else {
                // Simple [[ ... ]] without enough args
                output.push_str("1;\n");
            }
        } else if cmd.name == "shopt" {
            // Handle shopt command for shell options
            if cmd.args.len() >= 2 && cmd.args[0] == "-s" {
                let option = &cmd.args[1];
                if option == "extglob" {
                    output.push_str("# extglob option enabled\n");
                } else if option == "nocasematch" {
                    output.push_str("# nocasematch option enabled\n");
                } else {
                    output.push_str(&format!("# shopt -s {} not implemented\n", option));
                }
            } else {
                // Other shopt options not yet implemented
                output.push_str("# shopt option not implemented\n");
            }
            // shopt commands always succeed (return true)
            output.push_str("1;\n");
        } else if cmd.name == "set" {
            // Handle set command for shell options
            if cmd.args.len() >= 1 {
                let options = &cmd.args[0];
                if options.contains('e') {
                    output.push_str("$SIG{__DIE__} = sub { die @_; };\n");
                }
                if options.contains('u') {
                    output.push_str("use strict;\n");
                }
                if options.contains('o') {
                    // Handle -o pipefail
                    if cmd.args.len() >= 2 && cmd.args[1] == "pipefail" {
                        output.push_str("# pipefail option not implemented in Perl\n");
                    }
                }
            }
        } else if cmd.name == "declare" {
            // Handle declare command for associative arrays
            if cmd.args.len() >= 2 && matches!(&cmd.args[0], Word::Literal(lit) if lit == "-A") {
                if let Word::Literal(array_name) = &cmd.args[1] {
                    output.push_str(&format!("my %{} = ();\n", array_name));
                    if self.subshell_depth == 0 {
                        self.declared_locals.insert(array_name.to_string());
                    }
                } else {
                    // Skip if not a literal
                    output.push_str(&format!("# declare {:?} not yet implemented\n", cmd.args));
                }
            } else {
                // Other declare options not yet implemented
                output.push_str(&format!("# declare {:?} not yet implemented\n", cmd.args));
            }
        } else if cmd.name == "export" {
            // Persistently set environment variables provided as VAR=VAL pairs
            for arg in &cmd.args {
                if let Some(eq_idx) = arg.find('=') {
                    let (k, v) = arg.split_at(eq_idx);
                    let v2 = if v.len() > 0 { &v[1..] } else { "" };
                    output.push_str(&format!("$ENV{{{}}} = {};;\n", k, self.perl_string_literal(v2)));
                }
            }
        } else {
            // Check if this might be a function call (not a builtin)
            let builtins = ["echo", "cd", "ls", "grep", "cat", "mkdir", "rm", "mv", "cp", "test", "[", "[[", "shopt", "export", "declare", "true", "false"];
            if !builtins.contains(&cmd.name.as_str()) {
                // Check if this is an array assignment like map[foo]=bar
                if cmd.name.contains('[') && cmd.name.ends_with(']') {
                    if let Some(bracket_start) = cmd.name.find('[') {
                        let array_name = &cmd.name[..bracket_start];
                        let key = &cmd.name[bracket_start + 1..cmd.name.len() - 1];
                        if let Some(value) = cmd.args.first() {
                            let perl_value = self.word_to_perl(value);
                            output.push_str(&format!("${}{{{}}} = {};\n", array_name, key, perl_value));
                        }
                    }
                } else if self.declared_functions.contains(&cmd.name.to_string()) {
                    // This is a call to a defined function
                    let args = cmd
                        .args
                        .iter()
                        .map(|arg| self.word_to_perl(arg))
                        .collect::<Vec<_>>();
                    if args.is_empty() {
                        output.push_str(&format!("{}();\n", cmd.name));
                    } else {
                        output.push_str(&format!("{}({});\n", cmd.name, args.join(", ")));
                    }
                } else {
                    // Non-builtin command - use system() for external commands
                    let name = self.perl_string_literal(&cmd.name);
                    
                    // Special handling for touch command with brace expansions
                    if cmd.name == "touch" {
                        let mut expanded_args = Vec::new();
                        for arg in &cmd.args {
                            match arg {
                                Word::BraceExpansion(expansion) => {
                                    // Expand brace expansion for touch command
                                    if expansion.items.len() == 1 {
                                        match &expansion.items[0] {
                                            BraceItem::Range(range) => {
                                                if let (Ok(start), Ok(end)) = (range.start.parse::<i64>(), range.end.parse::<i64>()) {
                                                    let step = range.step.as_ref().and_then(|s| s.parse::<i64>().ok()).unwrap_or(1);
                                                    let values: Vec<String> = if step > 0 {
                                                        (start..=end).step_by(step as usize).map(|i| i.to_string()).collect()
                                                    } else {
                                                        (end..=start).rev().step_by((-step) as usize).map(|i| i.to_string()).collect()
                                                    };
                                                    for value in values {
                                                        expanded_args.push(format!("\"file_{:03}.txt\"", value));
                                                    }
                                                } else {
                                                    expanded_args.push(self.perl_string_literal(arg));
                                                }
                                            }
                                            _ => expanded_args.push(self.perl_string_literal(arg)),
                                        }
                                    } else {
                                        expanded_args.push(self.perl_string_literal(arg));
                                    }
                                }
                                _ => expanded_args.push(self.perl_string_literal(arg)),
                            }
                        }
                        if expanded_args.is_empty() {
                            output.push_str(&format!("system({});\n", name));
                        } else {
                            output.push_str(&format!("system({}, {});\n", name, expanded_args.join(", ")));
                        }
                    } else {
                        // Regular command handling
                        let args = cmd
                            .args
                            .iter()
                            .map(|arg| self.perl_string_literal(arg))
                            .collect::<Vec<_>>();
                        if args.is_empty() {
                            output.push_str(&format!("system({});\n", name));
                        } else {
                            output.push_str(&format!("system({}, {});\n", name, args.join(", ")));
                        }
                    }
                }
            } else {
                // Builtin command - handle as before
                let args = cmd
                    .args
                    .iter()
                    .map(|arg| self.word_to_perl(arg))
                    .collect::<Vec<_>>();
                if args.is_empty() {
                    output.push_str(&format!("{}();\n", cmd.name));
                } else {
                    output.push_str(&format!("{}({});\n", cmd.name, args.join(", ")));
                }
            }
        }
        
        // Handle redirects
        for redir in &cmd.redirects {
            match redir.operator {
                RedirectOperator::Input => {
                    // Input redirection: command < file
                    output.push_str(&format!("open(STDIN, '<', '{}') or die \"Cannot open file: $!\\n\";\n", redir.target));
                }
                RedirectOperator::Output => {
                    // Output redirection: command > file
                    output.push_str(&format!("open(STDOUT, '>', '{}') or die \"Cannot open file: $!\\n\";\n", redir.target));
                }
                RedirectOperator::Append => {
                    // Append redirection: command >> file
                    output.push_str(&format!("open(STDOUT, '>>', '{}') or die \"Cannot open file: $!\\n\";\n", redir.target));
                }
                RedirectOperator::HereString => {
                    // Here-string: command <<< "string"
                    if let Some(body) = &redir.heredoc_body {
                        // Create a temporary file with the string content
                        output.push_str(&format!("my $temp_content = {};\n", self.perl_string_literal(body)));
                        let fh = self.get_unique_file_handle();
                        output.push_str(&format!("open(my {}, '>', '/tmp/here_string_temp') or die \"Cannot create temp file: $!\\n\";\n", fh));
                        output.push_str(&format!("print {} $temp_content;\n", fh));
                        output.push_str(&format!("close({});\n", fh));
                        output.push_str("open(STDIN, '<', '/tmp/here_string_temp') or die \"Cannot open temp file: $!\\n\";\n");
                    }
                }
                RedirectOperator::Heredoc | RedirectOperator::HeredocTabs => {
                    // Heredoc: command << delimiter
                    // Skip heredoc handling for 'cat' command since it's handled specially in the cat command handler
                    if cmd.name != "cat" {
                        if let Some(body) = &redir.heredoc_body {
                            // Create a temporary file with the heredoc content
                            output.push_str(&format!("my $temp_content = {};\n", self.perl_string_literal(body)));
                            let fh = self.get_unique_file_handle();
                            output.push_str(&format!("open(my {}, '>', '/tmp/heredoc_temp') or die \"Cannot create temp file: $!\\n\";\n", fh));
                            output.push_str(&format!("print {} $temp_content;\n", fh));
                            output.push_str(&format!("close({});\n", fh));
                            output.push_str("open(STDIN, '<', '/tmp/heredoc_temp') or die \"Cannot open temp file: $!\\n\";\n");
                        }
                    }
                }
                _ => {
                    // Other redirects not yet implemented
                    output.push_str(&format!("# Redirect {:?} not yet implemented\n", redir.operator));
                }
            }
        }
        
        if has_env { output.push_str("}\n"); }
        output
    }

    fn generate_shopt_command(&mut self, cmd: &ShoptCommand) -> String {
        let mut output = String::new();
        
        // Handle shopt command for shell options
        if cmd.enable {
            match cmd.option.as_str() {
                "extglob" => {
                    output.push_str("# extglob option enabled\n");
                }
                "nocasematch" => {
                    output.push_str("# nocasematch option enabled\n");
                }
                _ => {
                    output.push_str(&format!("# shopt -s {} not implemented\n", cmd.option));
                }
            }
        } else {
            match cmd.option.as_str() {
                "extglob" => {
                    output.push_str("# extglob option disabled\n");
                }
                "nocasematch" => {
                    output.push_str("# nocasematch option disabled\n");
                }
                _ => {
                    output.push_str(&format!("# shopt -u {} not implemented\n", cmd.option));
                }
            }
        }
        
        // shopt commands always succeed (return true)
        output
    }
    
    fn generate_builtin_command(&mut self, cmd: &BuiltinCommand) -> String {
        let mut output = String::new();
        
        // Handle environment variables if any
        let has_env = !cmd.env_vars.is_empty();
        if has_env {
            output.push_str("{\n");
            for (var, value) in &cmd.env_vars {
                let val = self.perl_string_literal(value);
                output.push_str(&format!("local $ENV{{{}}} = {};;\n", var, val));
            }
        }
        
        // Generate the builtin command
        match cmd.name.as_str() {
            "set" => {
                // Convert shell set options to Perl equivalents
                for arg in &cmd.args {
                    if let Word::Literal(opt) = arg {
                        match opt.as_str() {
                            "-e" => output.push_str("$SIG{__DIE__} = sub { exit 1 };\n"),
                            "-u" => output.push_str("use strict;\n"),
                            "-o" => {
                                // Handle pipefail and other options
                                if let Some(_next_arg) = cmd.args.iter().skip(1).find(|a| {
                                    if let Word::Literal(s) = a { s == "pipefail" } else { false }
                                }) {
                                    output.push_str("# set -o pipefail\n");
                                }
                            }
                            _ => output.push_str(&format!("# set {}\n", opt)),
                        }
                    }
                }
            }
            "export" => {
                // Convert export to Perl environment variable assignment
                for arg in &cmd.args {
                    if let Word::Literal(var) = arg {
                        if var.contains('=') {
                            let parts: Vec<&str> = var.splitn(2, '=').collect();
                            if parts.len() == 2 {
                                let var_name = parts[0];
                                let var_value = self.perl_string_literal(parts[1]);
                                output.push_str(&format!("$ENV{{{}}} = {};\n", var_name, var_value));
                            }
                        } else {
                            output.push_str(&format!("# export {}\n", var));
                        }
                    }
                }
            }
            "local" => {
                // Convert local to Perl my declaration
                for arg in &cmd.args {
                    if let Word::Literal(var) = arg {
                        if var.contains('=') {
                            let parts: Vec<&str> = var.splitn(2, '=').collect();
                            if parts.len() == 2 {
                                let var_name = parts[0];
                                let var_value = self.perl_string_literal(parts[1]);
                                output.push_str(&format!("my ${} = {};\n", var_name, var_value));
                                self.declared_locals.insert(var_name.to_string());
                            }
                        } else {
                            output.push_str(&format!("my ${};\n", var));
                            self.declared_locals.insert(var.to_string());
                        }
                    }
                }
            }
            "unset" => {
                // Convert unset to Perl undef and ensure variable is declared
                for arg in &cmd.args {
                    if let Word::Literal(var) = arg {
                        // First declare the variable if it's not already declared
                        if !self.declared_locals.contains(var) {
                            output.push_str(&format!("my ${} = undef;\n", var));
                            self.declared_locals.insert(var.to_string());
                        }
                        output.push_str(&format!("undef ${};\n", var));
                    }
                }
            }
            _ => {
                // For other builtins, generate a comment
                output.push_str(&format!("# {} {}\n", cmd.name, 
                    cmd.args.iter().map(|arg| self.word_to_perl(arg)).collect::<Vec<_>>().join(" ")));
            }
        }
        
        // Close environment variable block if needed
        if has_env {
            output.push_str("}\n");
        }
        
        output
    }

    fn generate_test_expression(&mut self, test_expr: &TestExpression) -> String {
        // Parse the test expression to extract components
        let expr = &test_expr.expression;
        let modifiers = &test_expr.modifiers;
        
        // Parse the expression to determine the type of test
        if expr.contains(" =~ ") {
            // Regex matching: [[ $var =~ pattern ]]
            let parts: Vec<&str> = expr.split(" =~ ").collect();
            if parts.len() == 2 {
                let var = parts[0].trim();
                let pattern = parts[1].trim();
                
                // Convert to Perl regex matching
                format!("({} =~ /{}/)", var, pattern)
            } else {
                "0".to_string()
            }
        } else if expr.contains(" == ") {
            // Pattern matching: [[ $var == pattern ]]
            let parts: Vec<&str> = expr.split(" == ").collect();
            if parts.len() == 2 {
                let var = parts[0].trim();
                let pattern = parts[1].trim();
                
                if modifiers.extglob {
                    // Handle extglob patterns
                    let regex_pattern = self.convert_extglob_to_perl_regex(pattern);
                    if modifiers.nocasematch {
                        format!("({} =~ /{}/i)", var, regex_pattern)
                    } else {
                        format!("({} =~ /{}/)", var, regex_pattern)
                    }
                } else {
                    // Regular glob pattern matching - convert glob to regex
                    let regex_pattern = self.convert_glob_to_regex(pattern);
                    if modifiers.nocasematch {
                        // Case-insensitive matching
                        format!("({} =~ /^{}$/i)", var, regex_pattern)
                    } else {
                        // Case-sensitive matching
                        format!("({} =~ /^{}$/)", var, regex_pattern)
                    }
                }
            } else {
                "0".to_string()
            }
        } else if expr.contains(" != ") {
            // Pattern matching: [[ $var != pattern ]]
            let parts: Vec<&str> = expr.split(" != ").collect();
            if parts.len() == 2 {
                let var = parts[0].trim();
                let pattern = parts[1].trim();
                
                if modifiers.extglob {
                    // Handle extglob patterns
                    let regex_pattern = self.convert_extglob_to_perl_regex(pattern);
                    if modifiers.nocasematch {
                        format!("({} !~ /{}/i)", var, regex_pattern)
                    } else {
                        format!("({} !~ /{}/)", var, regex_pattern)
                    }
                } else {
                    // Regular pattern matching
                    if modifiers.nocasematch {
                        // Case-insensitive matching
                        format!("lc({}) !~ /^{}$/i", var, pattern.replace("*", ".*"))
                    } else {
                        // Case-sensitive matching
                        format!("{} !~ /^{}$/", var, pattern.replace("*", ".*"))
                    }
                }
            } else {
                "0".to_string()
            }
        } else if expr.contains(" -eq ") {
            // Numeric equality: [[ $var -eq value ]]
            let parts: Vec<&str> = expr.split(" -eq ").collect();
            if parts.len() == 2 {
                let var = parts[0].trim();
                let value = parts[1].trim();
                
                format!("{} == {}", var, value)
            } else {
                "0".to_string()
            }
        } else if expr.contains(" -ne ") {
            // Numeric inequality: [[ $var -ne value ]]
            let parts: Vec<&str> = expr.split(" -ne ").collect();
            if parts.len() == 2 {
                let var = parts[0].trim();
                let value = parts[1].trim();
                
                format!("{} != {}", var, value)
            } else {
                "0".to_string()
            }
        } else if expr.contains(" -lt ") {
            // Less than: [[ $var -lt value ]]
            let parts: Vec<&str> = expr.split(" -lt ").collect();
            if parts.len() == 2 {
                let var = parts[0].trim();
                let value = parts[1].trim();
                
                format!("{} < {}", var, value)
            } else {
                "0".to_string()
            }
        } else if expr.contains(" -le ") {
            // Less than or equal: [[ $var -le value ]]
            let parts: Vec<&str> = expr.split(" -le ").collect();
            if parts.len() == 2 {
                let var = parts[0].trim();
                let value = parts[1].trim();
                
                format!("{} <= {}", var, value)
            } else {
                "0".to_string()
            }
        } else if expr.contains(" -gt ") {
            // Greater than: [[ $var -gt value ]]
            let parts: Vec<&str> = expr.split(" -gt ").collect();
            if parts.len() == 2 {
                let var = parts[0].trim();
                let value = parts[1].trim();
                
                format!("{} > {}", var, value)
            } else {
                "0".to_string()
            }
        } else if expr.contains(" -ge ") {
            // Greater than or equal: [[ $var -ge value ]]
            let parts: Vec<&str> = expr.split(" -ge ").collect();
            if parts.len() == 2 {
                let var = parts[0].trim();
                let value = parts[1].trim();
                
                format!("{} >= {}", var, value)
            } else {
                "0".to_string()
            }
        } else if expr.contains(" -z ") {
            // String is empty: [[ -z $var ]]
            let var_str = expr.replace("-z", "").trim().to_string();
            format!("{} eq ''", var_str)
        } else if expr.contains(" -n ") {
            // String is not empty: [[ -n $var ]]
            let var_str = expr.replace("-n", "").trim().to_string();
            format!("{} ne ''", var_str)
        } else if expr.contains(" -f ") {
            // File exists and is regular: [[ -f $var ]]
            let var_str = expr.replace("-f", "").trim().to_string();
            format!("-f {}", var_str)
        } else if expr.contains(" -d ") {
            // Directory exists: [[ -d $var ]]
            let var_str = expr.replace("-d", "").trim().to_string();
            format!("-d {}", var_str)
        } else if expr.contains(" -e ") {
            // File exists: [[ -e $var ]]
            let var_str = expr.replace("-e", "").trim().to_string();
            format!("-e {}", var_str)
        } else if expr.contains(" -r ") {
            // File is readable: [[ -r $var ]]
            let var_str = expr.replace("-r", "").trim().to_string();
            format!("-r {}", var_str)
        } else if expr.contains(" -w ") {
            // File is writable: [[ -w $var ]]
            let var_str = expr.replace("-w", "").trim().to_string();
            format!("-w {}", var_str)
        } else if expr.contains(" -x ") {
            // File is executable: [[ -x $var ]]
            let var_str = expr.replace("-x", "").trim().to_string();
            format!("-x {}", var_str)
        } else {
            // Unknown test expression
            format!("0 # Unknown test: {}", expr)
        }
    }

    
    fn generate_test_command(&mut self, cmd: &SimpleCommand, output: &mut String) {
        // Convert test conditions to Perl
        if cmd.args.len() == 3 {
            // Format: [ operand1 operator operand2 ]
            let operand1 = &cmd.args[0];
            let operator = &cmd.args[1];
            let operand2 = &cmd.args[2];
            
            // Ensure variables are declared if they're shell variables
            if let Word::Variable(var_name) = operand1 {
                if !self.declared_locals.contains(var_name) {
                    output.push_str(&format!("my ${} = 0;\n", var_name));
                    self.declared_locals.insert(var_name.to_string());
                }
            }
            if let Word::Variable(var_name) = operand2 {
                if !self.declared_locals.contains(var_name) {
                    output.push_str(&format!("my ${} = 0;\n", var_name));
                    self.declared_locals.insert(var_name.to_string());
                }
            }
            
            match operator.as_str() {
                "-lt" => {
                    output.push_str(&format!("{} < {}", operand1, operand2));
                }
                "-le" => {
                    output.push_str(&format!("{} <= {}", operand1, operand2));
                }
                "-eq" => {
                    output.push_str(&format!("{} == {}", operand1, operand2));
                }
                "-ne" => {
                    output.push_str(&format!("{} != {}", operand1, operand2));
                }
                "-gt" => {
                    output.push_str(&format!("{} > {}", operand1, operand2));
                }
                "-ge" => {
                    output.push_str(&format!("{} >= {}", operand1, operand2));
                }
                _ => {
                    output.push_str(&format!("{} {} {}", operand1, operator, operand2));
                }
            }
        } else if cmd.args.len() >= 2 {
            let operator = &cmd.args[0];
            let operand = &cmd.args[1];
            
            // Ensure variables are declared if they're shell variables
            if let Word::Variable(var_name) = operand {
                if !self.declared_locals.contains(var_name) {
                    output.push_str(&format!("my ${} = 0;\n", var_name));
                    self.declared_locals.insert(var_name.to_string());
                }
            }
            
            match operator.as_str() {
                "-f" => {
                    output.push_str(&format!("-f {}", self.word_to_perl_for_test(operand)));
                }
                "-d" => {
                    output.push_str(&format!("-d {}", self.word_to_perl_for_test(operand)));
                }
                "-e" => {
                    output.push_str(&format!("-e {}", self.word_to_perl_for_test(operand)));
                }
                "-r" => {
                    output.push_str(&format!("-r {}", self.word_to_perl_for_test(operand)));
                }
                "-w" => {
                    output.push_str(&format!("-w {}", self.word_to_perl_for_test(operand)));
                }
                "-x" => {
                    output.push_str(&format!("-x {}", self.word_to_perl_for_test(operand)));
                }
                "-z" => {
                    output.push_str(&format!("-z {}", self.word_to_perl_for_test(operand)));
                }
                "-n" => {
                    output.push_str(&format!("-s {}", self.word_to_perl_for_test(operand)));
                }
                _ => {
                    output.push_str(&format!("{} {} {}", self.word_to_perl_for_test(operand), operator, self.word_to_perl_for_test(operand)));
                }
            }
        }
    }

    fn generate_pipeline(&mut self, pipeline: &Pipeline) -> String {
        let mut output = String::new();
        
        let has_pipe = pipeline.operators.iter().any(|op| matches!(op, PipeOperator::Pipe));
        if pipeline.commands.len() == 1 {
            output.push_str(&self.generate_command(&pipeline.commands[0]));
        } else if has_pipe {
            // Handle pipelines with for loops and other commands
            // Use unique variable names for each pipeline to avoid redeclaration warnings
            self.pipeline_counter += 1;
            let pipeline_id = self.pipeline_counter;
            output.push_str(&format!("my $output_{};\n", pipeline_id));
            
            // Check if the first command is a for loop
            if let Command::For(for_loop) = &pipeline.commands[0] {
                // Generate the for loop directly in Perl and capture its output
                let variable = &for_loop.variable;
                let items = &for_loop.items;
                
                // Convert items to Perl array syntax
                let items_str = if items.len() == 1 {
                    match &items[0] {
                        Word::StringInterpolation(interp) => {
                            if interp.parts.len() == 1 {
                                if let StringPart::MapAccess(map_name, key) = &interp.parts[0] {
                                    if key == "@" {
                                        format!("@{}", map_name)
                                    } else {
                                        format!("@{}", map_name)
                                    }
                                } else if let StringPart::MapKeys(map_name) = &interp.parts[0] {
                                    // This is ${!map[@]} - convert to keys(%map)
                                    format!("keys(%{})", map_name)
                                } else if let StringPart::Variable(var) = &interp.parts[0] {
                                    if var.starts_with("!") && var.ends_with("[@]") {
                                        // This is !map[@] - convert to keys(%map)
                                        let map_name = &var[1..var.len()-3];
                                        format!("keys(%{})", map_name)
                                    } else if var.ends_with("[@]") {
                                        let array_name = &var[..var.len()-3];
                                        format!("@{}", array_name)
                                    } else {
                                        format!("@{}", var)
                                    }
                                } else {
                                    format!("@{}", items[0])
                                }
                            } else {
                                format!("@{}", items[0])
                            }
                        }
                        Word::MapAccess(map_name, key) => {
                            if key == "@" {
                                format!("@{}", map_name)
                            } else {
                                format!("@{}", map_name)
                            }
                        }
                        _ => format!("@{}", items[0])
                    }
                } else {
                    format!("({})", items.iter().map(|s| format!("\"{}\"", self.word_to_perl(s))).collect::<Vec<_>>().join(", "))
                };
                
                // Generate the for loop that builds the output string for the pipeline
                output.push_str(&format!("for my ${} ({}) {{\n", variable, items_str));
                // Instead of printing directly, build the output string
                for cmd in &for_loop.body.commands {
                    if let Command::Simple(simple_cmd) = cmd {
                        if simple_cmd.name == "echo" {
                            // Convert echo to building output string
                            let mut echo_parts = Vec::new();
                            for arg in &simple_cmd.args {
                                match arg {
                                    Word::StringInterpolation(interp) => {
                                        // Handle string interpolation by converting to Perl string concatenation
                                        let parts: Vec<String> = interp.parts.iter().map(|part| {
                                            match part {
                                                StringPart::Literal(lit) => format!("\"{}\"", self.escape_perl_string(lit)),
                                                StringPart::Variable(var) => format!("${}", var),
                                                StringPart::MapAccess(map_name, key) => {
                                                    if key.starts_with('$') {
                                                        // Key is a variable like $k
                                                        format!("${}{{{}}}", map_name, format!("${}", &key[1..]))
                                                    } else {
                                                        // Key is a literal
                                                        format!("${}{{{}}}", map_name, key)
                                                    }
                                                }
                                                StringPart::MapKeys(map_name) => {
                                                    // ${!map[@]} -> keys(%map)
                                                    format!("keys(%{})", map_name)
                                                }
                                                _ => format!("{:?}", part)
                                            }
                                        }).collect();
                                        echo_parts.push(format!("{}", parts.join(" . ")));
                                    }
                                    _ => {
                                        // For non-interpolated words, just convert normally
                                        echo_parts.push(self.word_to_perl(arg));
                                    }
                                }
                            }
                            let echo_str = echo_parts.join(" . ");
                            output.push_str(&self.indent());
                            output.push_str(&format!("$output_{} .= {} . \"\\n\";\n", pipeline_id, echo_str));
                        } else {
                            // For other commands, generate normally but capture output
                            output.push_str(&self.indent());
                            output.push_str(&format!("$output_{} .= `{}`;\n", pipeline_id, self.command_to_string(&Command::Simple(simple_cmd.clone()))));
                        }
                    } else {
                        // For non-simple commands, generate normally but capture output
                        output.push_str(&self.indent());
                        output.push_str(&format!("$output_{} .= `{}`;\n", pipeline_id, self.command_to_string(cmd)));
                    }
                }
                output.push_str("}\n");
            } else {
                // First command - check if it's a find command and handle specially
                if let Command::Simple(cmd) = &pipeline.commands[0] {
                    if cmd.name == "find" {
                        // Handle find command with Windows compatibility
                        let mut find_args = Vec::new();
                        for arg in &cmd.args {
                            match arg {
                                Word::Literal(s) => {
                                    if s == "." {
                                        find_args.push(".".to_string());
                                    } else if s == "-name" {
                                        find_args.push("-name".to_string());
                                    } else {
                                        // Convert glob pattern to regex for find
                                        let pattern = self.convert_glob_to_regex(s);
                                        find_args.push(pattern);
                                    }
                                }
                                Word::StringInterpolation(interp) => {
                                    if interp.parts.len() == 1 {
                                        if let StringPart::Literal(s) = &interp.parts[0] {
                                            // Convert glob pattern to regex for find
                                            let pattern = self.convert_glob_to_regex(s);
                                            find_args.push(pattern);
                                        } else {
                                            find_args.push(self.convert_string_interpolation_to_perl(interp));
                                        }
                                    } else {
                                        find_args.push(self.convert_string_interpolation_to_perl(interp));
                                    }
                                }
                                _ => find_args.push(self.word_to_perl(arg))
                            }
                        }
                        
                        // Use Perl's File::Find instead of system find for cross-platform compatibility
                        if find_args.len() >= 3 && find_args[1] == "-name" {
                            let pattern = &find_args[2];
                            let dir = &find_args[0];
                            self.needs_file_find = true;
                            output.push_str(&format!("my @find_files_{};\n", pipeline_id));
                            // The pattern is already a regex from convert_glob_to_regex, no need to escape again
                            output.push_str(&format!("find({{wanted => sub {{ if ($_ =~ /{}/) {{ push @find_files_{}, $File::Find::name; }} }}, no_chdir => 1}}, '{}');\n", pattern, pipeline_id, dir));
                            output.push_str(&format!("$output_{} = join(\"\\n\", @find_files_{});\n", pipeline_id, pipeline_id));
                        } else {
                            // Fallback to system find command
                            let cmd_str = self.command_to_string(&pipeline.commands[0]);
                            let escaped_cmd = cmd_str.replace("'", "'\"'\"'");
                            output.push_str(&format!("$output_{} = `{}`;\n", pipeline_id, escaped_cmd));
                        }
                    } else if cmd.name == "ls" {
                        // Handle ls command natively in Perl
                        let mut ls_args = Vec::new();
                        for arg in &cmd.args {
                            match arg {
                                Word::Literal(s) => ls_args.push(s.clone()),
                                Word::StringInterpolation(interp) => ls_args.push(self.convert_string_interpolation_to_perl(interp)),
                                _ => ls_args.push(self.word_to_perl(arg))
                            }
                        }
                        
                        let dir = if ls_args.is_empty() { "." } else { &ls_args[0] };
                        output.push_str(&format!("my @ls_files_{};\n", pipeline_id));
                        output.push_str(&format!("if (opendir(my $dh_{}, '{}')) {{\n", pipeline_id, dir));
                        output.push_str(&format!("    while (my $file = readdir($dh_{})) {{\n", pipeline_id));
                        output.push_str(&format!("        next if $file eq '.' || $file eq '..';\n"));
                        output.push_str(&format!("        push @ls_files_{}, $file;\n", pipeline_id));
                        output.push_str("    }\n");
                        output.push_str(&format!("    closedir($dh_{});\n", pipeline_id));
                        output.push_str("}\n");
                        output.push_str(&format!("$output_{} = join(\"\\n\", @ls_files_{});\n", pipeline_id, pipeline_id));
                    } else if cmd.name == "cat" {
                        // Handle cat command natively in Perl
                        let mut cat_args = Vec::new();
                        for arg in &cmd.args {
                            match arg {
                                Word::Literal(s) => cat_args.push(s.clone()),
                                Word::StringInterpolation(interp) => cat_args.push(self.convert_string_interpolation_to_perl(interp)),
                                _ => cat_args.push(self.word_to_perl(arg))
                            }
                        }
                        
                        if cat_args.is_empty() {
                            // No arguments - read from stdin (not implemented)
                            output.push_str(&format!("$output_{} = '';\n", pipeline_id));
                        } else {
                            // Read from file(s)
                            let file = &cat_args[0];
                            output.push_str(&format!("my $cat_content_{} = '';\n", pipeline_id));
                            output.push_str(&format!("if (open(my $fh_{}, '<', '{}')) {{\n", pipeline_id, file));
                            output.push_str(&format!("    while (my $line = <$fh_{}>) {{\n", pipeline_id));
                            output.push_str(&format!("        $cat_content_{} .= $line;\n", pipeline_id));
                            output.push_str("    }\n");
                            output.push_str(&format!("    close($fh_{});\n", pipeline_id));
                            output.push_str("} else {\n");
                            output.push_str(&format!("    warn \"cat: {}: No such file or directory\";\n", file));
                            output.push_str(&format!("    exit(1);\n"));
                            output.push_str("}\n");
                            output.push_str(&format!("$output_{} = $cat_content_{};\n", pipeline_id, pipeline_id));
                        }
                    } else {
                        // First command - capture output using system call
                        output.push_str(&format!("$output_{} = `{}`;\n", pipeline_id, self.command_to_string(&pipeline.commands[0])));
                    }
                } else {
                    // First command - capture output using system call
                    output.push_str(&format!("$output_{} = `{}`;\n", pipeline_id, self.command_to_string(&pipeline.commands[0])));
                }
            }
            
            // Handle subsequent commands
            for (_i, command) in pipeline.commands.iter().enumerate().skip(1) {

                if let Command::Simple(cmd) = command {
                    if cmd.name == "sort" {
                        // Handle sort command with flags
                        let mut sort_flags = String::new();
                        for arg in &cmd.args {
                            if let Word::Literal(lit) = arg {
                                sort_flags.push_str(lit);
                            }
                        }
                        
                        if sort_flags.contains('r') {
                            // Reverse sort
                            if sort_flags.contains('n') {
                                // Numeric reverse sort
                                output.push_str(&format!("$output_{} = join(\"\\n\", reverse(sort {{ $a <=> $b }} split(/\\n/, $output_{})));\n", pipeline_id, pipeline_id));
                            } else {
                                // String reverse sort
                                output.push_str(&format!("$output_{} = join(\"\\n\", reverse(sort(split(/\\n/, $output_{}))));\n", pipeline_id, pipeline_id));
                            }
                        } else if sort_flags.contains('n') {
                            // Numeric sort
                            output.push_str(&format!("$output_{} = join(\"\\n\", sort {{ $a <=> $b }} split(/\\n/, $output_{}));\n", pipeline_id, pipeline_id));
                        } else {
                            // Default string sort
                            output.push_str(&format!("$output_{} = join(\"\\n\", sort(split(/\\n/, $output_{})));\n", pipeline_id, pipeline_id));
                        }
                    } else if cmd.name == "uniq" {
                        // Handle uniq command with flags
                        let mut uniq_flags = String::new();
                        for arg in &cmd.args {
                            if let Word::Literal(lit) = arg {
                                uniq_flags.push_str(lit);
                            }
                        }
                        
                        if uniq_flags.contains('c') {
                            // Count occurrences
                            output.push_str(&format!("my %count_{};\n", pipeline_id));
                            output.push_str(&format!("for my $line (split(/\\n/, $output_{})) {{\n", pipeline_id));
                            output.push_str(&format!("    $count_{}{{$line}}++;\n", pipeline_id));
                            output.push_str("}\n");
                            output.push_str(&format!("my @uniq_result_{};\n", pipeline_id));
                            output.push_str(&format!("for my $key (keys %count_{}) {{\n", pipeline_id));
                            output.push_str(&format!("    my $count_val = $count_{}{{", pipeline_id));
                            output.push_str("$key");
                            output.push_str("};\n");
                            output.push_str(&format!("    my $count_str = \"$count_val $key\";\n"));
                            output.push_str(&format!("    push @uniq_result_{}, $count_str;\n", pipeline_id));
                            output.push_str("}\n");
                            output.push_str(&format!("$output_{} = join(\"\\n\", @uniq_result_{});\n", pipeline_id, pipeline_id));
                        } else {
                            // Default uniq behavior
                            output.push_str(&format!("my @lines_{} = split(/\\n/, $output_{});\n", pipeline_id, pipeline_id));
                            output.push_str(&format!("my @uniq_lines_{};\n", pipeline_id));
                            output.push_str(&format!("my $prev_{};\n", pipeline_id));
                            output.push_str(&format!("for my $line (@lines_{}) {{\n", pipeline_id));
                            output.push_str(&format!("    if (!defined($prev_{}) || $line ne $prev_{}) {{\n", pipeline_id, pipeline_id));
                            output.push_str(&format!("        push @uniq_lines_{}, $line;\n", pipeline_id));
                            output.push_str(&format!("        $prev_{} = $line;\n", pipeline_id));
                            output.push_str("    }\n");
                            output.push_str("}\n");
                            output.push_str(&format!("$output_{} = join(\"\\n\", @uniq_lines_{});\n", pipeline_id, pipeline_id));
                        }
                    } else if cmd.name == "wc" {
                        // Handle wc command with flags
                        let mut wc_flags = String::new();
                        for arg in &cmd.args {
                            if let Word::Literal(lit) = arg {
                                wc_flags.push_str(lit);
                            }
                        }
                        
                        if wc_flags.contains('l') {
                            // Count lines
                            output.push_str(&format!("$output_{} = scalar(split(/\\n/, $output_{}));\n", pipeline_id, pipeline_id));
                        } else if wc_flags.contains('w') {
                            // Count words
                            output.push_str(&format!("$output_{} = scalar(split(/\\s+/, $output_{}));\n", pipeline_id, pipeline_id));
                        } else if wc_flags.contains('c') {
                            // Count characters
                            output.push_str(&format!("$output_{} = length($output_{});\n", pipeline_id, pipeline_id));
                        } else {
                            // Default: count lines, words, characters
                            output.push_str(&format!("my $lines_{} = scalar(split(/\\n/, $output_{}));\n", pipeline_id, pipeline_id));
                            output.push_str(&format!("my $words_{} = scalar(split(/\\s+/, $output_{}));\n", pipeline_id, pipeline_id));
                            output.push_str(&format!("my $chars_{} = length($output_{});\n", pipeline_id, pipeline_id));
                            output.push_str(&format!("$output_{} = \"$lines_{} $words_{} $chars_{}\";\n", pipeline_id, pipeline_id, pipeline_id, pipeline_id));
                        }
                    } else if cmd.name == "grep" {
                        // Handle grep command
                        let pattern = if let Some(arg) = cmd.args.first() {
                            // Convert grep pattern to proper Perl regex
                            match arg {
                                Word::Literal(s) => {
                                                                            // Check if the pattern is already a regex pattern (contains regex metacharacters)
                                        // Also check for escaped backslashes (\\), which indicate regex patterns
                                        if s.contains('\\') || s.contains('^') || s.contains('$') || s.contains('[') || s.contains(']') || s.contains('(') || s.contains(')') || s.contains('|') || s.contains('+') || s.contains('*') || s.contains('?') {
                                            // Pattern is already a regex, but may need conversion from shell escape to Perl escape
                                            if s.contains('\\') {
                                                // Convert shell backslash escapes to Perl regex escapes
                                                s.replace("\\\\", "\\").replace("\\", "")
                                            } else {
                                                // Pattern is already a valid Perl regex
                                                s.clone()
                                            }
                                        } else {
                                            // Convert shell glob pattern to Perl regex
                                            self.convert_glob_to_regex(s)
                                        }
                                }
                                Word::StringInterpolation(interp) => {
                                    // Handle string interpolation in grep patterns
                                    if interp.parts.len() == 1 {
                                        if let StringPart::Literal(s) = &interp.parts[0] {
                                                                                    // Check if the pattern is already a regex pattern
                                        // Also check for escaped backslashes (\\), which indicate regex patterns
                                        if s.contains('\\') || s.contains('^') || s.contains('$') || s.contains('[') || s.contains(']') || s.contains('(') || s.contains(')') || s.contains('|') || s.contains('+') || s.contains('*') || s.contains('?') {
                                            // Pattern is already a regex, but may need conversion from shell escape to Perl escape
                                            if s.contains('\\') {
                                                // Convert shell backslash escapes to Perl regex escapes
                                                s.replace("\\\\", "\\").replace("\\", "")
                                            } else {
                                                // Pattern is already a valid Perl regex
                                                s.clone()
                                            }
                                        } else {
                                            // Convert shell glob pattern to Perl regex
                                            self.convert_glob_to_regex(s)
                                        }
                                        } else {
                                            // For other parts, use the converted string
                                            self.convert_string_interpolation_to_perl(interp)
                                        }
                                    } else {
                                        // For complex interpolations, reconstruct the full pattern and check if it's regex
                                        let full_pattern = self.convert_string_interpolation_to_perl(interp);
                                        // Check if the reconstructed pattern contains regex metacharacters
                                        if full_pattern.contains('\\') || full_pattern.contains('^') || full_pattern.contains('$') || full_pattern.contains('[') || full_pattern.contains(']') || full_pattern.contains('(') || full_pattern.contains(')') || full_pattern.contains('|') || full_pattern.contains('+') || full_pattern.contains('*') || full_pattern.contains('?') {
                                            // Pattern is already a regex, use as-is
                                            full_pattern
                                        } else {
                                            // Convert shell glob pattern to Perl regex
                                            self.convert_glob_to_regex(&full_pattern)
                                        }
                                    }
                                }
                                _ => self.word_to_perl(arg)
                            }
                        } else {
                            "".to_string()
                        };
                        output.push_str(&format!("my @grep_lines_{};\n", pipeline_id));
                        output.push_str(&format!("for my $line (split(/\\n/, $output_{})) {{\n", pipeline_id));
                        output.push_str(&format!("    if ($line =~ /{}/) {{\n", pattern));
                        output.push_str(&format!("        push @grep_lines_{}, $line;\n", pipeline_id));
                        output.push_str("    }\n");
                        output.push_str("}\n");
                        output.push_str(&format!("$output_{} = join(\"\\n\", @grep_lines_{});\n", pipeline_id, pipeline_id));
                    } else if cmd.name == "xargs" {
                        // Handle xargs command with cross-platform compatibility
                        if let Some(grep_cmd) = cmd.args.first() {
                            if grep_cmd.to_string() == "grep" {
                                // Handle xargs grep -l pattern
                                let pattern = if cmd.args.len() > 2 {
                                    match &cmd.args[2] {
                                        Word::Literal(s) => s.clone(),
                                        Word::StringInterpolation(interp) => {
                                            if interp.parts.len() == 1 {
                                                if let StringPart::Literal(s) = &interp.parts[0] {
                                                    s.clone()
                                                } else {
                                                    self.word_to_perl(&cmd.args[2])
                                                }
                                            } else {
                                                self.word_to_perl(&cmd.args[2])
                                            }
                                        }
                                        _ => self.word_to_perl(&cmd.args[2])
                                    }
                                } else {
                                    "".to_string()
                                };
                                
                                output.push_str(&format!("my @xargs_files_{};\n", pipeline_id));
                                output.push_str(&format!("for my $file (split(/\\n/, $output_{})) {{\n", pipeline_id));
                                output.push_str(&format!("    if ($file ne '') {{\n"));
                                output.push_str(&format!("        # Use Perl's built-in file reading instead of system grep for cross-platform compatibility\n"));
                                output.push_str(&format!("        my $found = 0;\n"));
                                output.push_str(&format!("        if (open(my $fh, '<', $file)) {{\n"));
                                output.push_str(&format!("            while (my $line = <$fh>) {{\n"));
                                output.push_str(&format!("                if ($line =~ /{}/) {{\n", pattern));
                                output.push_str(&format!("                    $found = 1;\n"));
                                output.push_str(&format!("                    last;\n"));
                                output.push_str(&format!("                }}\n"));
                                output.push_str(&format!("            }}\n"));
                                output.push_str(&format!("            close($fh);\n"));
                                output.push_str(&format!("        }}\n"));
                                output.push_str(&format!("        if ($found) {{\n"));
                                output.push_str(&format!("            push @xargs_files_{}, $file;\n", pipeline_id));
                                output.push_str(&format!("        }}\n"));
                                output.push_str(&format!("    }}\n"));
                                output.push_str(&format!("}}\n"));
                                output.push_str(&format!("$output_{} = join(\"\\n\", @xargs_files_{});\n", pipeline_id, pipeline_id));
                            } else {
                                // Generic xargs handling - fallback to system command
                                let cmd_str = self.command_to_string(command);
                                let escaped_cmd = cmd_str.replace("'", "'\"'\"'");
                                output.push_str(&format!("$output_{} = `echo \"$output_{}\" | {}`;\n", pipeline_id, pipeline_id, escaped_cmd));
                            }
                        } else {
                            // No arguments to xargs - fallback to system command
                            let cmd_str = self.command_to_string(command);
                            let escaped_cmd = cmd_str.replace("'", "'\"'\"'");
                            output.push_str(&format!("$output_{} = `echo \"$output_{}\" | {}`;\n", pipeline_id, pipeline_id, escaped_cmd));
                        }
                    } else if cmd.name == "find" {
                        // Handle find command with Windows compatibility
                        let mut find_args = Vec::new();
                        for arg in &cmd.args {
                            match arg {
                                Word::Literal(s) => {
                                    if s == "." {
                                        find_args.push(".".to_string());
                                    } else if s == "-name" {
                                        find_args.push("-name".to_string());
                                    } else {
                                        // Convert glob pattern to regex for find
                                        let pattern = self.convert_glob_to_regex(s);
                                        find_args.push(pattern);
                                    }
                                }
                                Word::StringInterpolation(interp) => {
                                    if interp.parts.len() == 1 {
                                        if let StringPart::Literal(s) = &interp.parts[0] {
                                            // Convert glob pattern to regex for find
                                            let pattern = self.convert_glob_to_regex(s);
                                            find_args.push(pattern);
                                        } else {
                                            find_args.push(self.convert_string_interpolation_to_perl(interp));
                                        }
                                    } else {
                                        find_args.push(self.convert_string_interpolation_to_perl(interp));
                                    }
                                }
                                _ => find_args.push(self.word_to_perl(arg))
                            }
                        }
                        
                        // Use Perl's File::Find instead of system find for cross-platform compatibility
                        if find_args.len() >= 3 && find_args[1] == "-name" {
                            let pattern = &find_args[2];
                            let dir = &find_args[0];
                            self.needs_file_find = true;
                            output.push_str(&format!("my @find_files_{};\n", pipeline_id));
                            // The pattern is already a regex from convert_glob_to_regex, no need to escape again
                            output.push_str(&format!("find({{wanted => sub {{ if ($_ =~ /{}/) {{ push @find_files_{}, $File::Find::name; }} }}, no_chdir => 1}}, '{}');\n", pattern, pipeline_id, dir));
                            output.push_str(&format!("$output_{} = join(\"\\n\", @find_files_{});\n", pipeline_id, pipeline_id));
                        } else {
                            // Fallback to system find command
                            let cmd_str = self.command_to_string(command);
                            let escaped_cmd = cmd_str.replace("'", "'\"'\"'");
                            output.push_str(&format!("$output_{} = `echo \"$output_{}\" | {}`;\n", pipeline_id, pipeline_id, escaped_cmd));
                        }
                    } else {
                        // Other commands - pipe through
                        let cmd_str = self.command_to_string(command);
                        let escaped_cmd = cmd_str.replace("'", "'\"'\"'");
                        output.push_str(&format!("$output_{} = `echo \"$output_{}\" | {}`;\n", pipeline_id, pipeline_id, escaped_cmd));
                    }
                } else {
                    // For non-simple commands, generate normally but capture output
                    output.push_str(&self.indent());
                    output.push_str(&format!("$output_{} = `echo \"$output_{}\" | {}`;\n", pipeline_id, pipeline_id, self.command_to_string(command)));
                }
            }
            output.push_str(&format!("print($output_{});\n", pipeline_id));
        } else {
            // Implement && and || via Perl boolean expressions
            // Assign the result to a variable to avoid "void context" warnings
            // Use a unique variable name for each pipeline to avoid redeclaration warnings
            self.pipeline_counter += 1;
            output.push_str(&format!("my $pipeline_result_{} = ", self.pipeline_counter));
            if let Some(first) = pipeline.commands.first() {
                match first {
                    Command::TestExpression(test_expr) => {
                        // Generate the test expression directly as a Perl boolean expression
                        // Wrap in parentheses to ensure proper context
                        output.push_str(&format!("({})", self.generate_test_expression(test_expr)));
                    }
                    _ => {
                        // For non-test expressions, use system() calls
                        output.push_str(&format!("system('{}') == 0", self.command_to_string(first)));
                    }
                }
            }
            for (idx, op) in pipeline.operators.iter().enumerate() {
                let cmd = &pipeline.commands[idx + 1];
                match (op, cmd) {
                    (PipeOperator::And, Command::TestExpression(test_expr)) => {
                        output.push_str(" && ");
                        // Wrap test expressions in parentheses to ensure proper context
                        output.push_str(&format!("({})", self.generate_test_expression(test_expr)));
                    }
                    (PipeOperator::Or, Command::TestExpression(test_expr)) => {
                        output.push_str(" || ");
                        output.push_str(&format!("({})", self.generate_test_expression(test_expr)));
                    }
                    (PipeOperator::And, _) => {
                        output.push_str(" && ");
                        output.push_str(&format!("system('{}') == 0", self.command_to_string(cmd)));
                    }
                    (PipeOperator::Or, _) => {
                        output.push_str(" || ");
                        output.push_str(&format!("system('{}') == 0", self.command_to_string(cmd)));
                    }
                    (PipeOperator::Pipe, _) => {}
                }
            }
            // Add semicolon and newline after the pipeline
            output.push_str(";\n");
        }
        
        output
    }

    fn command_to_string(&mut self, command: &Command) -> String {
        match command {
            Command::Simple(cmd) => {
                if cmd.args.is_empty() {
                    cmd.name.to_string()
                } else {
                    let args = cmd.args.iter().map(|arg| arg.to_string()).collect::<Vec<_>>().join(" ");
                    format!("{} {}", cmd.name, args)
                }
            }
            Command::TestExpression(test_expr) => {
                // Convert test expression to Perl test
                self.generate_test_expression(test_expr)
            }

            _ => "command".to_string(),
        }
    }

    fn generate_if_statement(&mut self, if_stmt: &IfStatement) -> String {
        let mut output = String::new();
        
        // Generate condition
        output.push_str("if (");
        match &*if_stmt.condition {
            Command::Simple(cmd) if cmd.name == "[" || cmd.name == "test" => {
                self.generate_test_command(cmd, &mut output);
            }
            _ => {
                output.push_str(&self.generate_command(&if_stmt.condition));
            }
        }
        output.push_str(") {\n");
        
        // Generate then branch
        self.indent_level += 1;
        output.push_str(&self.indent());
        output.push_str(&self.generate_command(&if_stmt.then_branch));
        self.indent_level -= 1;
        
        // Generate else branch if present
        if let Some(else_branch) = &if_stmt.else_branch {
            output.push_str(&self.indent());
            output.push_str("} else {\n");
            self.indent_level += 1;
            output.push_str(&self.indent());
            output.push_str(&self.generate_command(else_branch));
            self.indent_level -= 1;
        }
        
        output.push_str(&self.indent());
        output.push_str("}\n");
        
        output
    }

    fn generate_while_loop(&mut self, while_loop: &WhileLoop) -> String {
        let mut output = String::new();
        
        // Handle different types of conditions
        match &*while_loop.condition {
            Command::Simple(cmd) if cmd.name == "[" || cmd.name == "test" => {
                // For test commands, generate a simple while loop
                // Initialize any variables used in test conditions
                if cmd.args.len() >= 3 {
                    // Check both operands for variables that need initialization
                    let operand1 = &cmd.args[0];
                    let operand2 = &cmd.args[2];
                    
                    // Initialize first operand if it's a variable
                    if let Word::Variable(var_name) = operand1 {
                        if !self.declared_locals.contains(var_name) {
                            // Check if this variable was used in a previous for loop
                            if var_name == "i" {
                                output.push_str(&format!("my ${} = 5;\n", var_name));
                            } else {
                                output.push_str(&format!("my ${} = 0;\n", var_name));
                            }
                            self.declared_locals.insert(var_name.to_string());
                        }
                    }
                    
                    // Initialize second operand if it's a variable
                    if let Word::Variable(var_name) = operand2 {
                        if !self.declared_locals.contains(var_name) {
                            output.push_str(&format!("my ${} = 0;\n", var_name));
                            self.declared_locals.insert(var_name.to_string());
                        }
                    }
                } else if cmd.args.len() >= 1 {
                    // Handle single argument test conditions
                    let var_name = cmd.args[0].trim_start_matches('$');
                    if !self.declared_locals.contains(var_name) {
                        output.push_str(&format!("my ${} = 0;\n", var_name));
                        self.declared_locals.insert(var_name.to_string());
                    }
                }
                output.push_str("while (");
                self.generate_test_command(cmd, &mut output);
                output.push_str(") {\n");
            }
            _ => {
                // For other command types, generate a complex while loop with exit status check
                output.push_str("while (1) {\n");
                output.push_str(&self.indent());
                output.push_str("my $condition = ");
                output.push_str("system(");
                output.push_str(&self.generate_command(&while_loop.condition));
                output.push_str(") == 0");
                output.push_str(";\n");
                output.push_str(&self.indent());
                output.push_str("last unless $condition;\n");
            }
        }
        
        self.indent_level += 1;
        
        // Generate body commands
        for command in &while_loop.body.commands {
            output.push_str(&self.indent());
            output.push_str(&self.generate_command(command));
        }
        
        self.indent_level -= 1;
        output.push_str("}\n");
        
        output
    }

    fn find_for_loop_variable(&self, command: &Command) -> Option<String> {
        match command {
            Command::For(for_loop) => Some(for_loop.variable.clone()),
            Command::Block(block) => {
                for cmd in &block.commands {
                    if let Some(var) = self.find_for_loop_variable(cmd) {
                        return Some(var);
                    }
                }
                None
            }
            _ => None
        }
    }

    fn generate_for_loop(&mut self, for_loop: &ForLoop) -> String {
        let variable = &for_loop.variable;
        let items = &for_loop.items;
        let body = &for_loop.body;
        
        // Special case for iterating over arguments ($@)
        if items.len() == 1 {
            let item = &items[0];
            if matches!(item, Word::Variable(var) if var == "@") {
                self.indent_level += 1;
                let body_code = self.generate_block(body);
                self.indent_level -= 1;
                return format!("for my ${} (@ARGV) {{\n{}}}\n", variable, body_code);
            } else if let Word::StringInterpolation(interp) = item {
                if interp.parts.len() == 1 {
                    if let StringPart::Variable(var) = &interp.parts[0] {
                        if var == "@" {
                            self.indent_level += 1;
                            let body_code = self.generate_block(body);
                            self.indent_level -= 1;
                            return format!("for my ${} (@ARGV) {{\n{}}}\n", variable, body_code);
                        }
                    }
                }
            }
        }
        
        // Convert shell brace expansion to Perl range syntax
        let items_str = if items.len() == 1 {
            match &items[0] {
                Word::BraceExpansion(expansion) => {
                    // Handle brace expansion items
                    if expansion.items.len() == 1 {
                        match &expansion.items[0] {
                            BraceItem::Range(range) => {
                                // Convert {1..5} to 1..5
                                format!("{}..{}", range.start, range.end)
                            }
                            BraceItem::Literal(s) => {
                                // Single literal item
                                format!("\"{}\"", s)
                            }
                            BraceItem::Sequence(seq) => {
                                // Convert {a,b,c} to ("a", "b", "c")
                                format!("({})", seq.iter().map(|s| format!("\"{}\"", s)).collect::<Vec<_>>().join(", "))
                            }
                        }
                    } else {
                        // Multiple items
                        let parts: Vec<String> = expansion.items.iter().map(|item| {
                            match item {
                                BraceItem::Literal(s) => format!("\"{}\"", s),
                                BraceItem::Range(range) => format!("{}..{}", range.start, range.end),
                                BraceItem::Sequence(seq) => format!("({})", seq.iter().map(|s| format!("\"{}\"", s)).collect::<Vec<_>>().join(", ")),
                            }
                        }).collect();
                        format!("({})", parts.join(", "))
                    }
                }
                Word::Literal(s) if s.starts_with('{') && s.ends_with('}') => {
                    // Fallback for literal strings that look like brace expansions
                    let content = &s[1..s.len()-1];
                    if content.contains("..") {
                        // Already in range format like {1..5}
                        content.to_string()
                    } else {
                        // Convert {a,b,c} to ("a", "b", "c")
                        let parts: Vec<&str> = content.split(',').collect();
                        if parts.len() > 1 {
                            format!("({})", parts.iter().map(|s| format!("\"{}\"", s)).collect::<Vec<_>>().join(", "))
                        } else {
                            content.to_string()
                        }
                    }
                }
                Word::StringInterpolation(interp) => {
                    // Handle string interpolation specially for for loops
                    if interp.parts.len() == 1 {
                        if let StringPart::Variable(var) = &interp.parts[0] {
                            if var.starts_with('!') && var.ends_with("[@]") {
                                // This is !map[@] - convert to keys(%map) without quotes
                                let array_name = &var[1..var.len()-3]; // Remove ! prefix and [@] suffix
                                format!("keys(%{})", array_name)
                            } else if var.starts_with('#') && var.contains('[') {
                                // This is #arr[@] - convert to scalar(@arr) without quotes
                                if let Some(bracket_start) = var.find('[') {
                                    let array_name = &var[1..bracket_start];
                                    format!("scalar(@{})", array_name)
                                } else {
                                    format!("${}", var)
                                }
                            } else if var.ends_with("[@]") {
                                // This is arr[@] - convert to @arr without quotes
                                let array_name = &var[..var.len()-3];
                                format!("@{}", array_name)
                            } else if var.contains('[') && var.ends_with(']') {
                                // This is arr[1] - convert to $arr[1] without quotes
                                if let Some(bracket_start) = var.find('[') {
                                    let array_name = &var[..bracket_start];
                                    let key = &var[bracket_start..];
                                    format!("${}{}", array_name, key)
                                } else {
                                    format!("${}", var)
                                }
                            } else {
                                // Regular variable - wrap in quotes
                                format!("\"${}\"", var)
                            }
                        } else if let StringPart::MapAccess(map_name, key) = &interp.parts[0] {
                            // Handle MapAccess specially for for loops
                            if key == "@" {
                                // This is arr[@] - convert to @arr without quotes
                                format!("@{}", map_name)
                            } else if key.starts_with('#') && key.contains('[') {
                                // This is #arr[@] - convert to scalar(@arr) without quotes
                                if let Some(bracket_start) = key.find('[') {
                                    let array_name = &key[1..bracket_start];
                                    format!("scalar(@{})", array_name)
                                } else {
                                    format!("${}{}", map_name, key)
                                }
                            } else if key.starts_with('!') && key.ends_with("[@]") {
                                // This is !map[@] - convert to keys(%map) without quotes
                                let array_name = &key[1..key.len()-3]; // Remove ! prefix and [@] suffix
                                format!("keys(%{})", array_name)
                            } else {
                                // Regular map access - wrap in quotes
                                format!("\"${}{}\"", map_name, key)
                            }
                        } else if let StringPart::MapKeys(map_name) = &interp.parts[0] {
                            // This is ${!map[@]} - convert to keys(%map) without quotes
                            format!("keys(%{})", map_name)
                        } else {
                            // Other parts - wrap in quotes
                            format!("\"{}\"", items[0])
                        }
                    } else {
                        // Multiple parts - wrap in quotes
                        format!("\"{}\"", items[0])
                    }
                }
                Word::MapAccess(map_name, key) => {
                    // Handle map access specially for for loops
                    if key == "@" {
                        // This is arr[@] - convert to @arr without quotes
                        format!("@{}", map_name)
                    } else if key.starts_with('#') && key.contains('[') {
                        // This is #arr[@] - convert to scalar(@arr) without quotes
                        if let Some(bracket_start) = key.find('[') {
                            let array_name = &key[1..bracket_start];
                            format!("scalar(@{})", array_name)
                        } else {
                            format!("${}{}", map_name, key)
                        }
                    } else if key.starts_with('!') && key.ends_with("[@]") {
                        // This is !map[@] - convert to keys(%map) without quotes
                        let array_name = &key[1..key.len()-3]; // Remove ! prefix and [@] suffix
                        format!("keys(%{})", array_name)
                    } else {
                        // Regular map access - wrap in quotes
                        format!("\"${}{}\"", map_name, key)
                    }
                }
                Word::MapKeys(map_name) => {
                    // This is !map[@] - convert to keys(%map) without quotes
                    format!("keys(%{})", map_name)
                }
                Word::MapLength(map_name) => {
                    // This is #arr[@] - convert to scalar(@arr) without quotes
                    format!("scalar(@{})", map_name)
                }
                _ => {
                    // Other word types - use proper word conversion
                    // Special handling for !map[@] variables
                    if let Word::Variable(var) = &items[0] {
                        if var.starts_with('!') && var.ends_with("[@]") {
                            // This is !map[@] - convert to keys(%map) without quotes
                            let array_name = &var[1..var.len()-3]; // Remove ! prefix and [@] suffix
                            format!("keys(%{})", array_name)
                        } else {
                            format!("\"{}\"", self.word_to_perl(&items[0]))
                        }
                    } else {
                        format!("\"{}\"", self.word_to_perl(&items[0]))
                    }
                }
            }
        } else if items.is_empty() {
            // No items specified, use default behavior
            "()".to_string()
        } else {
            // Multiple items
            format!("({})", items.iter().map(|s| format!("\"{}\"", self.word_to_perl(s))).collect::<Vec<_>>().join(", "))
        };
        
        self.indent_level += 1;
        let body_code = self.generate_block(body);
        self.indent_level -= 1;
        
        format!("for my ${} ({}) {{\n{}}}\n", variable, items_str, body_code)
    }

    fn parse_numeric_brace_range(&self, s: &str) -> Option<(i64, i64)> {
        SharedUtils::parse_numeric_brace_range(s)
    }

    fn parse_seq_command(&self, s: &str) -> Option<(i64, i64)> {
        SharedUtils::parse_seq_command(s)
    }

    fn expand_brace_expression(&self, expr: &str) -> String {
        // Handle simple numeric ranges like {1..5}
        if let Some(range) = self.parse_numeric_brace_range(expr) {
            let (start, end) = range;
            let values: Vec<String> = (start..=end).map(|i| i.to_string()).collect();
            return format!("({})", values.join(", "));
        }
        
        // Handle character ranges like {a..c}
        if expr.contains("..") {
            let parts: Vec<&str> = expr.split("..").collect();
            if parts.len() == 2 {
                if let (Some(start_char), Some(end_char)) = (parts[0].chars().next(), parts[1].chars().next()) {
                                            if start_char.is_ascii_lowercase() && end_char.is_ascii_lowercase() {
                            let start = start_char as u8;
                            let end = end_char as u8;
                            if start <= end {
                                let values: Vec<String> = (start..=end)
                                    .map(|c| format!("'{}'", char::from(c)))
                                    .collect();
                                return format!("({})", values.join(", "));
                            }
                        }
                }
            }
        }
        
        // Handle step ranges like {00..04..2}
        if expr.matches("..").count() == 2 {
            let parts: Vec<&str> = expr.split("..").collect();
            if parts.len() == 3 {
                if let (Ok(start), Ok(end), Ok(step)) = (parts[0].parse::<i64>(), parts[2].parse::<i64>(), parts[1].parse::<i64>()) {
                    let mut values = Vec::new();
                    let mut current = start;
                    while current <= end {
                        values.push(current.to_string());
                        current += step;
                    }
                    return format!("({})", values.join(", "));
                }
            }
        }
        
        // If no expansion possible, return as literal
        format!("'{}'", expr)
    }

    fn generate_function(&mut self, func: &Function) -> String {
        let mut output = String::new();
        
        // Track that this function is defined
        self.declared_functions.insert(func.name.clone());
        
        output.push_str(&format!("sub {} {{\n", func.name));
        self.indent_level += 1;
        
        // Generate body commands
        for command in &func.body.commands {
            output.push_str(&self.indent());
            output.push_str(&self.generate_command(command));
        }
        
        self.indent_level -= 1;
        output.push_str("}\n");
        
        output
    }

    fn generate_subshell(&mut self, command: &Command) -> String {
        let mut output = String::new();
        
        output.push_str("do {\n");
        self.indent_level += 1;
        self.subshell_depth += 1;
        output.push_str(&self.indent());
        output.push_str(&self.generate_command(command));
        if self.subshell_depth > 0 { self.subshell_depth -= 1; }
        self.indent_level -= 1;
        output.push_str("};\n");
        
        output
    }

    fn generate_background(&mut self, command: &Command) -> String {
        let mut output = String::new();
        // Use threads to emulate background
        output.push_str("use threads;\n");
        output.push_str("threads->create(sub {\n");
        self.indent_level += 1;
        output.push_str(&self.indent());
        output.push_str(&self.generate_command(command));
        self.indent_level -= 1;
        output.push_str("});\n");
        output
    }

    fn generate_block(&mut self, block: &Block) -> String {
        let mut output = String::new();
        for cmd in &block.commands {
            output.push_str(&self.indent());
            output.push_str(&self.generate_command(cmd));
        }
        output
    }

    fn indent(&self) -> String {
        SharedUtils::indent(self.indent_level)
    }
    
    fn escape_perl_string(&self, s: &str) -> String {
        SharedUtils::escape_string_for_language(s, "perl")
    }

    fn escape_perl_regex(&self, s: &str) -> String {
        s.chars().map(|c| match c {
            '\\' => "\\\\".to_string(),
            '/' => "\\/".to_string(),
            '^' => "\\^".to_string(),
            '$' => "\\$".to_string(),
            '.' => "\\.".to_string(),
            '*' => "\\*".to_string(),
            '+' => "\\+".to_string(),
            '?' => "\\?".to_string(),
            '(' => "\\(".to_string(),
            ')' => "\\)".to_string(),
            '[' => "\\[".to_string(),
            ']' => "\\]".to_string(),
            '{' => "\\{".to_string(),
            '}' => "\\}".to_string(),
            '|' => "\\|".to_string(),
            _ => c.to_string()
        }).collect()
    }

    fn perl_string_literal(&self, s: &str) -> String {
        // Handle strings that already contain escape sequences
        let mut result = String::new();
        
        for ch in s.chars() {
            match ch {
                '\n' => result.push_str("\\n"),
                '\t' => result.push_str("\\t"),
                '\r' => result.push_str("\\r"),
                '\x07' => result.push_str("\\a"),  // bell
                '\x08' => result.push_str("\\b"),  // backspace
                '\x0c' => result.push_str("\\f"),  // formfeed
                '\x0b' => result.push_str("\\v"),  // vertical tab
                '\\' => result.push_str("\\\\"),
                '"' => result.push_str("\\\""),
                '\'' => result.push_str("\\'"),
                _ => result.push(ch),
            }
        }
        
        // Format the result as a Perl string literal
        format!("\"{}\"", result)
    }

    fn convert_arithmetic_to_perl(&self, expr: &str) -> String {
        SharedUtils::convert_arithmetic_operators(expr, "perl")
    }

    fn convert_string_interpolation_to_perl_for_printf(&self, interp: &StringInterpolation) -> String {
        let mut result = String::new();
        
        for part in &interp.parts {
            match part {
                StringPart::Literal(s) => {
                    // Check if this literal contains array references like {map[foo]}
                    if s.starts_with('{') && s.ends_with('}') && s.contains('[') {
                        // This might be an array reference like {map[foo]}
                        let content = &s[1..s.len()-1]; // Remove { and }
                        if content.contains('[') && content.ends_with(']') {
                            if let Some(bracket_start) = content.find('[') {
                                let array_name = &content[..bracket_start];
                                let key = &content[bracket_start..];
                                // For associative arrays, use {} instead of []
                                if array_name == "map" {
                                    result.push_str(&format!("$map{{{}}}", &key[1..key.len()-1]));
                                } else {
                                    result.push_str(&format!("${}{}", array_name, key));
                                }
                                continue;
                            }
                        }
                    }
                    result.push_str(&self.escape_perl_string(s));
                }
                StringPart::MapAccess(map_name, key) => {
                    // Convert map access to Perl array/hash access
                    // For now, assume "map" is a hash and others are indexed arrays
                    if map_name == "map" {
                        // Convert map[key] to $map{key} for associative arrays
                        result.push_str(&format!("${}{{{}}}", map_name, key));
                    } else {
                        // Convert arr[key] to $arr[key] for indexed arrays
                        result.push_str(&format!("${}[{}]", map_name, key));
                    }
                }
                StringPart::MapKeys(map_name) => {
                    // Convert ${!map[@]} to keys(%map) for printf format strings
                    result.push_str(&format!("keys(%{})", map_name));
                }
                StringPart::MapLength(map_name) => {
                    // Convert ${#arr[@]} to scalar(@arr) for printf format strings
                    result.push_str(&format!("scalar(@{})", map_name));
                }
                StringPart::ParameterExpansion(pe) => {
                    result.push_str(&self.generate_parameter_expansion(pe));
                }
                StringPart::Variable(var) => {
                    // Convert shell variables to Perl variables
                    // For printf format strings, preserve array length expressions
                    if var == "#" {
                        result.push_str("scalar(@ARGV)");
                    } else if var == "@" {
                        result.push_str("join(\" \", @ARGV)");
                    } else if var == "1" {
                        result.push_str("$_[0]");
                    } else if var == "2" {
                        result.push_str("$_[1]");
                    } else if var == "3" {
                        result.push_str("$_[2]");
                    } else if var == "4" {
                        result.push_str("$_[3]");
                    } else if var == "5" {
                        result.push_str("$_[4]");
                    } else if var == "6" {
                        result.push_str("$_[5]");
                    } else if var == "7" {
                        result.push_str("$_[6]");
                    } else if var == "8" {
                        result.push_str("$_[7]");
                    } else if var == "9" {
                        result.push_str("$_[8]");
                    } else {
                        // Check for special shell array syntax
                        if var.starts_with('#') && var.contains('[') {
                            // This is #arr[@] - preserve as ${#arr[@]} for printf format strings
                            result.push_str(&format!("${{{}}}", var));
                        } else if var.starts_with('!') && var.ends_with("[@]") {
                            // This is !map[@] - preserve as ${!map[@]} for printf format strings
                            result.push_str(&format!("${{{}}}", var));
                        } else if var.starts_with('!') && var.contains('[') {
                            // This is !map[key] - preserve as ${!map[key]} for printf format strings
                            result.push_str(&format!("${{{}}}", var));
                        } else if var.starts_with('!') {
                            // This is !map - preserve as ${!map} for printf format strings
                            result.push_str(&format!("${{{}}}", var));
                        } else if var.ends_with("[@]") {
                            // This is arr[@] - preserve as ${arr[@]} for printf format strings
                            result.push_str(&format!("${{{}}}", var));
                        } else if var.contains('[') && var.ends_with(']') {
                            // This is arr[1] - preserve as ${arr[1]} for printf format strings
                            result.push_str(&format!("${{{}}}", var));
                        } else {
                            // For simple variable names, use ${var} to preserve shell syntax
                            result.push_str(&format!("${{{}}}", var));
                        }
                    }
                }
                StringPart::Arithmetic(arith) => {
                    // Convert shell arithmetic to Perl
                    let expr = self.convert_arithmetic_to_perl(&arith.expression);
                    result.push_str(&expr);
                }
                StringPart::CommandSubstitution(_) => {
                    // TODO: implement command substitution
                    result.push_str("''");
                }
            }
        }
        
        result
    }

    fn convert_string_interpolation_to_perl(&self, interp: &StringInterpolation) -> String {
        let mut result = String::new();
        
        // Special case: if we have only one part and it's a special variable that should be evaluated
        if interp.parts.len() == 1 {
            if let StringPart::Variable(var) = &interp.parts[0] {
                if var.starts_with('#') && var.ends_with("[@]") {
                    // This is #arr[@] - convert to scalar(@arr) in Perl without quotes
                    let array_name = &var[1..var.len()-3]; // Remove # prefix and [@] suffix
                    return format!("scalar(@{})", array_name);
                } else if var.starts_with('#') && var.ends_with("[*]") {
                    // This is #arr[*] - convert to scalar(@arr) in Perl without quotes
                    let array_name = &var[1..var.len()-3]; // Remove # prefix and [*] suffix
                    return format!("scalar(@{})", array_name);
                } else if var.starts_with('!') && var.ends_with("[@]") {
                    // This is !map[@] - convert to keys(%map) in Perl without quotes
                    let array_name = &var[1..var.len()-3]; // Remove ! prefix and [@] suffix
                    return format!("keys(%{})", array_name);
                } else if var.starts_with('!') && var.ends_with("[*]") {
                    // This is !map[*] - convert to keys(%map) in Perl without quotes
                    let array_name = &var[1..var.len()-3]; // Remove ! prefix and [*] suffix
                    return format!("keys(%{})", array_name);
                }
            }
            
            // Special case: if we have only one part and it's parameter expansion, return it without quotes
            if let StringPart::ParameterExpansion(pe) = &interp.parts[0] {
                return self.generate_parameter_expansion(pe);
            }
        }
        
        for part in &interp.parts {
            match part {
                StringPart::Literal(s) => {
                    // Check if this literal contains array references like {map[foo]}
                    if s.starts_with('{') && s.ends_with('}') && s.contains('[') {
                        // This might be an array reference like {map[foo]}
                        let content = &s[1..s.len()-1]; // Remove { and }
                        if content.contains('[') && content.ends_with(']') {
                            if let Some(bracket_start) = content.find('[') {
                                let array_name = &content[..bracket_start];
                                let key = &content[bracket_start..];
                                // For associative arrays, use {} instead of []
                                if array_name == "map" {
                                    result.push_str(&format!("$map{{{}}}", &key[1..key.len()-1]));
                                } else {
                                    result.push_str(&format!("${}{}", array_name, key));
                                }
                                continue;
                            }
                        }
                    }
                    result.push_str(&self.escape_perl_string(s));
                }
                StringPart::MapAccess(map_name, key) => {
                    // Convert map access to Perl array/hash access
                    debug_eprintln!("DEBUG: Processing MapAccess: map_name='{}', key='{}'", map_name, key);
                    // For now, assume "map" is a hash and others are indexed arrays
                    if map_name == "map" {
                        // Convert map[key] to $map{key} for associative arrays
                        debug_eprintln!("DEBUG: Converting map access to hash access: $map{{{}}}", key);
                        result.push_str(&format!("${}{{{}}}", map_name, key));
                    } else {
                        // Convert arr[key] to $arr[key] for indexed arrays
                        debug_eprintln!("DEBUG: Converting array access to indexed access: ${}[{}]", map_name, key);
                        result.push_str(&format!("${}[{}]", map_name, key));
                    }
                }
                StringPart::MapKeys(map_name) => {
                    // Convert ${!map[@]} to keys(%map) in Perl
                    debug_eprintln!("DEBUG: Processing MapKeys: map_name='{}'", map_name);
                    result.push_str(&format!("keys(%{})", map_name));
                }
                StringPart::MapLength(map_name) => {
                    // Convert ${#arr[@]} to scalar(@arr) in Perl
                    debug_eprintln!("DEBUG: Processing MapLength: map_name='{}'", map_name);
                    result.push_str(&format!("scalar(@{})", map_name));
                }
                StringPart::ParameterExpansion(pe) => {
                    result.push_str(&self.generate_parameter_expansion(pe));
                }
                StringPart::Variable(var) => {
                    // Convert shell variables to Perl variables
                            debug_eprintln!("DEBUG: Processing variable: '{}'", var);
        debug_eprintln!("DEBUG: Variable starts with '!': {}", var.starts_with('!'));
        debug_eprintln!("DEBUG: Variable ends with '[@]': {}", var.ends_with("[@]"));
        debug_eprintln!("DEBUG: Variable contains '[': {}", var.contains('['));
        debug_eprintln!("DEBUG: Variable length: {}", var.len());
        debug_eprintln!("DEBUG: Variable bytes: {:?}", var.as_bytes());
                    if var == "#" {
                        result.push_str("scalar(@ARGV)");
                    } else if var == "@" {
                        result.push_str("join(\" \", @ARGV)");
                    } else if var == "1" {
                        result.push_str("$_[0]");
                    } else if var == "2" {
                        result.push_str("$_[1]");
                    } else if var == "3" {
                        result.push_str("$_[2]");
                    } else if var == "4" {
                        result.push_str("$_[3]");
                    } else if var == "5" {
                        result.push_str("$_[4]");
                    } else if var == "6" {
                        result.push_str("$_[5]");
                    } else if var == "7" {
                        result.push_str("$_[6]");
                    } else if var == "8" {
                        result.push_str("$_[7]");
                    } else if var == "9" {
                        result.push_str("$_[8]");
                    } else {
                        // Check for special shell array syntax
                        if var.starts_with('#') && var.ends_with("[@]") {
                            // This is #arr[@] - convert to scalar(@arr) in Perl
                            let array_name = &var[1..var.len()-3]; // Remove # prefix and [@] suffix
                            // For scalar functions, we need to ensure they're evaluated, not treated as strings
                            result.push_str(&format!("scalar(@{})", array_name));
                        } else if var.starts_with('#') && var.ends_with("[*]") {
                            // This is #arr[*] - convert to scalar(@arr) in Perl
                            let array_name = &var[1..var.len()-3]; // Remove # prefix and [*] suffix
                            result.push_str(&format!("scalar(@{})", array_name));
                        } else if var.starts_with('!') && var.ends_with("[@]") {
                            // This is !map[@] - convert to keys(%map) in Perl
                            debug_eprintln!("DEBUG: Found !map[@] pattern, converting to keys(%map)");
                            let array_name = &var[1..var.len()-3]; // Remove ! prefix and [@] suffix
                            debug_eprintln!("DEBUG: Array name extracted: '{}'", array_name);
                            result.push_str(&format!("keys(%{})", array_name));
                        } else if var.starts_with('!') && var.contains('[') {
                            // This is !map[key] - convert to keys(%map) in Perl (more general pattern)
                            debug_eprintln!("DEBUG: Found !map[key] pattern, converting to keys(%map)");
                            if let Some(bracket_start) = var.find('[') {
                                let array_name = &var[1..bracket_start]; // Remove ! prefix
                                debug_eprintln!("DEBUG: Array name extracted from !map[key]: '{}'", array_name);
                                result.push_str(&format!("keys(%{})", array_name));
                            } else {
                                result.push_str(&format!("${}", var));
                            }
                        } else if var.starts_with('!') {
                            // This is !map - convert to keys(%map) in Perl (fallback)
                            debug_eprintln!("DEBUG: Found !map pattern (fallback), converting to keys(%map)");
                            let array_name = &var[1..]; // Remove ! prefix
                            debug_eprintln!("DEBUG: Array name extracted from !map: '{}'", array_name);
                            result.push_str(&format!("keys(%{})", array_name));
                        } else if var.ends_with("[@]") {
                            // This is arr[@] - convert to @arr in Perl
                            debug_eprintln!("DEBUG: Found arr[@] pattern, converting to @arr");
                            let array_name = &var[..var.len()-3]; // Remove [@] suffix
                            debug_eprintln!("DEBUG: Array name extracted from arr[@]: '{}'", array_name);
                            result.push_str(&format!("@{}", array_name));
                        } else if var.contains('[') && var.ends_with(']') {
                            // This is arr[1] - convert to $arr[1] in Perl or $arr{key} for hashes
                            debug_eprintln!("DEBUG: Found arr[1] pattern, converting to array access");
                            if let Some(bracket_start) = var.find('[') {
                                let array_name = &var[..bracket_start];
                                let key = &var[bracket_start..];
                                // Check if this is a hash (associative array) - for now, assume map is a hash
                                if array_name == "map" {
                                    // Convert map[key] to $map{key} for associative arrays
                                    let key_content = &key[1..key.len()-1]; // Remove [ and ]
                                    debug_eprintln!("DEBUG: Converting map[key] to $map{{{}}}", key_content);
                                    result.push_str(&format!("${}{{{}}}", array_name, key_content));
                                } else {
                                    // Regular indexed array - use [] syntax in Perl
                                    debug_eprintln!("DEBUG: Converting arr[key] to ${}[{}]", array_name, key);
                                    result.push_str(&format!("${}[{}]", array_name, key));
                                }
                            } else {
                                result.push_str(&format!("${}", var));
                            }
                        } else {
                            // For simple variable names, use $var instead of ${var}
                            debug_eprintln!("DEBUG: Simple variable, using ${}", var);
                            result.push_str(&format!("${}", var));
                        }
                    }
                }
                StringPart::Arithmetic(arith) => {
                    // Convert shell arithmetic to Perl
                    let expr = self.convert_arithmetic_to_perl(&arith.expression);
                    result.push_str(&expr);
                }
                StringPart::CommandSubstitution(_) => {
                    // TODO: implement command substitution
                    result.push_str("''");
                }
            }
        }
        
        result
    }

    fn word_to_perl(&self, word: &Word) -> String {
        match word {
            Word::Literal(s) => self.escape_perl_string(s),
            Word::Array(name, elements) => {
                // Convert array declaration to Perl array
                let elements_str = elements.iter()
                    .map(|e| format!("'{}'", e.replace("'", "\\'")))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("@{} = ({});", name, elements_str)
            },
            Word::ParameterExpansion(pe) => self.generate_parameter_expansion(pe),
            Word::Variable(var) => {
                // Handle parameter expansion operators
                if var.contains("^^") {
                    // ${var^^} -> uc($var) - uppercase all characters
                    let var_name = var.replace("^^", "");
                    format!("uc(${})", var_name)
                } else if var.contains(",,)") {
                    // ${var,,} -> lc($var) - lowercase all characters
                    let var_name = var.replace(",,)", "");
                    format!("lc(${})", var_name)
                } else if var.contains("^)") {
                    // ${var^} -> ucfirst($var) - uppercase first character
                    let var_name = var.replace("^)", "");
                    format!("ucfirst(${})", var_name)
                } else if var.ends_with("##*/") {
                    // ${var##*/} -> basename($var) - remove longest prefix matching */
                    let var_name = var.replace("##*/", "");
                    format!("basename(${})", var_name)
                } else if var.ends_with("%/*") {
                    // ${var%/*} -> dirname($var) - remove shortest suffix matching /*
                    let var_name = var.replace("%/*", "");
                    format!("dirname(${})", var_name)
                } else if var.contains("//") {
                    // ${var//pattern/replacement} -> $var =~ s/pattern/replacement/g
                    let parts: Vec<&str> = var.split("//").collect();
                    if parts.len() >= 2 {
                        let var_name = parts[0];
                        if parts.len() >= 3 {
                            let pattern = parts[1];
                            let replacement = parts[2];
                            format!("${} =~ s/{}/{}/g", var_name, pattern, replacement)
                        } else {
                            // Only 2 parts: ${var//pattern} -> $var =~ s/pattern//g
                            let pattern = parts[1];
                            format!("${} =~ s/{}/g", var_name, pattern)
                        }
                    } else {
                        format!("${}", var)
                    }
                } else if var.contains("#") && !var.starts_with('#') {
                    // ${var#pattern} -> $var =~ s/^pattern// - remove shortest prefix
                    let parts: Vec<&str> = var.split("#").collect();
                    if parts.len() == 2 {
                        let var_name = parts[0];
                        let pattern = parts[1];
                        format!("${} =~ s/^{}//", var_name, pattern)
                    } else {
                        format!("${}", var)
                    }
                } else if var.contains("%") && !var.starts_with('%') {
                    // ${var%pattern} -> $var =~ s/pattern$// - remove shortest suffix
                    let parts: Vec<&str> = var.split("%").collect();
                    if parts.len() == 2 {
                        let var_name = parts[0];
                        let pattern = parts[1];
                        format!("${} =~ s/{}$//", var_name, pattern)
                    } else {
                        format!("${}", var)
                    }
                } else if var.contains(":-") {
                    // ${var:-default} -> defined($var) ? $var : 'default'
                    let parts: Vec<&str> = var.split(":-").collect();
                    if parts.len() == 2 {
                        let var_name = parts[0];
                        let default = parts[1];
                        format!("defined(${}) ? ${} : '{}'", var_name, var_name, default)
                    } else {
                        format!("${}", var)
                    }
                } else if var.contains(":=") {
                    // ${var:=default} -> $var //= 'default' (set if undefined)
                    let parts: Vec<&str> = var.split(":=").collect();
                    if parts.len() == 2 {
                        let var_name = parts[0];
                        let default = parts[1];
                        format!("${} //= '{}'", var_name, default)
                    } else {
                        format!("${}", var)
                    }
                } else if var.contains(":?") {
                    // ${var:?error} -> die if undefined
                    let parts: Vec<&str> = var.split(":?").collect();
                    if parts.len() == 2 {
                        let var_name = parts[0];
                        let error = parts[1];
                        format!("defined(${}) ? ${} : die('{}')", var_name, var_name, error)
                    } else {
                        format!("${}", var)
                    }
                } else if var.starts_with('#') && var.ends_with("[@]") {
                    // ${#arr[@]} -> scalar(@arr)
                    let array_name = &var[1..var.len()-3];
                    format!("scalar(@{})", array_name)
                } else if var.starts_with('!') && var.ends_with("[@]") {
                    // ${!map[@]} -> keys(%map)
                    let hash_name = &var[1..var.len()-3];
                    format!("keys(%{})", hash_name)
                } else if var.starts_with('#') && var.ends_with("[*]") {
                    // ${#arr[*]} -> scalar(@arr)
                    let array_name = &var[1..var.len()-3];
                    format!("scalar(@{})", array_name)
                } else if var.starts_with('!') && var.ends_with("[*]") {
                    // ${!map[*]} -> keys(%map)
                    let hash_name = &var[1..var.len()-3];
                    format!("keys(%{})", hash_name)
                } else {
                    format!("${}", var)
                }
            },
            Word::MapAccess(map_name, key) => {
                // For now, assume "map" is a hash and others are indexed arrays
                if map_name == "map" {
                    format!("${}{{{}}}", map_name, key)
                } else {
                    format!("${}[{}]", map_name, key)
                }
            },
            Word::MapKeys(map_name) => {
                // ${!map[@]} -> keys(%map)
                format!("keys(%{})", map_name)
            },
            Word::MapLength(map_name) => {
                // ${#arr[@]} -> scalar(@arr)
                format!("scalar(@{})", map_name)
            },
            Word::Arithmetic(expr) => self.convert_arithmetic_to_perl(&expr.expression),
            Word::BraceExpansion(expansion) => {
                // Handle brace expansion by expanding it to actual values
                if expansion.items.len() == 1 {
                    match &expansion.items[0] {
                        BraceItem::Range(range) => {
                            // Expand range like {1..5} to "1 2 3 4 5"
                            // Check if this is a character range
                            if let (Some(start_char), Some(end_char)) = (range.start.chars().next(), range.end.chars().next()) {
                                if start_char.is_ascii_lowercase() && end_char.is_ascii_lowercase() {
                                    // This is a character range
                                    let start = start_char as u8;
                                    let end = end_char as u8;
                                    if start <= end {
                                        let step = range.step.as_ref().and_then(|s| s.parse::<usize>().ok()).unwrap_or(1);
                                        let values: Vec<String> = (start..=end)
                                            .step_by(step)
                                            .map(|c| char::from(c).to_string())
                                            .collect();
                                        values.join(" ")
                                    } else {
                                        // Reverse range
                                        let step = range.step.as_ref().and_then(|s| s.parse::<usize>().ok()).unwrap_or(1);
                                        let values: Vec<String> = (end..=start)
                                            .rev()
                                            .step_by(step)
                                            .map(|c| char::from(c).to_string())
                                            .collect();
                                        values.join(" ")
                                    }
                                } else {
                                    // This is a numeric range
                                    self.expand_brace_range(range)
                                }
                            } else {
                                // This is a numeric range
                                self.expand_brace_range(range)
                            }
                        }
                        BraceItem::Literal(s) => {
                            // Handle literal strings that might contain ranges like "a..c" or "00..04..2"
                            if s.contains("..") {
                                let parts: Vec<&str> = s.split("..").collect();
                                if parts.len() == 2 {
                                    // Simple range like "a..c"
                                    if let (Some(start_char), Some(end_char)) = (parts[0].chars().next(), parts[1].chars().next()) {
                                        if start_char.is_ascii_lowercase() && end_char.is_ascii_lowercase() {
                                            let start = start_char as u8;
                                            let end = end_char as u8;
                                            if start <= end {
                                                let values: Vec<String> = (start..=end)
                                                    .map(|c| char::from(c).to_string())
                                                    .collect();
                                                values.join(" ")
                                            } else {
                                                s.clone()
                                            }
                                        } else {
                                            s.clone()
                                        }
                                    } else {
                                        s.clone()
                                    }
                                } else if parts.len() == 3 && parts[1].contains("..") {
                                    // Character range with step like "a..z..3"
                                    let sub_parts: Vec<&str> = parts[1].split("..").collect();
                                    if sub_parts.len() == 2 {
                                        if let (Some(start_char), Some(end_char)) = (parts[0].chars().next(), sub_parts[1].chars().next()) {
                                            if start_char.is_ascii_lowercase() && end_char.is_ascii_lowercase() {
                                                if let Ok(step) = parts[2].parse::<usize>() {
                                                    let start = start_char as u8;
                                                    let end = end_char as u8;
                                                    if start <= end {
                                                        let values: Vec<String> = (start..=end)
                                                            .step_by(step)
                                                            .map(|c| char::from(c).to_string())
                                                            .collect();
                                                        values.join(" ")
                                                    } else {
                                                        s.clone()
                                                    }
                                                } else {
                                                    s.clone()
                                                }
                                            } else {
                                                s.clone()
                                            }
                                        } else {
                                            s.clone()
                                        }
                                    } else {
                                        s.clone()
                                    }
                                } else if parts.len() == 3 {
                                    // Range with step like "00..04..2"
                                    if let (Ok(start), Ok(end), Ok(step)) = (parts[0].parse::<i64>(), parts[1].parse::<i64>(), parts[2].parse::<i64>()) {
                                        let values: Vec<String> = (start..=end).step_by(step as usize).map(|i| {
                                            // Preserve leading zeros by formatting with the same width as the original
                                            if parts[0].starts_with('0') && parts[0].len() > 1 {
                                                format!("{:0width$}", i, width = parts[0].len())
                                            } else {
                                                i.to_string()
                                            }
                                        }).collect();
                                        values.join(" ")
                                    } else {
                                        s.clone()
                                    }
                                } else {
                                    s.clone()
                                }
                            } else if s.contains(',') {
                                // Handle comma-separated sequences like "a,b,c"
                                let parts: Vec<&str> = s.split(',').collect();
                                if parts.len() > 1 {
                                    parts.join(" ")
                                } else {
                                    s.clone()
                                }
                            } else {
                                s.clone()
                            }
                        }
                        BraceItem::Sequence(seq) => {
                            // Expand sequence like {a,b,c} to "a b c"
                            seq.join(" ")
                        }
                    }
                } else {
                    // Multiple items - expand each one and join
                    let expanded_items: Vec<Vec<String>> = expansion.items.iter().map(|item| {
                        match item {
                            BraceItem::Literal(s) => vec![s.clone()],
                            BraceItem::Range(range) => {
                                self.expand_brace_range(range).split_whitespace().map(|s| s.to_string()).collect()
                            }
                            BraceItem::Sequence(seq) => seq.clone()
                        }
                    }).collect();
                    
                    // Generate cartesian product for multiple brace expansions like {a,b,c}{1,2,3}
                    if expanded_items.len() == 2 {
                        let mut result = Vec::new();
                        for item1 in &expanded_items[0] {
                            for item2 in &expanded_items[1] {
                                result.push(format!("{}{}", item1, item2));
                            }
                        }
                        result.join(" ")
                    } else {
                        // For more than 2 items, just join them (this could be enhanced for full cartesian product)
                        expanded_items.iter().map(|items| items.join(" ")).collect::<Vec<_>>().join(" ")
                    }
                }
            }
            Word::CommandSubstitution(_) => "`command`".to_string(),
            Word::StringInterpolation(interp) => {
                // For function arguments, we need quoted strings
                // If it's just a single literal part, wrap it in quotes
                if interp.parts.len() == 1 {
                    if let StringPart::Literal(s) = &interp.parts[0] {
                        return format!("\"{}\"", self.escape_perl_string(s));
                    }
                    // If it's just a single parameter expansion part, return it without quotes
                    if let StringPart::ParameterExpansion(pe) = &interp.parts[0] {
                        return self.generate_parameter_expansion(pe);
                    }
                }
                // For more complex interpolations, wrap the result in quotes
                let content = self.convert_string_interpolation_to_perl(interp);
                format!("\"{}\"", content)
            },
        }
    }

    fn word_to_perl_for_test(&self, word: &Word) -> String {
        match word {
            Word::Literal(s) => {
                // For test commands, use single quotes to match test expectations
                format!("'{}'", self.escape_perl_string(s))
            },
            Word::Array(name, elements) => {
                // Convert array declaration to Perl array
                let elements_str = elements.iter()
                    .map(|e| format!("'{}'", e.replace("'", "\\'")))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("@{} = ({});", name, elements_str)
            },
            Word::ParameterExpansion(pe) => self.generate_parameter_expansion(pe),
            Word::Variable(var) => {
                // First try to extract parameter expansion operators using the helper method
                if let Some((base_var, operator)) = self.extract_parameter_expansion(var) {
                    self.apply_parameter_expansion(&base_var, &operator)
                } else if var.contains(":-") {
                    // ${var:-default} -> defined($var) ? $var : 'default'
                    let parts: Vec<&str> = var.split(":-").collect();
                    if parts.len() == 2 {
                        let var_name = parts[0];
                        let default = parts[1];
                        // Ensure variable is declared
                        if var_name.starts_with('$') && !self.declared_locals.contains(&var_name[1..]) {
                            format!("my {} = undef; defined({}) ? {} : '{}'", var_name, var_name, var_name, default)
                        } else {
                            format!("defined({}) ? {} : '{}'", var_name, var_name, default)
                        }
                    } else {
                        format!("${}", var)
                    }
                } else if var.contains(":=") {
                    // ${var:=default} -> $var //= 'default' (set if undefined)
                    let parts: Vec<&str> = var.split(":=").collect();
                    if parts.len() == 2 {
                        let var_name = parts[0];
                        let default = parts[1];
                        // Ensure variable is declared
                        if var_name.starts_with('$') && !self.declared_locals.contains(&var_name[1..]) {
                            format!("my {} = undef; {} //= '{}'", var_name, var_name, default)
                        } else {
                            format!("{} //= '{}'", var_name, default)
                        }
                    } else {
                        format!("${}", var)
                    }
                } else if var.contains(":?") {
                    // ${var:?error} -> die if undefined
                    let parts: Vec<&str> = var.split(":?").collect();
                    if parts.len() == 2 {
                        let var_name = parts[0];
                        let error = parts[1];
                        // Ensure variable is declared
                        if var_name.starts_with('$') && !self.declared_locals.contains(&var_name[1..]) {
                            format!("my {} = undef; defined({}) ? {} : die('{}')", var_name, var_name, var_name, error)
                        } else {
                            format!("defined({}) ? {} : die('{}')", var_name, var_name, error)
                        }
                    } else {
                        format!("${}", var)
                    }
                } else if var.starts_with('#') && var.ends_with("[@]") {
                    // ${#arr[@]} -> scalar(@arr)
                    let array_name = &var[1..var.len()-3];
                    format!("scalar(@{})", array_name)
                } else if var.starts_with('!') && var.ends_with("[@]") {
                    // ${!map[@]} -> keys(%map)
                    let hash_name = &var[1..var.len()-3];
                    format!("keys(%{})", hash_name)
                } else if var.starts_with('#') && var.ends_with("[*]") {
                    // ${#arr[*]} -> scalar(@arr)
                    let array_name = &var[1..var.len()-3];
                    format!("scalar(@{})", array_name)
                } else if var.starts_with('!') && var.ends_with("[*]") {
                    // ${!map[*]} -> keys(%map)
                    let hash_name = &var[1..var.len()-3];
                    format!("keys(%{})", hash_name)
                } else {
                    format!("${}", var)
                }
            },
            Word::MapAccess(map_name, key) => {
                // For now, assume "map" is a hash and others are indexed arrays
                if map_name == "map" {
                    format!("${}{{{}}}", map_name, key)
                } else {
                    format!("${}[{}]", map_name, key)
                }
            },
            Word::MapKeys(map_name) => {
                // ${!map[@]} -> keys(%map)
                format!("keys(%{})", map_name)
            },
            Word::MapLength(map_name) => {
                // ${#arr[@]} -> scalar(@arr)
                format!("scalar(@{})", map_name)
            },
            Word::Arithmetic(expr) => self.convert_arithmetic_to_perl(&expr.expression),
            Word::BraceExpansion(expansion) => {
                // Handle brace expansion by expanding it to actual values
                if expansion.items.len() == 1 {
                    match &expansion.items[0] {
                        BraceItem::Range(range) => {
                            // Expand range like {1..5} to "1 2 3 4 5"
                            if let (Ok(start), Ok(end)) = (range.start.parse::<i64>(), range.end.parse::<i64>()) {
                                let step = range.step.as_ref().and_then(|s| s.parse::<i64>().ok()).unwrap_or(1);
                                let values: Vec<String> = if step > 0 {
                                    (start..=end).step_by(step as usize).map(|i| i.to_string()).collect()
                                } else {
                                    (end..=start).rev().step_by((-step) as usize).map(|i| i.to_string()).collect()
                                };
                                values.join(" ")
                            } else {
                                // If parsing fails, fall back to literal
                                format!("{{{}}}", range.start)
                            }
                        }
                        BraceItem::Literal(s) => s.clone(),
                        BraceItem::Sequence(seq) => {
                            // Expand sequence like {a,b,c} to "a b c"
                            seq.join(" ")
                        }
                    }
                } else {
                    // Multiple items - expand each one and join
                    let parts: Vec<String> = expansion.items.iter().map(|item| {
                        match item {
                            BraceItem::Literal(s) => s.clone(),
                            BraceItem::Range(range) => {
                                if let (Ok(start), Ok(end)) = (range.start.parse::<i64>(), range.end.parse::<i64>()) {
                                    let step = range.step.as_ref().and_then(|s| s.parse::<i64>().ok()).unwrap_or(1);
                                    let values: Vec<String> = if step > 0 {
                                        (start..=end).step_by(step as usize).map(|i| i.to_string()).collect()
                                    } else {
                                        (end..=start).rev().step_by((-step) as usize).map(|i| i.to_string()).collect()
                                    };
                                    values.join(" ")
                                } else {
                                    format!("{{{}}}", range.start)
                                }
                            }
                            BraceItem::Sequence(seq) => seq.join(" ")
                        }
                    }).collect();
                    parts.join(" ")
                }
            }
            Word::CommandSubstitution(_) => "`command`".to_string(),
            Word::StringInterpolation(interp) => {
                // For test commands, simple literal strings need to be quoted
                if interp.parts.len() == 1 {
                    if let StringPart::Literal(s) = &interp.parts[0] {
                        return format!("'{}'", self.escape_perl_string(s));
                    }
                }
                self.convert_string_interpolation_to_perl(interp)
            },
        }
    }

    fn convert_extglob_to_perl_regex(&self, pattern: &str) -> String {
        // Handle extglob patterns like !(*.min).js
        if pattern.starts_with("!(") && pattern.contains(")") {
            if let Some(close_paren) = pattern.find(')') {
                let negated_pattern = &pattern[2..close_paren];
                let suffix = &pattern[close_paren + 1..];
                
                // Convert the negated pattern to a regex
                let negated_regex = self.convert_extglob_negated_pattern(negated_pattern);
                
                if suffix.is_empty() {
                    // No suffix, just negate the pattern
                    format!("^(?!{})$", negated_regex)
                } else {
                    // Has suffix, we need to check if the string ends with the suffix
                    // but the part before the suffix doesn't match the negated pattern
                    let suffix_regex = self.convert_simple_pattern_to_regex(suffix);
                    
                    // For !(*.min).js, we want to match strings that:
                    // 1. End with .js
                    // 2. The part before .js doesn't match *.min
                    
                    // The correct approach is to check if the string doesn't match the pattern
                    // that would be formed by combining the negated pattern with the suffix
                    // For !(*.min).js, we want to avoid matching strings that end with .min.js
                    
                    // The regex should be: ^(?!.*\.min\.js$).*\.js$
                    // This means: start of string, not followed by anything ending in .min.js, then anything, then .js, then end
                    
                    // For !(*.min).js, we want to avoid matching strings that end with .min.js
                    // So we check if the string doesn't match the pattern that would be formed
                    // by combining the negated pattern with the suffix
                    
                    // The negated_regex already starts with .* (from the * conversion),
                    // so we don't need to add another .* in front
                    let combined_negated = format!("{}{}", negated_regex, suffix_regex);
                    
                    // We need to allow any content before the suffix, so the final regex should be:
                    // ^(?!.*\.min\.js$).*\.js$ - this allows any content before .js
                    format!("^(?!{}){}$", combined_negated, ".*".to_string() + &suffix_regex)
                }
            } else {
                // Fallback if parentheses don't match
                self.convert_simple_pattern_to_regex(pattern)
            }
        } else {
            // Not an extglob pattern, use regular conversion
            self.convert_simple_pattern_to_regex(pattern)
        }
    }
    
    fn convert_extglob_negated_pattern(&self, pattern: &str) -> String {
        // For extglob negated patterns like *.min, we need to handle * specially
        // The * in extglob means "any sequence of characters" 
        // We want to create a regex that matches the literal pattern
        // For *.min, we want to match any sequence followed by .min
        // First escape special characters, then convert * to .*
        pattern
            .replace(".", "\\.") // Escape dots first
            .replace("[", "\\[") // Escape brackets
            .replace("]", "\\]") // Escape brackets
            .replace("(", "\\(") // Escape parentheses
            .replace(")", "\\)") // Escape parentheses
            .replace("*", ".*")  // Convert * to .* for regex
            .replace("?", ".")   // Convert ? to . for regex
    }
    
    fn convert_simple_pattern_to_regex(&self, pattern: &str) -> String {
        // Convert shell glob patterns to regex
        pattern
            .replace("*", ".*")
            .replace("?", ".")
            .replace(".", "\\.")
            .replace("[", "\\[")
            .replace("]", "\\]")
            .replace("(", "\\(")
            .replace(")", "\\)")
    }

    fn expand_brace_range(&self, range: &crate::ast::BraceRange) -> String {
        // First check if this is a character range
        if let (Some(start_char), Some(end_char)) = (range.start.chars().next(), range.end.chars().next()) {
            if start_char.is_ascii_lowercase() && end_char.is_ascii_lowercase() {
                // This is a character range
                let start = start_char as u8;
                let end = end_char as u8;
                if start <= end {
                    let step = range.step.as_ref().and_then(|s| s.parse::<usize>().ok()).unwrap_or(1);
                    let values: Vec<String> = (start..=end)
                        .step_by(step)
                        .map(|c| char::from(c).to_string())
                        .collect();
                    values.join(" ")
                } else {
                    // Reverse range
                    let step = range.step.as_ref().and_then(|s| s.parse::<usize>().ok()).unwrap_or(1);
                    let values: Vec<String> = (end..=start)
                        .rev()
                        .step_by(step)
                        .map(|c| char::from(c).to_string())
                        .collect();
                    values.join(" ")
                }
            } else {
                // Try numeric range
                if let (Ok(start), Ok(end)) = (range.start.parse::<i64>(), range.end.parse::<i64>()) {
                    let step = range.step.as_ref().and_then(|s| s.parse::<i64>().ok()).unwrap_or(1);
                    let values: Vec<String> = if step > 0 {
                        (start..=end).step_by(step as usize).map(|i| {
                            // Preserve leading zeros by formatting with the same width as the original
                            if range.start.starts_with('0') && range.start.len() > 1 {
                                format!("{:0width$}", i, width = range.start.len())
                            } else {
                                i.to_string()
                            }
                        }).collect()
                    } else {
                        (end..=start).rev().step_by((-step) as usize).map(|i| {
                            if range.start.starts_with('0') && range.start.len() > 1 {
                                format!("{:0width$}", i, width = range.start.len())
                            } else {
                                i.to_string()
                            }
                        }).collect()
                    };
                    values.join(" ")
                } else {
                    // If parsing fails, fall back to literal
                    format!("{{{}}}", range.start)
                }
            }
        } else {
            // Try numeric range
            if let (Ok(start), Ok(end)) = (range.start.parse::<i64>(), range.end.parse::<i64>()) {
                let step = range.step.as_ref().and_then(|s| s.parse::<i64>().ok()).unwrap_or(1);
                let values: Vec<String> = if step > 0 {
                    (start..=end).step_by(step as usize).map(|i| {
                        // Preserve leading zeros by formatting with the same width as the original
                        if range.start.starts_with('0') && range.start.len() > 1 {
                            format!("{:0width$}", i, width = range.start.len())
                        } else {
                            i.to_string()
                        }
                    }).collect()
                } else {
                    (end..=start).rev().step_by((-step) as usize).map(|i| {
                        if range.start.starts_with('0') && range.start.len() > 1 {
                            format!("{:0width$}", i, width = range.start.len())
                        } else {
                            i.to_string()
                        }
                    }).collect()
                };
                values.join(" ")
            } else {
                // If parsing fails, fall back to literal
                format!("{{{}}}", range.start)
            }
        }
    }

    fn combine_adjacent_brace_expansions(&self, args: &[Word]) -> Vec<String> {
        let mut result = Vec::new();
        let mut i = 0;
        
        while i < args.len() {
            if let Word::BraceExpansion(expansion) = &args[i] {
                // Check if the next argument is also a brace expansion
                if i + 1 < args.len() {
                    if let Word::BraceExpansion(next_expansion) = &args[i + 1] {
                        // We have two adjacent brace expansions - combine them
                        let left_items = self.expand_brace_expansion_to_strings(expansion);
                        let right_items = self.expand_brace_expansion_to_strings(next_expansion);
                        
                        // Generate cartesian product
                        for left in &left_items {
                            for right in &right_items {
                                result.push(format!("{}{}", left, right));
                            }
                        }
                        i += 2; // Skip both expansions
                        continue;
                    }
                }
                
                // Single brace expansion
                let expanded = self.expand_brace_expansion_to_strings(expansion);
                result.extend(expanded);
            } else {
                // Non-brace expansion word
                result.push(self.word_to_perl(&args[i]));
            }
            i += 1;
        }
        
        result
    }

    fn expand_brace_expansion_to_strings(&self, expansion: &crate::ast::BraceExpansion) -> Vec<String> {
        let mut result = Vec::new();
        
        for item in &expansion.items {
            match item {
                BraceItem::Literal(s) => {
                    // Handle literal strings that might contain ranges like "a..c" or "00..04..2"
                    if s.contains("..") {
                        let parts: Vec<&str> = s.split("..").collect();
                        if parts.len() == 2 {
                            // Simple range like "a..c"
                            if let (Some(start_char), Some(end_char)) = (parts[0].chars().next(), parts[1].chars().next()) {
                                if start_char.is_ascii_lowercase() && end_char.is_ascii_lowercase() {
                                    let start = start_char as u8;
                                    let end = end_char as u8;
                                    if start <= end {
                                        let values: Vec<String> = (start..=end)
                                            .map(|c| char::from(c).to_string())
                                            .collect();
                                        result.extend(values);
                                    } else {
                                        result.push(s.clone());
                                    }
                                } else {
                                    result.push(s.clone());
                                }
                            } else {
                                result.push(s.clone());
                            }
                        } else if parts.len() == 3 {
                            // Range with step like "00..04..2"
                            if let (Ok(start), Ok(end), Ok(step)) = (parts[0].parse::<i64>(), parts[1].parse::<i64>(), parts[2].parse::<i64>()) {
                                let values: Vec<String> = (start..=end).step_by(step as usize).map(|i| {
                                    // Preserve leading zeros by formatting with the same width as the original
                                    if parts[0].starts_with('0') && parts[0].len() > 1 {
                                        format!("{:0width$}", i, width = parts[0].len())
                                    } else {
                                        i.to_string()
                                    }
                                }).collect();
                                result.extend(values);
                            } else {
                                result.push(s.clone());
                            }
                        } else {
                            result.push(s.clone());
                        }
                    } else if s.contains(',') {
                        // Handle comma-separated sequences like "a,b,c"
                        let parts: Vec<&str> = s.split(',').collect();
                        if parts.len() > 1 {
                            result.extend(parts.iter().map(|&s| s.to_string()));
                        } else {
                            result.push(s.clone());
                        }
                    } else {
                        result.push(s.clone());
                    }
                }
                BraceItem::Range(range) => {
                    // Check if this is a character range
                    if let (Some(start_char), Some(end_char)) = (range.start.chars().next(), range.end.chars().next()) {
                        if start_char.is_ascii_lowercase() && end_char.is_ascii_lowercase() {
                            // This is a character range
                            let start = start_char as u8;
                            let end = end_char as u8;
                            if start <= end {
                                let step = range.step.as_ref().and_then(|s| s.parse::<usize>().ok()).unwrap_or(1);
                                let values: Vec<String> = (start..=end)
                                    .step_by(step)
                                    .map(|c| char::from(c).to_string())
                                    .collect();
                                result.extend(values);
                            } else {
                                // Reverse range
                                let step = range.step.as_ref().and_then(|s| s.parse::<usize>().ok()).unwrap_or(1);
                                let values: Vec<String> = (end..=start)
                                    .rev()
                                    .step_by(step)
                                    .map(|c| char::from(c).to_string())
                                    .collect();
                                result.extend(values);
                            }
                        } else {
                            // This is a numeric range
                            let expanded = self.expand_brace_range(range);
                            result.extend(expanded.split_whitespace().map(|s| s.to_string()));
                        }
                    } else {
                        // This is a numeric range
                        let expanded = self.expand_brace_range(range);
                        result.extend(expanded.split_whitespace().map(|s| s.to_string()));
                    }
                }
                BraceItem::Sequence(seq) => {
                    result.extend(seq.iter().cloned());
                }
            }
        }
        
        result
    }

    fn extract_parameter_expansion(&self, var: &str) -> Option<(String, String)> {
        // Try to extract base variable name and operator from malformed parameter expansion
        // Handle case modification operators
        if var.contains("^^") {
            let var_name = var.replace("^^", "");
            Some((var_name, "^^".to_string()))
        } else if var.contains(",,)") {
            let var_name = var.replace(",,)", "");
            Some((var_name, ",,".to_string()))
        } else if var.contains("^)") {
            let var_name = var.replace("^)", "");
            Some((var_name, "^".to_string()))
        } else if var.contains("##*/") {
            let var_name = var.replace("##*/", "");
            Some((var_name, "##*/".to_string()))
        } else if var.contains("%/*") {
            let var_name = var.replace("%/*", "");
            Some((var_name, "%/*".to_string()))
        } else if var.contains("//") {
            let parts: Vec<&str> = var.split("//").collect();
            if parts.len() == 3 {
                Some((parts[0].to_string(), "//".to_string()))
            } else {
                None
            }
        } else if var.contains("#") && !var.starts_with('#') {
            let parts: Vec<&str> = var.split("#").collect();
            if parts.len() == 2 {
                Some((parts[0].to_string(), "#".to_string()))
            } else {
                None
            }
        } else if var.contains("%") && !var.starts_with('%') {
            let parts: Vec<&str> = var.split("%").collect();
            if parts.len() == 2 {
                Some((parts[0].to_string(), "%".to_string()))
            } else {
                None
            }
        } else {
            None
        }
    }

    fn apply_parameter_expansion(&self, base_var: &str, operator: &str) -> String {
        match operator {
            "^^" => format!("uc(${})", base_var),
            ",," => format!("lc(${})", base_var),
            "^" => format!("ucfirst(${})", base_var),
            "##*/" => format!("basename(${})", base_var),
            "%/*" => format!("dirname(${})", base_var),
            "//" => format!("${} =~ s///g", base_var), // Placeholder for pattern replacement
            "#" => format!("${} =~ s/^{}//", base_var, ""), // Placeholder for pattern
            "%" => format!("${} =~ s/{}$//", base_var, ""), // Placeholder for pattern
            _ => format!("${}", base_var)
        }
    }

    fn generate_parameter_expansion(&self, pe: &ParameterExpansion) -> String {
        match &pe.operator {
            ParameterExpansionOperator::UppercaseAll => format!("uc(${})", pe.variable),
            ParameterExpansionOperator::LowercaseAll => format!("lc(${})", pe.variable),
            ParameterExpansionOperator::UppercaseFirst => format!("ucfirst(${})", pe.variable),
            ParameterExpansionOperator::RemoveLongestPrefix(pattern) => {
                let escaped_pattern = self.escape_perl_regex(pattern);
                format!("do {{ my $temp = ${}; $temp =~ s/^{}//; $temp }}", pe.variable, escaped_pattern)
            },
            ParameterExpansionOperator::RemoveShortestPrefix(pattern) => {
                let escaped_pattern = self.escape_perl_regex(pattern);
                format!("do {{ my $temp = ${}; $temp =~ s/^{}//; $temp }}", pe.variable, escaped_pattern)
            },
            ParameterExpansionOperator::RemoveLongestSuffix(pattern) => {
                let escaped_pattern = self.escape_perl_regex(pattern);
                format!("do {{ my $temp = ${}; $temp =~ s/{}$//; $temp }}", pe.variable, escaped_pattern)
            },
            ParameterExpansionOperator::RemoveShortestSuffix(pattern) => {
                let escaped_pattern = self.escape_perl_regex(pattern);
                format!("do {{ my $temp = ${}; $temp =~ s/{}$//; $temp }}", pe.variable, escaped_pattern)
            },
            ParameterExpansionOperator::SubstituteAll(pattern, replacement) => {
                let escaped_pattern = self.escape_perl_regex(pattern);
                let escaped_replacement = self.escape_perl_regex(replacement);
                format!("do {{ my $temp = ${}; $temp =~ s/{}/{}/g; $temp }}", pe.variable, escaped_pattern, escaped_replacement)
            },
            ParameterExpansionOperator::DefaultValue(default) => format!("defined(${}) ? ${} : '{}'", pe.variable, pe.variable, default),
            ParameterExpansionOperator::AssignDefault(default) => format!("${} //= '{}'", pe.variable, default),
            ParameterExpansionOperator::ErrorIfUnset(error) => format!("defined(${}) ? ${} : die('{}')", pe.variable, pe.variable, error),
            ParameterExpansionOperator::Basename => format!("basename(${})", pe.variable),
            ParameterExpansionOperator::Dirname => format!("dirname(${})", pe.variable),
        }
    }

    fn convert_glob_to_regex(&self, pattern: &str) -> String {
        SharedUtils::convert_glob_to_regex(pattern)
    }
} 
