use crate::ast::{Command, SimpleCommand, Pipeline, IfStatement, WhileLoop, ForLoop, Function};

pub struct LuaGenerator;

impl LuaGenerator {
    pub fn new() -> Self {
        LuaGenerator
    }

    pub fn generate(&self, commands: &[Command]) -> String {
        let mut lua_code = String::new();
        
        // Add necessary imports
        lua_code.push_str("-- Generated Lua code from shell script\n");
        lua_code.push_str("local os = require('os')\n");
        lua_code.push_str("local io = require('io')\n");
        lua_code.push_str("local lfs = require('lfs')\n\n");
        
        for command in commands {
            let command_code = self.generate_command(command);
            lua_code.push_str(&command_code);
            // Only add newline if the command doesn't already end with one
            if !command_code.ends_with('\n') {
                lua_code.push('\n');
            }
        }
        
        lua_code
    }

    fn generate_command(&self, command: &Command) -> String {
        match command {
            Command::Simple(cmd) => self.generate_simple_command(cmd),
            Command::Pipeline(pipeline) => self.generate_pipeline(pipeline),
            Command::If(if_stmt) => self.generate_if_statement(if_stmt),
            Command::While(while_loop) => self.generate_while_loop(while_loop),
            Command::For(for_loop) => self.generate_for_loop(for_loop),
            Command::Function(func) => self.generate_function(func),
            Command::Subshell(subshell) => self.generate_subshell(subshell),
        }
    }

    fn generate_simple_command(&self, cmd: &SimpleCommand) -> String {
        let mut lua_code = String::new();
        
        // Handle environment variables
        for (key, value) in &cmd.env_vars {
            lua_code.push_str(&format!("os.setenv('{}', '{}')\n", key, value));
        }
        
        // Handle special commands
        match cmd.name.as_str() {
            "echo" => {
                if cmd.args.is_empty() {
                    lua_code.push_str("print()\n");
                } else {
                    let args_str = cmd.args.iter()
                        .map(|arg| self.escape_lua_string(arg))
                        .collect::<Vec<_>>()
                        .join(", ");
                    lua_code.push_str(&format!("print({})\n", args_str));
                }
            }
            "cd" => {
                if cmd.args.is_empty() {
                    lua_code.push_str("os.execute('cd')\n");
                } else {
                    let dir = self.escape_lua_string(&cmd.args[0]);
                    lua_code.push_str(&format!("os.execute('cd {}')\n", dir));
                }
            }
            "ls" => {
                if cmd.args.is_empty() {
                    lua_code.push_str("for file in lfs.dir('.') do\n");
                    lua_code.push_str("    if file ~= '.' and file ~= '..' then\n");
                    lua_code.push_str("        print(file)\n");
                    lua_code.push_str("    end\n");
                    lua_code.push_str("end\n");
                } else {
                    let args = cmd.args.iter()
                        .map(|arg| self.escape_lua_string(arg))
                        .collect::<Vec<_>>()
                        .join(" ");
                    lua_code.push_str(&format!("os.execute('ls {}')\n", args));
                }
            }
            "grep" => {
                if cmd.args.len() >= 2 {
                    let pattern = self.escape_lua_string(&cmd.args[0]);
                    let file = self.escape_lua_string(&cmd.args[1]);
                    lua_code.push_str(&format!("os.execute('grep {} {}')\n", pattern, file));
                } else {
                    let args = cmd.args.iter()
                        .map(|arg| self.escape_lua_string(arg))
                        .collect::<Vec<_>>()
                        .join(" ");
                    lua_code.push_str(&format!("os.execute('grep {}')\n", args));
                }
            }
            "cat" => {
                if !cmd.args.is_empty() {
                    let file = self.escape_lua_string(&cmd.args[0]);
                    lua_code.push_str(&format!("local file = io.open({}, 'r')\n", file));
                    lua_code.push_str("if file then\n");
                    lua_code.push_str("    local content = file:read('*all')\n");
                    lua_code.push_str("    print(content)\n");
                    lua_code.push_str("    file:close()\n");
                    lua_code.push_str("end\n");
                }
            }
            "mkdir" => {
                if !cmd.args.is_empty() {
                    let dir = self.escape_lua_string(&cmd.args[0]);
                    lua_code.push_str(&format!("lfs.mkdir({})\n", dir));
                }
            }
            "rm" => {
                if !cmd.args.is_empty() {
                    let file = self.escape_lua_string(&cmd.args[0]);
                    lua_code.push_str(&format!("os.remove({})\n", file));
                }
            }
            "mv" => {
                if cmd.args.len() >= 2 {
                    let src = self.escape_lua_string(&cmd.args[0]);
                    let dst = self.escape_lua_string(&cmd.args[1]);
                    lua_code.push_str(&format!("os.rename({}, {})\n", src, dst));
                }
            }
            "cp" => {
                if cmd.args.len() >= 2 {
                    let src = self.escape_lua_string(&cmd.args[0]);
                    let dst = self.escape_lua_string(&cmd.args[1]);
                    lua_code.push_str(&format!("os.execute('cp {} {}')\n", src, dst));
                }
            }
            _ => {
                // Generic command
                let args = cmd.args.iter()
                    .map(|arg| self.escape_lua_string(arg))
                    .collect::<Vec<_>>()
                    .join(", ");
                lua_code.push_str(&format!("os.execute('{} {}')\n", cmd.name, args));
            }
        }
        
        lua_code
    }

    fn generate_pipeline(&self, pipeline: &Pipeline) -> String {
        let mut lua_code = String::new();
        
        // For now, just execute the first command
        // In a more complete implementation, you'd need to handle pipes
        if let Some(first_cmd) = pipeline.commands.first() {
            lua_code.push_str(&self.generate_command(first_cmd));
        }
        
        lua_code
    }

    fn generate_if_statement(&self, if_stmt: &IfStatement) -> String {
        let mut lua_code = String::new();
        
        // Convert shell condition to Lua condition
        let condition = self.convert_condition_to_lua(&if_stmt.condition);
        
        lua_code.push_str(&format!("if {} then\n", condition));
        lua_code.push_str(&self.generate_command(&if_stmt.then_branch));
        
        if let Some(else_branch) = &if_stmt.else_branch {
            lua_code.push_str("else\n");
            lua_code.push_str(&self.generate_command(else_branch));
        }
        
        lua_code.push_str("end\n");
        
        lua_code
    }

    fn generate_while_loop(&self, while_loop: &WhileLoop) -> String {
        let mut lua_code = String::new();
        
        let condition = self.convert_condition_to_lua(&while_loop.condition);
        
        lua_code.push_str(&format!("while {} do\n", condition));
        lua_code.push_str(&self.generate_command(&while_loop.body));
        lua_code.push_str("end\n");
        
        lua_code
    }

    fn generate_for_loop(&self, for_loop: &ForLoop) -> String {
        let mut lua_code = String::new();
        
        lua_code.push_str(&format!("for {} in {} do\n", for_loop.variable, for_loop.items.join(", ")));
        lua_code.push_str(&self.generate_command(&for_loop.body));
        lua_code.push_str("end\n");
        
        lua_code
    }

    fn generate_function(&self, func: &Function) -> String {
        let mut lua_code = String::new();
        
        lua_code.push_str(&format!("function {}(...)\n", func.name));
        lua_code.push_str(&self.generate_command(&func.body));
        lua_code.push_str("end\n");
        
        lua_code
    }

    fn generate_subshell(&self, subshell: &Command) -> String {
        let mut lua_code = String::new();
        
        lua_code.push_str("do\n");
        lua_code.push_str(&self.generate_command(subshell));
        lua_code.push_str("end\n");
        
        lua_code
    }

    fn escape_lua_string(&self, s: &str) -> String {
        if s.contains('\'') && !s.contains('"') {
            // Use double quotes if string contains single quotes
            format!("\"{}\"", s.replace("\"", "\\\""))
        } else {
            // Use single quotes by default
            format!("'{}'", s.replace("'", "\\'"))
        }
    }

    fn convert_condition_to_lua(&self, condition: &Command) -> String {
        // Convert shell test conditions to Lua
        match condition {
            Command::Simple(cmd) => {
                if cmd.name == "test" || cmd.name == "[" {
                    if cmd.args.len() >= 2 {
                        let operator = &cmd.args[0];
                        let value = &cmd.args[1];
                        
                        match operator.as_str() {
                            "-f" => format!("lfs.attributes({}) ~= nil", self.escape_lua_string(value)),
                            "-d" => format!("lfs.attributes({}, 'mode') == 'directory'", self.escape_lua_string(value)),
                            "-e" => format!("lfs.attributes({}) ~= nil", self.escape_lua_string(value)),
                            "-r" => format!("lfs.attributes({}, 'mode'):find('r') ~= nil", self.escape_lua_string(value)),
                            "-w" => format!("lfs.attributes({}, 'mode'):find('w') ~= nil", self.escape_lua_string(value)),
                            "-x" => format!("lfs.attributes({}, 'mode'):find('x') ~= nil", self.escape_lua_string(value)),
                            "-z" => format!("#{} == 0", self.escape_lua_string(value)),
                            "-n" => format!("#{} > 0", self.escape_lua_string(value)),
                            "-eq" => format!("{} == {}", self.escape_lua_string(&cmd.args[0]), self.escape_lua_string(value)),
                            "-ne" => format!("{} ~= {}", self.escape_lua_string(&cmd.args[0]), self.escape_lua_string(value)),
                            "-lt" => format!("{} < {}", self.escape_lua_string(&cmd.args[0]), self.escape_lua_string(value)),
                            "-le" => format!("{} <= {}", self.escape_lua_string(&cmd.args[0]), self.escape_lua_string(value)),
                            "-gt" => format!("{} > {}", self.escape_lua_string(&cmd.args[0]), self.escape_lua_string(value)),
                            "-ge" => format!("{} >= {}", self.escape_lua_string(&cmd.args[0]), self.escape_lua_string(value)),
                            _ => format!("os.execute('test {} {}') == 0", operator, self.escape_lua_string(value)),
                        }
                    } else {
                        "false".to_string()
                    }
                } else {
                    // For other commands, check if they succeed
                    format!("os.execute('{}') == 0", cmd.name)
                }
            }
            _ => "false".to_string(), // For non-simple commands, default to false
        }
    }
}
