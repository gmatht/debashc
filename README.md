# sh2perl - Shell Script Parser in Rust

A comprehensive Rust library and command-line tool for parsing and lexing shell/bash scripts. This project provides a robust foundation for analyzing shell scripts, converting them to other formats, or building shell script analysis tools.

## Features

- **Complete Lexer**: Tokenizes shell/bash scripts with support for all major shell constructs
- **AST Parser**: Converts tokens into a structured Abstract Syntax Tree
- **Shell Constructs Support**:
  - Simple commands and arguments
  - Pipelines and redirections
  - Control flow (if/else, loops, functions)
  - Variable expansions
  - Command substitutions
  - Arithmetic expressions
  - File test operators
  - Comments and whitespace

## Installation

```bash
git clone https://github.com/yourusername/sh2perl.git
cd sh2perl
cargo build --release
```

## Usage

### Command Line Interface

The `sh2perl` binary provides several commands for analyzing shell scripts:

```bash
# Tokenize a shell script
sh2perl lex "echo hello world"

# Parse a shell script to AST
sh2perl parse "ls | grep test"

# Parse a shell script from file
sh2perl file examples/simple.sh

# Interactive mode
sh2perl interactive
```

### Library Usage

```rust
use sh2perl::{Lexer, Parser};

// Tokenize a shell script
let input = "echo hello world";
let mut lexer = Lexer::new(input);
while let Some(token) = lexer.next() {
    println!("Token: {:?}", token);
}

// Parse a shell script
let mut parser = Parser::new(input);
match parser.parse() {
    Ok(commands) => {
        for command in commands {
            println!("Command: {:?}", command);
        }
    }
    Err(e) => eprintln!("Parse error: {}", e),
}
```

## Examples

### Simple Command
```bash
echo "Hello, World!"
```
Parses to a `SimpleCommand` with name "echo" and arguments.

### Pipeline
```bash
ls | grep "\.txt$" | wc -l
```
Parses to a `Pipeline` with multiple commands connected by pipe operators.

### Control Flow
```bash
if [ -f file.txt ]; then
    echo "File exists"
else
    echo "File does not exist"
fi
```
Parses to an `IfStatement` with condition, then branch, and else branch.

### Function Definition
```bash
function greet() {
    echo "Hello, $1!"
}
```
Parses to a `Function` with name and body.

## Project Structure

```
sh2perl/
├── src/
│   ├── lib.rs          # Library entry point
│   ├── main.rs         # Binary entry point
│   ├── lexer.rs        # Tokenizer implementation
│   ├── parser.rs       # Parser implementation
│   └── ast.rs          # Abstract Syntax Tree definitions
├── examples/           # Example shell scripts
├── tests/              # Test files
└── Cargo.toml          # Project configuration
```

## API Documentation

### Lexer

The `Lexer` struct provides tokenization of shell scripts:

```rust
pub struct Lexer {
    tokens: Vec<(Token, usize, usize)>,
    current: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self;
    pub fn next(&mut self) -> Option<&Token>;
    pub fn peek(&self) -> Option<&Token>;
    pub fn is_eof(&self) -> bool;
}
```

### Parser

The `Parser` struct converts tokens into an AST:

```rust
pub struct Parser {
    lexer: Lexer,
}

impl Parser {
    pub fn new(input: &str) -> Self;
    pub fn parse(&mut self) -> Result<Vec<Command>, ParserError>;
}
```

### AST Nodes

The AST is composed of various command types:

- `SimpleCommand`: Basic commands with arguments and redirections
- `Pipeline`: Multiple commands connected by operators
- `IfStatement`: Conditional execution
- `WhileLoop`: Loop constructs
- `ForLoop`: Iteration over lists
- `Function`: Function definitions
- `Subshell`: Commands in subshells

## Testing

Run the test suite:

```bash
cargo test
```

Run specific test categories:

```bash
cargo test lexer
cargo test parser
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Run the test suite
6. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Roadmap

- [ ] Support for more shell features (case statements, arrays)
- [ ] Better error reporting with line numbers
- [ ] AST pretty printing
- [ ] Shell script to other language converters
- [ ] Static analysis tools
- [ ] Performance optimizations

