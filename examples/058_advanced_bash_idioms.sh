#!/bin/bash

# Advanced Bash Idioms: Nesting and Combining Control Blocks
# This file demonstrates complex bash patterns and idioms

echo "=== Advanced Bash Idioms Examples ==="
echo

# Example 1: Nested loops with conditional logic and array manipulation
echo "1. Nested loops with conditional logic and array manipulation:"
numbers=(1 2 3 4 5)
letters=(a b c d e)
for num in "${numbers[@]}"; do
    for letter in "${letters[@]}"; do
        if [[ $num -gt 3 && $letter != "c" ]]; then
            echo "  Number $num with letter $letter (filtered)"
        fi
    done
done
echo

# Example 2: Function with nested case statements and parameter expansion
echo "2. Function with nested case statements and parameter expansion:"
process_data() {
    local data_type="$1"
    local value="$2"
    
    case "$data_type" in
        "string")
            case "${value,,}" in  # Convert to lowercase
                "hello"|"hi")
                    echo "  Greeting detected: $value"
                    ;;
                "bye"|"goodbye")
                    echo "  Farewell detected: $value"
                    ;;
                *)
                    echo "  Unknown string: $value"
                    ;;
            esac
            ;;
        "number")
            if [[ "$value" =~ ^[0-9]+$ ]]; then
                if (( value % 2 == 0 )); then
                    echo "  Even number: $value"
                else
                    echo "  Odd number: $value"
                fi
            else
                echo "  Invalid number: $value"
            fi
            ;;
        *)
            echo "  Unknown data type: $data_type"
            ;;
    esac
}

process_data "string" "Hello"
process_data "string" "Bye"
process_data "number" "42"
process_data "number" "17"
echo

# Example 3: Complex conditional with command substitution and arithmetic
echo "3. Complex conditional with command substitution and arithmetic:"
file_count=$(find . -maxdepth 1 -type f | wc -l)
dir_count=$(find . -maxdepth 1 -type d | wc -l)

if [[ $file_count -gt 0 && $dir_count -gt 1 ]]; then
    if (( file_count > dir_count )); then
        echo "  More files ($file_count) than directories ($dir_count)"
    elif (( file_count == dir_count )); then
        echo "  Equal count: $file_count files and $dir_count directories"
    else
        echo "  More directories ($dir_count) than files ($file_count)"
    fi
else
    echo "  Insufficient items for comparison"
fi
echo

# Example 4: Nested here-documents with parameter expansion
echo "4. Nested here-documents with parameter expansion:"
user="admin"
host="localhost"
port="22"

cat <<-EOF
    SSH Configuration:
    $(cat <<-INNER
        User: $user
        Host: $host
        Port: $port
        Status: $(ping -c 1 $host >/dev/null 2>&1 && echo "Online" || echo "Offline")
    INNER
    )
EOF
echo

# Example 5: Array processing with nested loops and conditional logic
echo "5. Array processing with nested loops and conditional logic:"
declare -A matrix
matrix[0,0]=1; matrix[0,1]=2; matrix[0,2]=3
matrix[1,0]=4; matrix[1,1]=5; matrix[1,2]=6
matrix[2,0]=7; matrix[2,1]=8; matrix[2,2]=9

for i in {0..2}; do
    for j in {0..2}; do
        value=${matrix[$i,$j]}
        if [[ $value -gt 5 ]]; then
            echo -n "  [$value] "
        else
            echo -n "  $value "
        fi
    done
    echo
done
echo

# Example 6: Process substitution with nested commands and error handling
echo "6. Process substitution with nested commands and error handling:"
{
    echo "Processing files..."
    while IFS= read -r line; do
        if [[ -f "$line" ]]; then
            size=$(stat -c%s "$line" 2>/dev/null || echo "unknown")
            echo "  File: $line (size: $size bytes)"
        elif [[ -d "$line" ]]; then
            count=$(find "$line" -maxdepth 1 -type f 2>/dev/null | wc -l)
            echo "  Directory: $line ($count files)"
        else
            echo "  Unknown: $line"
        fi
    done < <(find . -maxdepth 1 -name "*.sh" | head -5)
} 2>/dev/null || echo "  Error occurred during processing"
echo

# Example 7: Function composition with local variables and nested scopes
echo "7. Function composition with local variables and nested scopes:"
outer_function() {
    local outer_var="outer"
    echo "  Outer function: $outer_var"
    
    inner_function() {
        local inner_var="inner"
        echo "    Inner function: $inner_var, accessing outer: $outer_var"
        
        deepest_function() {
            local deepest_var="deepest"
            echo "      Deepest function: $deepest_var, inner: $inner_var, outer: $outer_var"
        }
        
        deepest_function
    }
    
    inner_function
}

outer_function
echo

# Example 8: Complex pipeline with conditional processing
echo "8. Complex pipeline with conditional processing:"
echo "Processing text with conditional filters:"
cat <<'TEXT' | while IFS= read -r line; do
    if [[ "$line" =~ ^[A-Z] ]]; then
        echo "  Uppercase start: $line" | tr '[:lower:]' '[:upper:]'
    elif [[ "$line" =~ [0-9] ]]; then
        echo "  Contains number: $line" | sed 's/[0-9]/#/g'
    else
        echo "  Other: $line" | rev
    fi
done
Hello World
test123
lowercase
TEXT
echo

# Example 9: Error handling with nested traps and cleanup
echo "9. Error handling with nested traps and cleanup:"
cleanup() {
    echo "  Cleaning up..."
    rm -f /tmp/temp_file_*
}

trap cleanup EXIT

nested_error_handling() {
    local temp_file="/tmp/temp_file_$$"
    
    # Create temporary file
    echo "Creating temporary file: $temp_file"
    echo "test data" > "$temp_file"
    
    # Simulate nested operations
    if [[ -f "$temp_file" ]]; then
        echo "  File exists, processing..."
        
        # Nested condition with error handling
        if [[ -s "$temp_file" ]]; then
            echo "  File has content, reading..."
            content=$(cat "$temp_file")
            echo "  Content: $content"
        else
            echo "  File is empty"
        fi
        
        # Clean up this specific file
        rm -f "$temp_file"
        echo "  Temporary file removed"
    fi
}

nested_error_handling
echo

# Example 10: Advanced parameter expansion with nested operations
echo "10. Advanced parameter expansion with nested operations:"
test_string="Hello World 123"
echo "  Original: '$test_string'"
echo "  Length: ${#test_string}"
echo "  First word: ${test_string%% *}"
echo "  Last word: ${test_string##* }"
echo "  Middle: ${test_string#* }"
echo "  Middle: ${test_string% *}"
echo "  Uppercase: ${test_string^^}"
echo "  Lowercase: ${test_string,,}"
echo "  Capitalize: ${test_string^}"
echo

# Example 11: Complex arithmetic with nested expressions
echo "11. Complex arithmetic with nested expressions:"
a=10
b=5
c=3

result=$(( (a + b) * c - (a % b) / c ))
echo "  Expression: (a + b) * c - (a % b) / c"
echo "  Values: a=$a, b=$b, c=$c"
echo "  Result: $result"

# Nested arithmetic in conditional
if (( (a > b) && (b < c) || (a % 2 == 0) )); then
    echo "  Complex condition met: a > b AND (b < c OR a is even)"
fi
echo

# Example 12: Nested command substitution with error handling
echo "12. Nested command substitution with error handling:"
echo "  Current directory: $(pwd)"
echo "  Parent directory: $(dirname "$(pwd)")"
echo "  Home directory: $(dirname "$(dirname "$(pwd)")")"

# Nested command with fallback
file_info=$(stat -c "%s %y" "nonexistent_file" 2>/dev/null || echo "File not found")
echo "  File info: $file_info"
echo

echo "=== Advanced Bash Idioms Examples Complete ==="
