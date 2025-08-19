#!/bin/bash

# This script combines multiple complex bash features that are challenging to parse and generate
# It tests the limits of the bash-to-perl converter

# 1. Complex nested subshells with process substitution
diff <(sort <(grep -v "^#" /etc/passwd | cut -d: -f1)) <(sort <(grep -v "^#" /etc/group | cut -d: -f1))

# 2. Nested brace expansions with ranges and sequences
echo "Files: " file_{a..z}_{1..10,20,30..40}.{txt,log,dat}

# 3. Complex parameter expansion with nested substitutions
name="John Doe"
echo "Hello ${name// /_}"  # Replace spaces with underscores
echo "Length: ${#name}"    # String length
echo "First: ${name:0:4}"  # Substring
echo "Last: ${name: -3}"   # Last 3 characters

# 4. Extended glob patterns with shopt
shopt -s extglob
shopt -s nocasematch

# 5. Complex case statement with pattern matching
case "$1" in
    [a-z]*) echo "Lowercase start";;
    [A-Z]*) echo "Uppercase start";;
    [0-9]*) echo "Number start";;
    ?) echo "Single character";;
    *) echo "Something else";;
esac

# 6. Nested arithmetic expressions
((i = 1 + (2 * 3) / 4))
((j = i++ + ++i))
echo "i=$i, j=$j"

# 7. Complex array operations with associative arrays
declare -A config
config["user"]="admin"
config["host"]="localhost"
config["port"]="8080"

# 8. Here-documents with variable interpolation
cat <<'EOF' > config.txt
User: $USER
Host: ${HOSTNAME:-localhost}
Path: $PWD
EOF

# 9. Process substitution in pipeline with complex commands
paste <(cut -d: -f1 /etc/passwd | sort) <(cut -d: -f3 /etc/passwd | sort -n) | head -10

# 10. Nested function definitions with local variables
outer_func() {
    local outer_var="outer"
    
    inner_func() {
        local inner_var="inner"
        echo "Outer: $outer_var, Inner: $inner_var"
        
        # Nested arithmetic
        ((result = outer_var + inner_var))
        echo "Result: $result"
    }
    
    inner_func
}

# 11. Complex test expressions with extended operators
if [[ "$1" =~ ^[0-9]+$ ]] && [[ "$2" == "test" || "$2" == "debug" ]]; then
    echo "Valid input"
fi

# 12. Brace expansion with nested sequences
mkdir -p project/{src/{main,test}/{java,resources},docs/{api,user},build/{classes,lib}}

# 13. Complex string manipulation with parameter expansion
filename="my_file.txt"
basename="${filename%.*}"           # Remove extension
extension="${filename##*.}"         # Get extension
uppercase="${filename^^}"           # Convert to uppercase
lowercase="${filename,,}"           # Convert to lowercase

# 14. Nested command substitution with arithmetic
result=$(echo $(( $(wc -l < /etc/passwd) + $(wc -l < /etc/group) )))

# 15. Complex pipeline with multiple redirects
grep -v "^#" /etc/passwd | cut -d: -f1,3 | sort -t: -k2 -n | head -5 > users.txt 2> errors.log

# 16. Function with complex argument handling
process_files() {
    local -a files=("$@")
    local count=0
    
    for file in "${files[@]}"; do
        if [[ -f "$file" ]]; then
            ((count++))
            echo "Processing: $file"
        fi
    done
    
    echo "Total files processed: $count"
}

# 17. Complex while loop with nested conditionals
while IFS=: read -r user pass uid gid info home shell; do
    if [[ "$uid" -gt 1000 ]] && [[ "$shell" != "/bin/false" ]]; then
        if [[ "$home" =~ ^/home/ ]]; then
            echo "User: $user (UID: $uid) - $home"
        fi
    fi
done < /etc/passwd

# 18. Array slicing and manipulation
numbers=(1 2 3 4 5 6 7 8 9 10)
middle=("${numbers[@]:3:4}")        # Elements 4-7
first_half=("${numbers[@]:0:5}")   # First 5 elements
last_half=("${numbers[@]:5}")      # Last 5 elements

# 19. Complex pattern matching with extended globs
for file in *.{txt,log,dat}; do
    case "$file" in
        @(*.txt|*.log)) echo "Text file: $file";;
        *.dat) echo "Data file: $file";;
        *) echo "Other file: $file";;
    esac
done

# 20. Nested subshells with environment variables
(
    export DEBUG=1
    export LOG_LEVEL=verbose
    
    (
        unset DEBUG
        echo "Inner: LOG_LEVEL=$LOG_LEVEL, DEBUG=${DEBUG:-unset}"
    )
    
    echo "Outer: LOG_LEVEL=$LOG_LEVEL, DEBUG=$DEBUG"
)

# 21. Complex string interpolation with multiple variables
message="Hello ${USER:-guest} from ${HOSTNAME:-localhost}"
echo "$message"

# 22. Function returning complex data structures
get_system_info() {
    local -A info
    info["os"]="$(uname -s)"
    info["arch"]="$(uname -m)"
    info["hostname"]="$(hostname)"
    info["user"]="$USER"
    
    # Return as associative array (bash 4+)
    declare -p info
}

# 23. Complex error handling with traps
trap 'echo "Error on line $LINENO"; exit 1' ERR
trap 'echo "Cleaning up..."; rm -f /tmp/temp_*' EXIT

# 24. Advanced parameter expansion with default values and transformations
input="${1:-default_value}"
sanitized="${input//[^a-zA-Z0-9]/_}"
uppercase="${sanitized^^}"
echo "Input: '$input' -> Sanitized: '$sanitized' -> Uppercase: '$uppercase'"

# 25. Complex command chaining with logical operators
[[ -f "$1" ]] && echo "File exists" || echo "File not found"
[[ -d "$2" ]] && cd "$2" && pwd || echo "Directory not accessible"

echo "Script completed successfully!"
