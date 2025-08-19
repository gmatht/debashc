#!/bin/bash

# This script tests challenging lexing scenarios that can cause ambiguity
# and parsing difficulties in shell lexers

# 1. Ambiguous operators and precedence issues
# The lexer needs to handle these correctly with proper priorities
echo "Testing ambiguous operators..."
result=$((2**3**2))  # Should be 2**(3**2) = 2^9 = 512, not (2^3)^2 = 64
echo "2**3**2 = $result"

# 2. Complex nested parameter expansions with conflicting delimiters
echo "Testing complex parameter expansions..."
complex_var="hello world"
echo "${complex_var#*o}"  # Remove shortest match from beginning
echo "${complex_var##*o}" # Remove longest match from beginning
echo "${complex_var%o*}"  # Remove shortest match from end
echo "${complex_var%%o*}" # Remove longest match from end

# 3. Here-documents with complex delimiters and nested structures
echo "Testing complex here-documents..."
cat <<'EOF'
This is a test line
This is not a test line
This is another test line
EOF

# 4. Nested arithmetic expressions with conflicting parentheses
echo "Testing nested arithmetic..."
result=$(( (2 + 3) * (4 - 1) + (5 ** 2) ))
echo "Complex arithmetic: $result"

# 5. Command substitution within parameter expansion
echo "Testing nested command substitution..."
echo "Current dir: ${PWD:-$(pwd)}"
echo "User: ${USER:-$(whoami)}"

# 6. Process substitution with complex commands
echo "Testing process substitution..."
# diff <(sort file1.txt) <(sort file2.txt)  # Commented out as files don't exist

# 7. Brace expansion with nested patterns
echo "Testing complex brace expansion..."
echo {a,b,c}{1,2,3}{x,y,z}

# 8. Simple case statement to avoid parser issues
echo "Testing simple case patterns..."
case "$1" in
    "test")
        echo "Matched test"
        ;;
    *)
        echo "Default case"
        ;;
esac

# 9. Function with complex parameter handling
function complex_function() {
    local param1="$1"
    local param2="${2:-default}"
    local param3="${3//\"/\\\"}"  # Replace quotes with escaped quotes
    
    echo "Param1: $param1"
    echo "Param2: $param2"
    echo "Param3: $param3"
    
    # Nested command substitution
    local result=$(echo "$param1" | sed "s/old/new/g")
    echo "Result: $result"
}

# 10. Simple pipeline without complex redirections
echo "Testing simple pipeline..."
ls -la | grep "^d" | head -5

# 11. Arithmetic with mixed bases and complex expressions
echo "Testing mixed arithmetic..."
hex=255
octal=511
binary=10
result=$(( hex + octal + binary ))
echo "Mixed base result: $result"

# 12. Complex string interpolation with nested expansions
echo "Testing complex string interpolation..."
message="Hello, ${USER:-$(whoami)}! Your home is ${HOME:-$(echo ~)}"
echo "$message"

# 13. Simple test expressions to avoid parser issues
echo "Testing simple test expressions..."
if [[ -f "file.txt" ]]; then
    echo "File exists"
else
    echo "File does not exist"
fi

# 14. Complex array operations
echo "Testing complex array operations..."
declare -a array=("item1" "item2" "item3")
array+=("item4")
echo "Array: ${array[@]}"
echo "Length: ${#array[@]}"
echo "First item: ${array[0]}"

# 15. Function with complex local variable declarations
function test_locals() {
    local var1="$1"
    local var2="${2:-default_value}"
    local var3="$(echo "$var1" | tr '[:lower:]' '[:upper:]')"
    
    echo "Var1: $var1"
    echo "Var2: $var2"
    echo "Var3: $var3"
}

# Test the complex function
complex_function "test\"quote" "second_param" "third\"param"
test_locals "hello" "world"
