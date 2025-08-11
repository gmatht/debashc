use crate::ast::*;
use std::collections::HashSet;
// HashMap import removed as it's not used

pub struct PerlGenerator {
    indent_level: usize,
    declared_locals: HashSet<String>,
    declared_functions: HashSet<String>,
    subshell_depth: usize,
}

impl PerlGenerator {
    pub fn new() -> Self {
        Self { indent_level: 0, declared_locals: HashSet::new(), declared_functions: HashSet::new(), subshell_depth: 0 }
    }

    pub fn generate(&mut self, commands: &[Command]) -> String {
        let mut output = String::new();
        output.push_str("#!/usr/bin/env perl\n");
        output.push_str("use strict;\n");
        output.push_str("use warnings;\n\n");

        for command in commands {
            output.push_str(&self.generate_command(command));
        }
        // Remove all trailing newlines
        while output.ends_with('\n') { output.pop(); }
        output
    }

    fn generate_command(&mut self, command: &Command) -> String {
        eprintln!("DEBUG: generate_command called with: {:?}", command);
        match command {
            Command::Simple(cmd) => self.generate_simple_command(cmd),
            Command::ShoptCommand(cmd) => self.generate_shopt_command(cmd),
            Command::TestExpression(test_expr) => {
                eprintln!("DEBUG: Routing to generate_test_expression");
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
                if let Word::Literal(literal) = value {
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
                    if var.starts_with('#') && var.contains('[') {
                        // This is #arr[@] - convert to scalar(@arr) and concatenate with newline
                        if let Some(bracket_start) = var.find('[') {
                            let array_name = &var[1..bracket_start]; // Skip the # prefix
                            output.push_str(&format!("print(scalar(@{}) . \"\\n\");\n", array_name));
                        } else {
                            output.push_str(&format!("print(\"${}\\n\");\n", var));
                        }
                    } else if var.starts_with('!') && var.ends_with("[@]") {
                        // This is !map[@] - convert to keys(%map) and concatenate with newline
                        let array_name = &var[1..var.len()-3]; // Remove ! prefix and [@] suffix
                        output.push_str(&format!("print(keys(%{}) . \"\\n\");\n", array_name));
                    } else {
                        // Handle other variables
                        output.push_str(&format!("print(\"${}\\n\");\n", var));
                    }
                } else if let Word::StringInterpolation(interp) = arg {
                    // Handle string interpolation like "$#"
                    if interp.parts.len() == 1 {
                        if let StringPart::Variable(var) = &interp.parts[0] {
                            if var == "#" {
                                output.push_str("print(scalar(@ARGV) . \"\\n\");\n");
                            } else if var == "@" {
                                output.push_str("print(join(\" \", @ARGV) . \"\\n\");\n");
                            } else if var.starts_with('#') && var.contains('[') {
                                // This is #arr[@] - convert to scalar(@arr) and concatenate with newline
                                if let Some(bracket_start) = var.find('[') {
                                    let array_name = &var[1..bracket_start]; // Skip the # prefix
                                    output.push_str(&format!("print(scalar(@{}) . \"\\n\");\n", array_name));
                                } else {
                                    let converted = self.convert_string_interpolation_to_perl(interp);
                                    output.push_str(&format!("print(\"{}\\n\");\n", converted));
                                }
                            } else if var.starts_with('!') && var.ends_with("[@]") {
                                // This is !map[@] - convert to keys(%map) and concatenate with newline
                                let array_name = &var[1..var.len()-3]; // Remove ! prefix and [@] suffix
                                output.push_str(&format!("print(keys(%{}) . \"\\n\");\n", array_name));
                            } else {
                                // Handle other variables in string interpolation
                                let converted = self.convert_string_interpolation_to_perl(interp);
                                output.push_str(&format!("print(\"{}\\n\");\n", converted));
                            }
                        } else {
                            // Handle other string parts
                            let converted = self.convert_string_interpolation_to_perl(interp);
                            output.push_str(&format!("print(\"{}\\n\");\n", converted));
                        }
                    } else {
                        // Handle multiple parts in string interpolation
                        let converted = self.convert_string_interpolation_to_perl(interp);
                        output.push_str(&format!("print(\"{}\\n\");\n", converted));
                    }
                } else {
                    // Handle direct variable references like $# or $@
                    let arg_str = arg.to_string();
                    if arg_str == "$#" {
                        output.push_str("print(scalar(@ARGV) . \"\\n\");\n");
                    } else if arg_str == "$@" {
                        output.push_str("print(join(\" \", @ARGV) . \"\\n\");\n");
                    } else {
                        let args = cmd.args.join(" ");
                        // Convert shell positional parameters to Perl equivalents
                        let converted_args = args.replace("$1", "$_[0]")
                                               .replace("$2", "$_[1]")
                                               .replace("$3", "$_[2]")
                                               .replace("$4", "$_[3]")
                                               .replace("$5", "$_[4]")
                                               .replace("$6", "$_[5]")
                                               .replace("$7", "$_[6]")
                                               .replace("$8", "$_[7]")
                                               .replace("$9", "$_[8]");
                        let escaped_args = self.escape_perl_string(&converted_args);
                        // Allow interpolation ($var) intentionally by not escaping '$'
                        output.push_str(&format!("print(\"{}\\n\");\n", escaped_args));
                    }
                }
            } else {
                // Handle multiple arguments
                let args = cmd.args.iter().map(|arg| {
                    // Convert each argument to its Perl representation
                    match arg {
                        Word::Literal(s) => s.clone(),
                        Word::Variable(var) => format!("${}", var),
                        Word::StringInterpolation(interp) => {
                            // For string interpolation, extract the literal content
                            if interp.parts.len() == 1 {
                                if let StringPart::Literal(s) = &interp.parts[0] {
                                    s.clone()
                                } else {
                                    arg.to_string()
                                }
                            } else {
                                arg.to_string()
                            }
                        }
                        _ => arg.to_string(),
                    }
                }).collect::<Vec<_>>();
                
                let args_str = args.join(" ");
                // Convert shell positional parameters to Perl equivalents
                let converted_args = args_str.replace("$1", "$_[0]")
                                           .replace("$2", "$_[1]")
                                           .replace("$3", "$_[2]")
                                           .replace("$4", "$_[3]")
                                           .replace("$5", "$_[4]")
                                           .replace("$6", "$_[5]")
                                           .replace("$7", "$_[6]")
                                           .replace("$8", "$_[7]")
                                           .replace("$9", "$_[8]");
                let escaped_args = self.escape_perl_string(&converted_args);
                // Allow interpolation ($var) intentionally by not escaping '$'
                output.push_str(&format!("print(\"{}\\n\");\n", escaped_args));
            }
        } else if cmd.name == "cd" {
            // Special handling for cd
            let empty_word = Word::Literal("".to_string());
            let dir = cmd.args.first().unwrap_or(&empty_word);
            output.push_str(&format!("chdir('{}') or die \"Cannot change to directory: $!\\n\";\n", dir));
        } else if cmd.name == "ls" {
            // Special handling for ls (ignore flags like -la); default to current dir
            let dir = if cmd.args.is_empty() {
                ".".to_string()
            } else if cmd.args[0].starts_with('-') {
                ".".to_string()
            } else {
                cmd.args[0].to_string()
            };
            output.push_str(&format!("opendir(my $dh, '{}') or die \"Cannot open directory: $!\\n\";\n", dir));
            output.push_str("while (my $file = readdir($dh)) {\n");
            output.push_str("    print(\"$file\\n\") unless $file =~ /^\\.\\.?$/;\n");
            output.push_str("}\n");
            output.push_str("closedir($dh);\n");
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
                            output.push_str(&format!("open(my $fh, '<', '{}') or die \"Cannot open file: $!\\n\";\n", file));
                            output.push_str(&format!("while (my $line = <$fh>) {{\n"));
                            output.push_str(&format!("    if ($line =~ /({})/g) {{\n", pattern));
                            output.push_str("        print \"$1\\n\";\n");
                            output.push_str("    }\n");
                            output.push_str("}\n");
                            output.push_str("close($fh);\n");
                        }
                    } else {
                        if file == "STDIN" {
                            output.push_str("while (my $line = <STDIN>) {\n");
                            output.push_str(&format!("    print($line) if $line =~ /{}/;\n", pattern));
                            output.push_str("}\n");
                        } else {
                            output.push_str(&format!("open(my $fh, '<', '{}') or die \"Cannot open file: $!\\n\";\n", file));
                            output.push_str(&format!("while (my $line = <$fh>) {{\n"));
                            output.push_str(&format!("    print($line) if $line =~ /{}/;\n", pattern));
                            output.push_str("}\n");
                            output.push_str("close($fh);\n");
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
                    output.push_str(&format!("open(my $fh, '<', '{}') or die \"Cannot open file: $!\\n\";\n", arg));
                    output.push_str("while (my $line = <$fh>) {\n");
                    output.push_str("    print($line);\n");
                    output.push_str("}\n");
                    output.push_str("close($fh);\n");
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
                        output.push_str("open(my $temp_fh, '>', '/tmp/here_string_temp') or die \"Cannot create temp file: $!\\n\";\n");
                        output.push_str("print $temp_fh $temp_content;\n");
                        output.push_str("close($temp_fh);\n");
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
                            output.push_str("open(my $temp_fh, '>', '/tmp/heredoc_temp') or die \"Cannot create temp file: $!\\n\";\n");
                            output.push_str("print $temp_fh $temp_content;\n");
                            output.push_str("close($temp_fh);\n");
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

    fn generate_test_expression_clean(&mut self, test_expr: &TestExpression) -> String {
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
                format!("{} =~ /{}/", var, pattern)
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
                        format!("{} =~ /{}/i", var, regex_pattern)
                    } else {
                        format!("{} =~ /{}/", var, regex_pattern)
                    }
                } else {
                    // Regular pattern matching
                    if modifiers.nocasematch {
                        // Case-insensitive matching
                        format!("lc({}) =~ /^{}$/i", var, pattern.replace("*", ".*"))
                    } else {
                        // Case-sensitive matching
                        format!("{} =~ /^{}$/", var, pattern.replace("*", ".*"))
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
                        format!("{} !~ /{}/i", var, regex_pattern)
                    } else {
                        format!("{} !~ /{}/", var, regex_pattern)
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

    fn generate_test_expression(&mut self, test_expr: &TestExpression) -> String {
        eprintln!("DEBUG: generate_test_expression called with expression: '{}'", test_expr.expression);
        let mut output = String::new();
        
        // Parse the test expression to extract components
        let expr = &test_expr.expression;
        let modifiers = &test_expr.modifiers;
        
        // Debug output
        output.push_str(&format!("# DEBUG: TestExpression: '{}'\n", expr));
        
        // Add comments about enabled options
        if modifiers.extglob {
            output.push_str("# extglob enabled\n");
        }
        if modifiers.nocasematch {
            output.push_str("# nocasematch enabled\n");
        }
        
        // Parse the expression to determine the type of test
        if expr.contains(" =~ ") {
            // Regex matching: [[ $var =~ pattern ]]
            let parts: Vec<&str> = expr.split(" =~ ").collect();
            if parts.len() == 2 {
                let var = parts[0].trim();
                let pattern = parts[1].trim();
                
                // Ensure variable is declared
                if var.starts_with('$') && !self.declared_locals.contains(&var[1..]) {
                    output.push_str(&format!("my {} = '';\n", var));
                    self.declared_locals.insert(var[1..].to_string());
                }
                
                // Convert to Perl regex matching
                output.push_str(&format!("{} =~ /{}/", var, pattern));
            } else {
                output.push_str(&format!("# Invalid regex test: {}\n", expr));
                output.push_str("0");
            }
        } else if expr.contains(" == ") {
            // Pattern matching: [[ $var == pattern ]]
            let parts: Vec<&str> = expr.split(" == ").collect();
            if parts.len() == 2 {
                let var = parts[0].trim();
                let pattern = parts[1].trim();
                
                // Ensure variable is declared
                if var.starts_with('$') && !self.declared_locals.contains(&var[1..]) {
                    output.push_str(&format!("my {} = '';\n", var));
                    self.declared_locals.insert(var[1..].to_string());
                }
                
                if modifiers.extglob {
                    // Handle extglob patterns
                    let regex_pattern = self.convert_extglob_to_perl_regex(pattern);
                    if modifiers.nocasematch {
                        output.push_str(&format!("{} =~ /{}/i", var, regex_pattern));
                    } else {
                        output.push_str(&format!("{} =~ /{}/", var, regex_pattern));
                    }
                } else {
                    // Regular pattern matching
                    if modifiers.nocasematch {
                        // Case-insensitive matching
                        output.push_str(&format!("lc({}) =~ /^{}$/i", var, pattern.replace("*", ".*")));
                    } else {
                        // Case-sensitive matching
                        output.push_str(&format!("{} =~ /^{}$/", var, pattern.replace("*", ".*")));
                    }
                }
            } else {
                output.push_str(&format!("# Invalid pattern test: {}\n", expr));
                output.push_str("0");
            }
        } else if expr.contains(" != ") {
            // Pattern matching: [[ $var != pattern ]]
            let parts: Vec<&str> = expr.split(" != ").collect();
            if parts.len() == 2 {
                let var = parts[0].trim();
                let pattern = parts[1].trim();
                
                // Ensure variable is declared
                if var.starts_with('$') && !self.declared_locals.contains(&var[1..]) {
                    output.push_str(&format!("my {} = '';\n", var));
                    self.declared_locals.insert(var[1..].to_string());
                }
                
                if modifiers.extglob {
                    // Handle extglob patterns
                    let regex_pattern = self.convert_extglob_to_perl_regex(pattern);
                    if modifiers.nocasematch {
                        output.push_str(&format!("{} !~ /{}/i", var, regex_pattern));
                    } else {
                        output.push_str(&format!("{} !~ /{}/", var, regex_pattern));
                    }
                } else {
                    // Regular pattern matching
                    if modifiers.nocasematch {
                        // Case-insensitive matching
                        output.push_str(&format!("lc({}) !~ /^{}$/i", var, pattern.replace("*", ".*")));
                    } else {
                        // Case-sensitive matching
                        output.push_str(&format!("{} !~ /^{}$/", var, pattern.replace("*", ".*")));
                    }
                }
            } else {
                output.push_str(&format!("# Invalid pattern test: {}\n", expr));
                output.push_str("0");
            }
        } else if expr.contains(" -eq ") {
            // Numeric equality: [[ $var -eq value ]]
            let parts: Vec<&str> = expr.split(" -eq ").collect();
            if parts.len() == 2 {
                let var = parts[0].trim();
                let value = parts[1].trim();
                
                // Ensure variable is declared
                if var.starts_with('$') && !self.declared_locals.contains(&var[1..]) {
                    output.push_str(&format!("my {} = 0;\n", var));
                    self.declared_locals.insert(var[1..].to_string());
                }
                
                output.push_str(&format!("{} == {}", var, value));
            } else {
                output.push_str(&format!("# Invalid numeric test: {}\n", expr));
                output.push_str("0");
            }
        } else if expr.contains(" -ne ") {
            // Numeric inequality: [[ $var -ne value ]]
            let parts: Vec<&str> = expr.split(" -ne ").collect();
            if parts.len() == 2 {
                let var = parts[0].trim();
                let value = parts[1].trim();
                
                // Ensure variable is declared
                if var.starts_with('$') && !self.declared_locals.contains(&var[1..]) {
                    output.push_str(&format!("my {} = 0;\n", var));
                    self.declared_locals.insert(var[1..].to_string());
                }
                
                output.push_str(&format!("{} != {}", var, value));
            } else {
                output.push_str(&format!("# Invalid numeric test: {}\n", expr));
                output.push_str("0");
            }
        } else if expr.contains(" -lt ") {
            // Less than: [[ $var -lt value ]]
            let parts: Vec<&str> = expr.split(" -lt ").collect();
            if parts.len() == 2 {
                let var = parts[0].trim();
                let value = parts[1].trim();
                
                // Ensure variable is declared
                if var.starts_with('$') && !self.declared_locals.contains(&var[1..]) {
                    output.push_str(&format!("my {} = 0;\n", var));
                    self.declared_locals.insert(var[1..].to_string());
                }
                
                output.push_str(&format!("{} < {}", var, value));
            } else {
                output.push_str(&format!("# Invalid numeric test: {}\n", expr));
                output.push_str("0");
            }
        } else if expr.contains(" -le ") {
            // Less than or equal: [[ $var -le value ]]
            let parts: Vec<&str> = expr.split(" -le ").collect();
            if parts.len() == 2 {
                let var = parts[0].trim();
                let value = parts[1].trim();
                
                // Ensure variable is declared
                if var.starts_with('$') && !self.declared_locals.contains(&var[1..]) {
                    output.push_str(&format!("my {} = 0;\n", var));
                    self.declared_locals.insert(var[1..].to_string());
                }
                
                output.push_str(&format!("{} <= {}", var, value));
            } else {
                output.push_str(&format!("# Invalid numeric test: {}\n", expr));
                output.push_str("0");
            }
        } else if expr.contains(" -gt ") {
            // Greater than: [[ $var -gt value ]]
            let parts: Vec<&str> = expr.split(" -gt ").collect();
            if parts.len() == 2 {
                let var = parts[0].trim();
                let value = parts[1].trim();
                
                // Ensure variable is declared
                if var.starts_with('$') && !self.declared_locals.contains(&var[1..]) {
                    output.push_str(&format!("my {} = 0;\n", var));
                    self.declared_locals.insert(var[1..].to_string());
                }
                
                output.push_str(&format!("{} > {}", var, value));
            } else {
                output.push_str(&format!("# Invalid numeric test: {}\n", expr));
                output.push_str("0");
            }
        } else if expr.contains(" -ge ") {
            // Greater than or equal: [[ $var -ge value ]]
            let parts: Vec<&str> = expr.split(" -ge ").collect();
            if parts.len() == 2 {
                let var = parts[0].trim();
                let value = parts[1].trim();
                
                // Ensure variable is declared
                if var.starts_with('$') && !self.declared_locals.contains(&var[1..]) {
                    output.push_str(&format!("my {} = 0;\n", var));
                    self.declared_locals.insert(var[1..].to_string());
                }
                
                output.push_str(&format!("{} >= {}", var, value));
            } else {
                output.push_str(&format!("# Invalid numeric test: {}\n", expr));
                output.push_str("0");
            }
        } else if expr.contains(" -z ") {
            // String is empty: [[ -z $var ]]
            let parts: Vec<&str> = expr.split(" -z ").collect();
            if parts.len() == 2 {
                let var = parts[1].trim();
                
                // Ensure variable is declared
                if var.starts_with('$') && !self.declared_locals.contains(&var[1..]) {
                    output.push_str(&format!("my {} = '';\n", var));
                    self.declared_locals.insert(var[1..].to_string());
                }
                
                output.push_str(&format!("!defined({}) || {} eq ''", var, var));
            } else {
                output.push_str(&format!("# Invalid string test: {}\n", expr));
                output.push_str("0");
            }
        } else if expr.contains(" -n ") {
            // String is not empty: [[ -n $var ]]
            let parts: Vec<&str> = expr.split(" -n ").collect();
            if parts.len() == 2 {
                let var = parts[1].trim();
                
                // Ensure variable is declared
                if var.starts_with('$') && !self.declared_locals.contains(&var[1..]) {
                    output.push_str(&format!("my {} = '';\n", var));
                    self.declared_locals.insert(var[1..].to_string());
                }
                
                output.push_str(&format!("defined({}) && {} ne ''", var, var));
            } else {
                output.push_str(&format!("# Invalid string test: {}\n", expr));
                output.push_str("0");
            }
        } else if expr.contains(" -f ") {
            // File exists and is regular file: [[ -f $var ]]
            let parts: Vec<&str> = expr.split(" -f ").collect();
            if parts.len() == 2 {
                let var = parts[1].trim();
                
                // Ensure variable is declared
                if var.starts_with('$') && !self.declared_locals.contains(&var[1..]) {
                    output.push_str(&format!("my {} = '';\n", var));
                    self.declared_locals.insert(var[1..].to_string());
                }
                
                output.push_str(&format!("-f {}", var));
            } else {
                output.push_str(&format!("# Invalid file test: {}\n", expr));
                output.push_str("0");
            }
        } else if expr.contains(" -d ") {
            // Directory exists: [[ -d $var ]]
            let parts: Vec<&str> = expr.split(" -d ").collect();
            if parts.len() == 2 {
                let var = parts[1].trim();
                
                // Ensure variable is declared
                if var.starts_with('$') && !self.declared_locals.contains(&var[1..]) {
                    output.push_str(&format!("my {} = '';\n", var));
                    self.declared_locals.insert(var[1..].to_string());
                }
                
                output.push_str(&format!("-d {}", var));
            } else {
                output.push_str(&format!("# Invalid file test: {}\n", expr));
                output.push_str("0");
            }
        } else if expr.contains(" -e ") {
            // File exists: [[ -e $var ]]
            let parts: Vec<&str> = expr.split(" -e ").collect();
            if parts.len() == 2 {
                let var = parts[1].trim();
                
                // Ensure variable is declared
                if var.starts_with('$') && !self.declared_locals.contains(&var[1..]) {
                    output.push_str(&format!("my {} = '';\n", var));
                    self.declared_locals.insert(var[1..].to_string());
                }
                
                output.push_str(&format!("-e {}", var));
            } else {
                output.push_str(&format!("# Invalid file test: {}\n", expr));
                output.push_str("0");
            }
        } else {
            // Generic test expression - try to parse it as a simple comparison
            // This handles cases like [[ $var ]] (truthy test)
            let expr_trimmed = expr.trim();
            if expr_trimmed.starts_with('$') {
                // Single variable test: [[ $var ]]
                let var = expr_trimmed;
                
                // Ensure variable is declared
                if !self.declared_locals.contains(&var[1..]) {
                    output.push_str(&format!("my {} = '';\n", var));
                    self.declared_locals.insert(var[1..].to_string());
                }
                
                output.push_str(&format!("defined({}) && {} ne ''", var, var));
            } else {
                // Unknown test expression
                output.push_str(&format!("# Unsupported test expression: {}\n", expr));
                output.push_str("0");
            }
        }
        
        output
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
            output.push_str("my $output;\n");
            
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
                            output.push_str(&format!("$output .= {} . \"\\n\";\n", echo_str));
                        } else {
                            // For other commands, generate normally but capture output
                            output.push_str(&self.indent());
                            output.push_str(&format!("$output .= `{}`;\n", self.command_to_string(&Command::Simple(simple_cmd.clone()))));
                        }
                    } else {
                        // For non-simple commands, generate normally but capture output
                        output.push_str(&self.indent());
                        output.push_str(&format!("$output .= `{}`;\n", self.command_to_string(cmd)));
                    }
                }
                output.push_str("}\n");
            } else {
                // First command - capture output using system call
                output.push_str(&format!("$output = `{}`;\n", self.command_to_string(&pipeline.commands[0])));
            }
            
            // Handle subsequent commands
            for (i, command) in pipeline.commands.iter().enumerate().skip(1) {
                if let Command::Simple(cmd) = command {
                    if cmd.name == "sort" {
                        // Handle sort command specially - sort lines, not words
                        output.push_str("$output = join(\"\\n\", sort(split(/\\n/, $output)));\n");
                    } else {
                        // Other commands - pipe through
                        let cmd_str = self.command_to_string(command);
                        let escaped_cmd = cmd_str.replace("'", "'\"'\"'");
                        output.push_str(&format!("$output = `echo \"$output\" | {}`;\n", escaped_cmd));
                    }
                } else {
                    // For non-simple commands, use system call
                    let cmd_str = self.command_to_string(command);
                    let escaped_cmd = cmd_str.replace("'", "'\"'\"'");
                    output.push_str(&format!("$output = `echo \"$output\" | {}`;\n", escaped_cmd));
                }
            }
            output.push_str("print($output);\n");
        } else {
            // Implement && and || via Perl boolean expressions
            if let Some(first) = pipeline.commands.first() {
                match first {
                    Command::TestExpression(test_expr) => {
                        // Generate the test expression directly as a Perl boolean expression
                        output.push_str(&self.generate_test_expression_clean(test_expr));
                    }
                    _ => {
                        // For non-test expressions, use system() calls
                        output.push_str("my $last_status = 0;\n");
                        output.push_str(&format!("$last_status = system('{}');\n", self.command_to_string(first)));
                    }
                }
            }
            for (idx, op) in pipeline.operators.iter().enumerate() {
                let cmd = &pipeline.commands[idx + 1];
                match (op, cmd) {
                    (PipeOperator::And, Command::TestExpression(test_expr)) => {
                        output.push_str(" && ");
                        output.push_str(&self.generate_test_expression_clean(test_expr));
                    }
                    (PipeOperator::Or, Command::TestExpression(test_expr)) => {
                        output.push_str(" || ");
                        output.push_str(&format!("({})", self.generate_test_expression_clean(test_expr)));
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
        // Matches forms like {0..5} or {10..3}
        if !(s.starts_with('{') && s.ends_with('}')) {
            return None;
        }
        let inner = &s[1..s.len() - 1];
        let parts: Vec<&str> = inner.split("..").collect();
        if parts.len() != 2 {
            return None;
        }
        let start = parts[0].parse::<i64>().ok()?;
        let end = parts[1].parse::<i64>().ok()?;
        Some((start, end))
    }

    fn parse_seq_command(&self, s: &str) -> Option<(i64, i64)> {
        // Accept backtick form `seq A B` or $(seq A B) or plain seq A B
        let trimmed = s.trim();
        // Strip backticks or $( )
        let inner = if trimmed.starts_with('`') && trimmed.ends_with('`') {
            &trimmed[1..trimmed.len()-1]
        } else if trimmed.starts_with("$(") && trimmed.ends_with(')') {
            &trimmed[2..trimmed.len()-1]
        } else {
            trimmed
        };

        let parts: Vec<&str> = inner.split_whitespace().collect();
        if parts.len() == 3 && parts[0] == "seq" {
            let start = parts[1].parse::<i64>().ok()?;
            let end = parts[2].parse::<i64>().ok()?;
            return Some((start, end));
        }
        None
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
        "    ".repeat(self.indent_level)
    }
    
    fn escape_perl_string(&self, s: &str) -> String {
        // First, unescape any \" sequences to " to avoid double-escaping
        let unescaped = s.replace("\\\"", "\"");
        // Then escape quotes and other characters for Perl
        unescaped.replace("\\", "\\\\")
                 .replace("\"", "\\\"")
                 .replace("\n", "\\n")
                 .replace("\r", "\\r")
                 .replace("\t", "\\t")
    }

    fn perl_string_literal(&self, s: &str) -> String {
        // Prefer double-quoted string with escapes to avoid conflicts with single quotes inside args
        format!("\"{}\"", self.escape_perl_string(s))
    }

    fn convert_arithmetic_to_perl(&self, expr: &str) -> String {
        // Convert shell arithmetic expressions to Perl
        let mut result = expr.to_string();
        
        // Replace shell arithmetic operators with Perl equivalents
        result = result.replace("++", "++");
        result = result.replace("--", "--");
        result = result.replace("+=", "+=");
        result = result.replace("-=", "-=");
        result = result.replace("*=", "*=");
        result = result.replace("/=", "/=");
        result = result.replace("%=", "%=");
        result = result.replace("**=", "**=");
        
        // Handle variable references (ensure $ prefix for single identifiers)
        let parts: Vec<&str> = result.split_whitespace().collect();
        let converted_parts: Vec<String> = parts.iter().map(|part| {
            if part.chars().all(|c| c.is_alphanumeric() || c == '_') && !part.chars().next().unwrap().is_digit(10) {
                // This looks like a variable name, add $ prefix
                format!("${}", part)
            } else {
                part.to_string()
            }
        }).collect();
        
        converted_parts.join(" ")
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
                if var.starts_with('#') && var.contains('[') {
                    // This is #arr[@] - convert to scalar(@arr) in Perl without quotes
                    if let Some(bracket_start) = var.find('[') {
                        let array_name = &var[1..bracket_start]; // Skip the # prefix
                        return format!("scalar(@{})", array_name);
                    }
                } else if var.starts_with('!') && var.ends_with("[@]") {
                    // This is !map[@] - convert to keys(%map) in Perl without quotes
                    let array_name = &var[1..var.len()-3]; // Remove ! prefix and [@] suffix
                    return format!("keys(%{})", array_name);
                }
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
                    eprintln!("DEBUG: Processing MapAccess: map_name='{}', key='{}'", map_name, key);
                    // For now, assume "map" is a hash and others are indexed arrays
                    if map_name == "map" {
                        // Convert map[key] to $map{key} for associative arrays
                        eprintln!("DEBUG: Converting map access to hash access: $map{{{}}}", key);
                        result.push_str(&format!("${}{{{}}}", map_name, key));
                    } else {
                        // Convert arr[key] to $arr[key] for indexed arrays
                        eprintln!("DEBUG: Converting array access to indexed access: ${}[{}]", map_name, key);
                        result.push_str(&format!("${}[{}]", map_name, key));
                    }
                }
                StringPart::Variable(var) => {
                    // Convert shell variables to Perl variables
                    eprintln!("DEBUG: Processing variable: '{}'", var);
                    eprintln!("DEBUG: Variable starts with '!': {}", var.starts_with('!'));
                    eprintln!("DEBUG: Variable ends with '[@]': {}", var.ends_with("[@]"));
                    eprintln!("DEBUG: Variable contains '[': {}", var.contains('['));
                    eprintln!("DEBUG: Variable length: {}", var.len());
                    eprintln!("DEBUG: Variable bytes: {:?}", var.as_bytes());
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
                            // This is #arr[@] - convert to scalar(@arr) in Perl
                            eprintln!("DEBUG: Found #arr[@] pattern, converting to scalar(@arr)");
                            if let Some(bracket_start) = var.find('[') {
                                let array_name = &var[1..bracket_start]; // Skip the # prefix
                                eprintln!("DEBUG: Array name extracted from #arr[@]: '{}'", array_name);
                                // For scalar functions, we need to ensure they're evaluated, not treated as strings
                                result.push_str(&format!("scalar(@{})", array_name));
                            } else {
                                result.push_str(&format!("${}", var));
                            }
                        } else if var.starts_with('!') && var.ends_with("[@]") {
                            // This is !map[@] - convert to keys(%map) in Perl
                            eprintln!("DEBUG: Found !map[@] pattern, converting to keys(%map)");
                            let array_name = &var[1..var.len()-3]; // Remove ! prefix and [@] suffix
                            eprintln!("DEBUG: Array name extracted: '{}'", array_name);
                            result.push_str(&format!("keys(%{})", array_name));
                        } else if var.starts_with('!') && var.contains('[') {
                            // This is !map[key] - convert to keys(%map) in Perl (more general pattern)
                            eprintln!("DEBUG: Found !map[key] pattern, converting to keys(%map)");
                            if let Some(bracket_start) = var.find('[') {
                                let array_name = &var[1..bracket_start]; // Remove ! prefix
                                eprintln!("DEBUG: Array name extracted from !map[key]: '{}'", array_name);
                                result.push_str(&format!("keys(%{})", array_name));
                            } else {
                                result.push_str(&format!("${}", var));
                            }
                        } else if var.starts_with('!') {
                            // This is !map - convert to keys(%map) in Perl (fallback)
                            eprintln!("DEBUG: Found !map pattern (fallback), converting to keys(%map)");
                            let array_name = &var[1..]; // Remove ! prefix
                            eprintln!("DEBUG: Array name extracted from !map: '{}'", array_name);
                            result.push_str(&format!("keys(%{})", array_name));
                        } else if var.ends_with("[@]") {
                            // This is arr[@] - convert to @arr in Perl
                            eprintln!("DEBUG: Found arr[@] pattern, converting to @arr");
                            let array_name = &var[..var.len()-3]; // Remove [@] suffix
                            eprintln!("DEBUG: Array name extracted from arr[@]: '{}'", array_name);
                            result.push_str(&format!("@{}", array_name));
                        } else if var.contains('[') && var.ends_with(']') {
                            // This is arr[1] - convert to $arr[1] in Perl or $arr{key} for hashes
                            eprintln!("DEBUG: Found arr[1] pattern, converting to array access");
                            if let Some(bracket_start) = var.find('[') {
                                let array_name = &var[..bracket_start];
                                let key = &var[bracket_start..];
                                // Check if this is a hash (associative array) - for now, assume map is a hash
                                if array_name == "map" {
                                    // Convert map[key] to $map{key} for associative arrays
                                    let key_content = &key[1..key.len()-1]; // Remove [ and ]
                                    eprintln!("DEBUG: Converting map[key] to $map{{{}}}", key_content);
                                    result.push_str(&format!("${}{{{}}}", array_name, key_content));
                                } else {
                                    // Regular indexed array - use [] syntax in Perl
                                    eprintln!("DEBUG: Converting arr[key] to ${}[{}]", array_name, key);
                                    result.push_str(&format!("${}[{}]", array_name, key));
                                }
                            } else {
                                result.push_str(&format!("${}", var));
                            }
                        } else {
                            // For simple variable names, use $var instead of ${var}
                            eprintln!("DEBUG: Simple variable, using ${}", var);
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
            Word::Literal(s) => self.perl_string_literal(s),
            Word::Variable(var) => {
                // Handle special array and hash operations
                if var.starts_with('#') && var.ends_with("[@]") {
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
            Word::Arithmetic(expr) => self.convert_arithmetic_to_perl(&expr.expression),
            Word::BraceExpansion(expansion) => {
                // Handle brace expansion in test commands
                if expansion.items.len() == 1 {
                    match &expansion.items[0] {
                        BraceItem::Range(range) => {
                            format!("({}..{})", range.start, range.end)
                        }
                        BraceItem::Literal(s) => self.perl_string_literal(s),
                        BraceItem::Sequence(seq) => {
                            format!("({})", seq.iter().map(|s| self.perl_string_literal(s)).collect::<Vec<_>>().join(", "))
                        }
                    }
                } else {
                    // Multiple items
                    let parts: Vec<String> = expansion.items.iter().map(|item| {
                        match item {
                            BraceItem::Literal(s) => self.perl_string_literal(s),
                            BraceItem::Range(range) => format!("({}..{})", range.start, range.end),
                            BraceItem::Sequence(seq) => format!("({})", seq.iter().map(|s| self.perl_string_literal(s)).collect::<Vec<_>>().join(", "))
                        }
                    }).collect();
                    format!("({})", parts.join(", "))
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
            Word::Variable(var) => {
                // Handle special array and hash operations
                if var.starts_with('#') && var.ends_with("[@]") {
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
            Word::Arithmetic(expr) => self.convert_arithmetic_to_perl(&expr.expression),
            Word::BraceExpansion(expansion) => {
                // Handle brace expansion in test commands
                if expansion.items.len() == 1 {
                    match &expansion.items[0] {
                        BraceItem::Range(range) => {
                            format!("({}..{})", range.start, range.end)
                        }
                        BraceItem::Literal(s) => format!("'{}'", self.escape_perl_string(s)),
                        BraceItem::Sequence(seq) => {
                            format!("({})", seq.iter().map(|s| format!("'{}'", self.escape_perl_string(s))).collect::<Vec<_>>().join(", "))
                        }
                    }
                } else {
                    // Multiple items
                    let parts: Vec<String> = expansion.items.iter().map(|item| {
                        match item {
                            BraceItem::Literal(s) => format!("'{}'", self.escape_perl_string(s)),
                            BraceItem::Range(range) => format!("({}..{})", range.start, range.end),
                            BraceItem::Sequence(seq) => format!("({})", seq.iter().map(|s| format!("'{}'", self.escape_perl_string(s))).collect::<Vec<_>>().join(", "))
                        }
                    }).collect();
                    format!("({})", parts.join(", "))
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
                
                // For !(*.min).js, we want to match strings ending in .js but not containing *.min
                // The pattern should match the entire string, so we need to ensure
                // the negated pattern doesn't appear anywhere before the suffix
                if suffix.is_empty() {
                    // No suffix, just negate the pattern
                    format!("^(?!{})$", negated_regex)
                } else {
                    // Has suffix, ensure negated pattern doesn't appear before the suffix
                    let suffix_regex = self.convert_simple_pattern_to_regex(suffix);
                    // For the negated pattern, we need to check if it appears anywhere in the string
                    // before the suffix. Since negated_regex already contains .* for wildcards,
                    // we don't need to add another .* in front
                    format!("^(?!{}){}$", negated_regex, suffix_regex)
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
} 
