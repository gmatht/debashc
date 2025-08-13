// Shell script examples for the Debashc compiler
// This file contains all examples that were previously embedded in WASM

export const examples = {
  // Basic examples
  'args.sh': `#!/bin/bash

# Demonstrate argument handling
echo "Script name: $0"
echo "First argument: $1"
echo "Second argument: $2"
echo "All arguments: $@"
echo "Number of arguments: $#`,

  'simple.sh': `#!/bin/bash

# Simple shell script example
echo "Hello, World!"
echo "Current directory: $(pwd)"
echo "Current user: $USER`,

  'simple_backup.sh': `#!/bin/bash

# Simple shell script example
echo "Hello, World!"
ls
echo \`ls\`
#Lets not consider ls -la at the moment as permissions are OS dependent
#ls -la
#grep "pattern" file.txt`,

  'misc.sh': `#!/bin/bash

# Miscellaneous shell features
echo "Testing various shell features"

# Variables
name="World"
echo "Hello, $name!"

# Command substitution
current_time=$(date)
echo "Current time: $current_time"

# Arithmetic
count=5
echo "Count: $count"`,

  'subprocess.sh': `#!/bin/bash

# Demonstrate subprocess execution
echo "Running subprocess commands..."

# Simple command
ls -la

# Command with output capture
files=$(ls)
echo "Files found: $files"

# Background process
sleep 5 &
echo "Background process started with PID: $!"`,

  'test_quoted.sh': `#!/bin/bash

# Test various quoting mechanisms
echo "Single quotes: 'Hello World'"
echo "Double quotes: \"Hello World\""
echo "Backticks: \`echo Hello World\`"
echo "Dollar quotes: $'Hello World'"

# Test with variables
name="World"
echo "Hello, $name"
echo 'Hello, $name'`,

  'cat_EOF.sh': `#!/bin/bash

# Demonstrate here document
cat << EOF
This is a here document.
It can contain multiple lines.
EOF

# With variables
name="World"
cat << EOF
Hello, $name!
This is another here document.
EOF`,

  'file.txt': `This is a sample text file.
It contains multiple lines.
Used for testing file operations.`,

  'cd..sh': `#!/bin/bash

# Test directory navigation
echo "Current directory: $(pwd)"
cd ..
echo "After cd ..: $(pwd)"
cd -
echo "After cd -: $(pwd)"`,

  'test_ls_star_dot_sh.sh': `#!/bin/bash

# Test globbing patterns
echo "All .sh files:"
ls *.sh

echo "All files starting with test:"
ls test*

echo "All files ending with .sh:"
ls *.sh`,

  // Control flow examples
  'control_flow.sh': `#!/bin/bash

# Basic control flow demonstration
echo "Control flow examples"

# If statement
if [ -f "test.txt" ]; then
    echo "test.txt exists"
else
    echo "test.txt does not exist"
fi

# For loop
for i in 1 2 3 4 5; do
    echo "Number: $i"
done`,

  'control_flow_if.sh': `#!/bin/bash

# Comprehensive if statement examples
echo "If statement examples"

# Simple if
if [ $1 -gt 10 ]; then
    echo "Argument is greater than 10"
fi

# If-else
if [ -d "$1" ]; then
    echo "$1 is a directory"
else
    echo "$1 is not a directory"
fi

# If-elif-else
if [ $1 -gt 0 ]; then
    echo "Positive number"
elif [ $1 -lt 0 ]; then
    echo "Negative number"
else
    echo "Zero"
fi`,

  'control_flow_loops.sh': `#!/bin/bash

# Loop examples
echo "Loop examples"

# For loop with range
for i in {1..5}; do
    echo "For loop: $i"
done

# While loop
count=0
while [ $count -lt 3 ]; do
    echo "While loop: $count"
    count=$((count + 1))
done

# Until loop
count=0
until [ $count -ge 3 ]; do
    echo "Until loop: $count"
    count=$((count + 1))
done`,

  'control_flow_function.sh': `#!/bin/bash

# Function examples
echo "Function examples"

# Define a function
greet() {
    local name="$1"
    echo "Hello, $name!"
}

# Call the function
greet "World"
greet "User"

# Function with return value
add() {
    local a="$1"
    local b="$2"
    echo $((a + b))
}

result=$(add 5 3)
echo "5 + 3 = $result"`,

  // Pipeline examples
  'pipeline.sh': `#!/bin/bash

# Pipeline examples
echo "Pipeline examples"

# Simple pipeline
echo "hello world" | tr '[:lower:]' '[:upper:]'

# Multiple commands in pipeline
ls -la | grep "\.sh$" | wc -l

# Pipeline with variables
files=$(ls | grep "\.txt$")
echo "Text files found: $files"`,

  // Local and environment examples
  'local.sh': `#!/bin/bash

# Local and environment variable examples
echo "Variable examples"

# Local variable
local_var="local value"
echo "Local variable: $local_var"

# Export to environment
export ENV_VAR="environment value"
echo "Environment variable: $ENV_VAR"

# Read-only variable
readonly READONLY_VAR="cannot change"
echo "Read-only variable: $READONLY_VAR"`,

  // Parameter expansion examples
  'parameter_expansion.sh': `#!/bin/bash

# Basic parameter expansion
echo "Parameter expansion examples"

# Default value
echo "Name: \${name:-Unknown}"

# Required parameter
echo "Required: \${required:?Parameter required}"

# Length
text="Hello"
echo "Length of '$text': \${#text}"`,

  'parameter_expansion_advanced.sh': `#!/bin/bash

# Advanced parameter expansion
echo "Advanced parameter expansion"

# String manipulation
text="Hello World"
echo "Original: $text"
echo "Uppercase: \${text^^}"
echo "Lowercase: \${text,,}"
echo "First char upper: \${text^}"`,

  'parameter_expansion_case.sh': `#!/bin/bash

# Case-based parameter expansion
echo "Case-based parameter expansion"

# Convert to uppercase
text="hello world"
echo "Uppercase: \${text^^}"

# Convert to lowercase
text="HELLO WORLD"
echo "Lowercase: \${text,,}"

# Capitalize first letter
text="hello world"
echo "Capitalized: \${text^}"`,

  'parameter_expansion_defaults.sh': `#!/bin/bash

# Default value parameter expansion
echo "Default value examples"

# Use default if empty
echo "Name: \${name:-Default Name}"

# Use default if unset
echo "Path: \${PATH:-/usr/bin}"

# Assign default if empty
echo "Before: $var"
echo "Default: \${var:=new_value}"
echo "After: $var"`,

  'parameter_expansion_more.sh': `#!/bin/bash

# More parameter expansion features
echo "More parameter expansion"

# Remove prefix
path="/usr/local/bin/script.sh"
echo "Path: $path"
echo "Without prefix: \${path#/usr/local/}"

# Remove suffix
echo "Without suffix: \${path%.sh}"

# Substring
text="Hello World"
echo "Substring: \${text:6:5}"`,

  // Brace expansion examples
  'brace_expansion.sh': `#!/bin/bash

# Brace expansion examples
echo "Brace expansion examples"

# Simple sequence
echo {1..5}

# With step
echo {0..10..2}

# Character sequence
echo {a..e}

# Multiple braces
echo {a,b,c}{1,2,3}`,

  'brace_expansion_basic.sh': `#!/bin/bash

# Basic brace expansion
echo "Basic brace expansion"

# Number sequence
for i in {1..5}; do
    echo "Number: $i"
done

# Character sequence
for c in {a..e}; do
    echo "Letter: $c"
done

# Step sequence
for i in {0..10..2}; do
    echo "Even: $i"
done`,

  'brace_expansion_advanced.sh': `#!/bin/bash

# Advanced brace expansion
echo "Advanced brace expansion"

# Nested braces
echo {a,b,c}{1,2,3}

# With text
echo prefix_{a,b,c}_suffix

# Complex patterns
echo {a..c}{1..3}{x,y}`,

  'brace_expansion_practical.sh': `#!/bin/bash

# Practical brace expansion examples
echo "Practical brace expansion"

# Create multiple files
touch file_{1..3}.txt

# Backup files
for file in *.txt; do
    cp "$file" "\${file%.txt}.bak"
done

# Multiple directories
mkdir -p dir_{a,b,c}`,

  // Arrays examples
  'arrays.sh': `#!/bin/bash

# Basic array examples
echo "Array examples"

# Declare array
declare -a fruits=("apple" "banana" "cherry")

# Access elements
echo "First fruit: \${fruits[0]}"
echo "All fruits: \${fruits[@]}"

# Array length
echo "Number of fruits: \${#fruits[@]}"`,

  'arrays_indexed.sh': `#!/bin/bash

# Indexed array examples
echo "Indexed array examples"

# Create array
numbers=(1 2 3 4 5)

# Loop through array
for i in "\${!numbers[@]}"; do
    echo "Index $i: \${numbers[i]}"
done

# Add element
numbers+=(6)
echo "After adding: \${numbers[@]}"`,

  'arrays_associative.sh': `#!/bin/bash

# Associative array examples
echo "Associative array examples"

# Declare associative array
declare -A person
person["name"]="John"
person["age"]="30"
person["city"]="New York"

# Access elements
echo "Name: \${person["name"]}"
echo "Age: \${person["age"]}"
echo "City: \${person["city"]}"`,

  // Pattern matching examples
  'pattern_matching.sh': `#!/bin/bash

# Pattern matching examples
echo "Pattern matching examples"

# Basic globbing
for file in *.sh; do
    echo "Shell script: $file"
done

# Extended globbing
shopt -s extglob
for file in *@(.sh|.txt); do
    echo "Shell or text file: $file"
done`,

  'pattern_matching_basic.sh': `#!/bin/bash

# Basic pattern matching
echo "Basic pattern matching"

# Simple glob
echo "Shell scripts:"
ls *.sh

# Question mark
echo "Files with 3 characters:"
ls ???

# Square brackets
echo "Files starting with a or b:"
ls [ab]*`,

  'pattern_matching_extglob.sh': `#!/bin/bash

# Extended globbing examples
echo "Extended globbing examples"

# Enable extended globbing
shopt -s extglob

# Zero or more occurrences
echo "Files with .sh extension:"
ls *@(.sh)

# One or more occurrences
echo "Files with .txt extension:"
ls +(.txt)

# Negation
echo "Files not ending in .sh:"
ls !(*.sh)`,

  'pattern_matching_nocase.sh': `#!/bin/bash

# Case-insensitive pattern matching
echo "Case-insensitive pattern matching"

# Enable nocaseglob
shopt -s nocaseglob

# Match regardless of case
echo "Files ending in .TXT (case insensitive):"
ls *.txt

# Disable nocaseglob
shopt -u nocaseglob`,

  // Process substitution examples
  'process_substitution.sh': `#!/bin/bash

# Process substitution examples
echo "Process substitution examples"

# Input process substitution
while read line; do
    echo "Read: $line"
done < <(echo "Hello"; echo "World")

# Output process substitution
echo "Output to file" > >(tee output.txt)`,

  'process_substitution_advanced.sh': `#!/bin/bash

# Advanced process substitution
echo "Advanced process substitution"

# Compare two processes
diff <(sort file1.txt) <(sort file2.txt)

# Multiple process substitutions
paste <(cut -d: -f1 /etc/passwd) <(cut -d: -f3 /etc/passwd)`,

  'process_substitution_comm.sh': `#!/bin/bash

# Process substitution with comm
echo "Process substitution with comm"

# Compare sorted lists
comm <(sort list1.txt) <(sort list2.txt)

# Show only common lines
comm -12 <(sort list1.txt) <(sort list2.txt)`,

  'process_substitution_mapfile.sh': `#!/bin/bash

# Process substitution with mapfile
echo "Process substitution with mapfile"

# Read lines into array
mapfile -t lines < <(echo "Line 1"; echo "Line 2"; echo "Line 3")

# Display array
for line in "\${lines[@]}"; do
    echo "Array line: $line"
done`,

  'process_substitution_here.sh': `#!/bin/bash

# Process substitution with here documents
echo "Process substitution with here documents"

# Use here document in process substitution
while read line; do
    echo "Processed: $line"
done < <(cat << 'EOF'
First line
Second line
Third line
EOF
)`,

  // ANSI quoting examples
  'ansi_quoting.sh': `#!/bin/bash

# ANSI quoting examples
echo "ANSI quoting examples"

# Basic ANSI quoting
echo $'Hello\nWorld'

# With escape sequences
echo $'Tab:\tText\nNew line'

# Unicode support
echo $'Unicode: \u0048\u0065\u006C\u006C\u006F'`,

  'ansi_quoting_basic.sh': `#!/bin/bash

# Basic ANSI quoting
echo "Basic ANSI quoting"

# Newline
echo $'Line 1\nLine 2'

# Tab
echo $'Column1\tColumn2\tColumn3'

# Bell
echo $'Alert\a'`,

  'ansi_quoting_escape.sh': `#!/bin/bash

# ANSI quoting with escape sequences
echo "ANSI quoting with escape sequences"

# Common escape sequences
echo $'Backspace:\bText'
echo $'Carriage return:\rNew text'
echo $'Vertical tab:\vText'`,

  'ansi_quoting_practical.sh': `#!/bin/bash

# Practical ANSI quoting examples
echo "Practical ANSI quoting examples"

# Color output
echo $'\\033[31mRed text\\033[0m'
echo $'\\033[32mGreen text\\033[0m'
echo $'\\033[34mBlue text\\033[0m'

# Cursor movement
echo $'\\033[2J\\033[H'  # Clear screen and move to top`,

  'ansi_quoting_unicode.sh': `#!/bin/bash

# ANSI quoting with Unicode
echo "ANSI quoting with Unicode examples"

# Basic Unicode
echo $'Hello: \u0048\u0065\u006C\u006C\u006F'

# Emoji support
echo $'Smile: \U0001F604'

# Special characters
echo $'Euro: \u20AC'`,

  // Grep examples
  'grep_basic.sh': `#!/bin/bash

# Basic grep examples
echo "Basic grep examples"

# Simple search
echo "Searching for 'hello' in files:"
grep "hello" *.txt

# Case insensitive
echo "Case insensitive search:"
grep -i "HELLO" *.txt

# Show line numbers
echo "With line numbers:"
grep -n "hello" *.txt`,

  'grep_advanced.sh': `#!/bin/bash

# Advanced grep examples
echo "Advanced grep examples"

# Regular expressions
echo "Using regex:"
grep -E "^[A-Z]" *.txt

# Invert match
echo "Lines not containing 'hello':"
grep -v "hello" *.txt

# Count matches
echo "Number of matches:"
grep -c "hello" *.txt`,

  'grep_context.sh': `#!/bin/bash

# Grep with context
echo "Grep with context examples"

# Show context lines
echo "With 2 lines before and after:"
grep -A 2 -B 2 "hello" *.txt

# Show only context
echo "Only context lines:"
grep -C 1 "hello" *.txt`,

  'grep_params.sh': `#!/bin/bash

# Grep with parameters
echo "Grep with parameters examples"

# Recursive search
echo "Recursive search:"
grep -r "hello" .

# Include/exclude patterns
echo "Only .txt files:"
grep -r --include="*.txt" "hello" .`,

  'grep_regex.sh': `#!/bin/bash

# Grep with regular expressions
echo "Grep with regular expressions"

# Extended regex
echo "Extended regex search:"
grep -E "(hello|world)" *.txt

# Perl-compatible regex
echo "Perl-compatible regex:"
grep -P "\\b\\w+\\b" *.txt`
};

// Helper function to get all example names
export function getExampleNames() {
  return Object.keys(examples);
}

// Helper function to get example by name
export function getExample(name) {
  return examples[name] || null;
}

// Helper function to get examples grouped by category
export function getExamplesByCategory() {
  const categories = {
    'Basic Examples': ['args.sh', 'simple.sh', 'simple_backup.sh', 'misc.sh', 'subprocess.sh', 'test_quoted.sh', 'cat_EOF.sh', 'file.txt', 'cd..sh', 'test_ls_star_dot_sh.sh'],
    'Control Flow': ['control_flow.sh', 'control_flow_if.sh', 'control_flow_loops.sh', 'control_flow_function.sh'],
    'Pipelines': ['pipeline.sh'],
    'Variables': ['local.sh'],
    'Parameter Expansion': ['parameter_expansion.sh', 'parameter_expansion_advanced.sh', 'parameter_expansion_case.sh', 'parameter_expansion_defaults.sh', 'parameter_expansion_more.sh'],
    'Brace Expansion': ['brace_expansion.sh', 'brace_expansion_basic.sh', 'brace_expansion_advanced.sh', 'brace_expansion_practical.sh'],
    'Arrays': ['arrays.sh', 'arrays_indexed.sh', 'arrays_associative.sh'],
    'Pattern Matching': ['pattern_matching.sh', 'pattern_matching_basic.sh', 'pattern_matching_extglob.sh', 'pattern_matching_nocase.sh'],
    'Process Substitution': ['process_substitution.sh', 'process_substitution_advanced.sh', 'process_substitution_comm.sh', 'process_substitution_mapfile.sh', 'process_substitution_here.sh'],
    'ANSI Quoting': ['ansi_quoting.sh', 'ansi_quoting_basic.sh', 'ansi_quoting_escape.sh', 'ansi_quoting_practical.sh', 'ansi_quoting_unicode.sh'],
    'Grep Examples': ['grep_basic.sh', 'grep_advanced.sh', 'grep_context.sh', 'grep_params.sh', 'grep_regex.sh']
  };
  
  return categories;
}

// Helper function to get examples as JSON (for compatibility with existing code)
export function examplesJson() {
  return JSON.stringify(Object.entries(examples).map(([name, content]) => ({
    name,
    content
  })));
}
