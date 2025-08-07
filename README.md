# Debashc - Shell Script Converter

A comprehensive Rust library and command-line tool for parsing and converting shell/bash scripts to other programming languages. This project provides a robust foundation for analyzing shell scripts, converting them to Perl, Rust, Python, and more, or building shell script analysis tools.

## Features

- **Complete Lexer**: Tokenizes shell/bash scripts with support for all major shell constructs
- **AST Parser**: Converts tokens into a structured Abstract Syntax Tree
- **Multi-language Code Generation**: Convert shell scripts to Perl, Rust, and Python
- **Web Assembly Support**: Run in the browser with a beautiful web interface
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
git clone https://github.com/yourusername/debashc.git
cd debashc
cargo build --release
```

## Usage

### Command Line Interface

The `debashc` binary provides several commands for analyzing and converting shell scripts:

```bash
# Tokenize a shell script
debashc lex "echo hello world"

# Parse a shell script to AST
debashc parse "ls | grep test"

# Parse a shell script from file
debashc file examples/simple.sh

# Convert shell script to Perl
debashc parse --perl "ls | grep test"

# Convert shell script to Rust
debashc parse --rust "ls | grep test"

# Convert shell script to Python
debashc parse --python "ls | grep test"

# Convert shell script file to Perl
debashc file --perl examples/simple.sh

# Interactive mode
debashc interactive
```

### Library Usage

```rust
use debashc::{Lexer, Parser, PerlGenerator, RustGenerator, PythonGenerator};

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

// Convert to Perl
let mut generator = PerlGenerator::new();
let perl_code = generator.generate(&commands);
println!("Perl code: {}", perl_code);
```

## Web Interface

Debashc includes a beautiful web interface powered by WebAssembly. You can run shell script conversions directly in your browser!

### Building and Running the Web Demo

```bash
# Build the WASM target
./build-wasm.sh

# Or manually:
cargo install wasm-pack
wasm-pack build --target web --out-dir www/pkg

# Serve the web interface
cd www
python3 -m http.server 8000
# Then open http://localhost:8000 in your browser
```

The web interface provides:
- Real-time shell script tokenization
- AST visualization
- Conversion to Perl, Rust, and Python
- Interactive examples
- Responsive design for mobile and desktop

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

## Shell to Perl Conversion

The `--perl` flag allows you to convert shell scripts to equivalent Perl code:

### Basic Command Conversion
```bash
# Shell script
echo "Hello, World!"

# Convert to Perl
sh2perl parse --perl 'echo "Hello, World!"'
```

Output:
```perl
print "Hello, World!\n";
```

### Pipeline Conversion
```bash
# Shell script
ls | grep "\.txt$" | wc -l

# Convert to Perl
sh2perl parse --perl 'ls | grep "\.txt$" | wc -l'
```

Output:
```perl
my $output;
$output = `ls`;
$output = `echo "$output" | grep "\.txt$"`;
print $output;
```

### Control Flow Conversion
```bash
# Shell script
if [ -f file.txt ]; then
    echo "File exists"
else
    echo "File does not exist"
fi

# Convert to Perl
sh2perl parse --perl 'if [ -f file.txt ]; then echo "File exists"; else echo "File does not exist"; fi'
```

Output:
```perl
if (-f 'file.txt') {
    print "File exists\n";
} else {
    print "File does not exist\n";
}
```

### File Operations
```bash
# Shell script
mkdir newdir
cp file.txt newdir/
rm oldfile.txt

# Convert to Perl
sh2perl parse --perl 'mkdir newdir; cp file.txt newdir/; rm oldfile.txt'
```

Output:
```perl
mkdir('newdir') or die "Cannot create directory: $!\n";
use File::Copy;
copy('file.txt', 'newdir/') or die "Cannot copy file: $!\n";
unlink('oldfile.txt') or die "Cannot remove file: $!\n";
```

### Converting Shell Script Files

You can also convert entire shell script files to Perl:

```bash
# Create a shell script file
cat > example.sh << 'EOF'
#!/bin/bash
if [ -d /tmp ]; then
    echo "Directory exists"
    ls /tmp | head -5
else
    echo "Directory does not exist"
fi
EOF

# Convert the file to Perl
sh2perl file --perl example.sh
```

Output:
```perl
if (-d '/tmp') {
    print "Directory exists\n";
    opendir(my $dh, '/tmp') or die "Cannot open directory: $!\n";
    while (my $file = readdir($dh)) {
        print "$file\n" unless $file =~ /^\.\.?$/;
    }
    closedir($dh);
} else {
    print "Directory does not exist\n";
}
```

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

