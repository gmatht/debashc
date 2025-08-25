#!/bin/bash

# 11. While loop with complex condition and nested commands
while IFS= read -r line && [ -n "$line" ] && (( counter < max_lines )); do
    if [[ "$line" =~ ^[[:space:]]*# ]]; then
        continue
    fi
    
    case "$line" in
        *\$\(*\)*)
            echo "Contains command substitution: $line"
            ;;
        *\$\{[^}]*\}*)
            echo "Contains parameter expansion: $line"
            ;;
        *\$\(\(*\)\)*)
            echo "Contains arithmetic expansion: $line"
            ;;
    esac
    
    (( counter++ ))
done < <(grep -v "^#" "$input_file" | head -n "$max_lines")
