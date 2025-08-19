#!/bin/bash

# This file contains bash constructs that are particularly challenging to parse
# due to complex nesting, ambiguous syntax, and edge cases

# 1. Deeply nested arithmetic expressions with mixed operators
result=$(( (a + b) * (c - d) / (e % f) + (g ** h) - (i << j) | (k & l) ^ (m | n) ))

# 2. Complex array assignments with nested expansions
declare -A matrix
matrix[0,0]=$(( (x + y) * z ))
matrix[1,1]=${array[${index}]}
matrix[2,2]=${!prefix@}
matrix[3,3]=${#array[@]}

# 3. Nested command substitutions with complex quoting
output=$(echo "Result: $(echo "Nested: $(echo "Deep: $(echo "Level 4")")")")

# 4. Complex parameter expansion with nested braces
echo "${var:-${default:-${fallback:-$(echo "computed")}}}"
echo "${array[${index}]:-${default[@]:0:2}}"
echo "${!prefix*[@]:0:3}"

# 5. Heredoc with complex content and nested expansions
cat << 'EOF' | grep -v "^#" | sed 's/^/  /'
# This is a comment
$(echo "Command substitution")
${var:-default}
$(( 1 + 2 * 3 ))
EOF

# 6. Complex pipeline with background processes and subshells
(echo "Starting"; sleep 1) &
(echo "Processing"; sleep 2) &
wait
echo "All done"

# 7. Nested if statements with complex conditions
if [[ $var =~ ^[0-9]+$ ]] && (( var > 0 )) && [ -f "$file" ]; then
    if [[ ${array[@]} =~ "value" ]] || (( ${#array[@]} > 5 )); then
        if [ "$(echo "$var" | grep -q "pattern")" ]; then
            echo "Deeply nested condition met"
        fi
    fi
fi

# 8. Complex case statement with patterns and command substitution
case "$(echo "$var" | tr '[:upper:]' '[:lower:]')" in
    *[0-9]*)
        case "${var,,}" in
            *pattern*)
                echo "Double nested pattern"
                ;;
            *)
                echo "Single nested pattern"
                ;;
        esac
        ;;
    *)
        echo "No numbers"
        ;;
esac

# 9. Function with complex parameter handling and local variables
complex_function() {
    local -a args=("$@")
    local -A options=()
    local i=0
    
    while (( i < ${#args[@]} )); do
        case "${args[i]}" in
            --*)
                local key="${args[i]#--}"
                local value="${args[i+1]:-true}"
                options["$key"]="$value"
                (( i += 2 ))
                ;;
            -*)
                local flags="${args[i]#-}"
                local j=0
                while (( j < ${#flags} )); do
                    options["${flags:j:1}"]="true"
                    (( j++ ))
                done
                (( i++ ))
                ;;
            *)
                break
                ;;
        esac
    done
    
    echo "Processed ${#options[@]} options"
}

# 10. Complex for loop with arithmetic and array manipulation
for ((i=0; i<${#array[@]}; i++)); do
    for ((j=0; j<${#array[i][@]}; j++)); do
        if (( array[i][j] > threshold )); then
            result[i]=$(( result[i] + array[i][j] ))
        fi
    done
done

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

# 12. Complex eval with nested expansions
eval "result=\$(( \${var:-0} + \${array[\${index:-0}]:-0} ))"

# 13. Nested subshells with complex command chains
(
    (
        (
            echo "Level 3"
            (echo "Level 4"; echo "Still level 4")
        ) | grep "Level"
    ) | sed 's/Level/Depth/'
) | wc -l

# 14. Complex redirects with process substitution
diff <(sort file1.txt) <(sort file2.txt) > comparison.txt 2>&1

# 15. Function definition with complex body and nested constructs
define_complex_function() {
    local name="$1"
    local body="$2"
    
    eval "$name() {
        $body
    }"
}

# 16. Complex test expressions with multiple operators
if [ -n "$var" -a -f "$file" -o -d "$dir" ] && [ "$(wc -l < "$file")" -gt 10 ]; then
    echo "Complex test passed"
fi

# 17. Nested brace expansion with complex patterns
echo {a,b,c}{1..3}{x,y,z}

# 18. Complex here-string with nested expansions
tr '[:upper:]' '[:lower:]' <<< "$(echo "UPPER: ${var^^}")"

# 19. Function call with complex argument processing
complex_function \
    --long-option="value with spaces" \
    --array-option=("item1" "item2" "item3") \
    --flag \
    "positional argument" \
    "${var:-default}" \
    "$(echo "computed")"

# 20. Final complex construct combining multiple challenging elements
(
    if [[ "$(echo "$var" | tr '[:upper:]' '[:lower:]')" =~ ^[a-z]+$ ]]; then
        for ((i=0; i<${#array[@]}; i++)); do
            if (( array[i] > threshold )) && [ -f "${files[i]}" ]; then
                result[i]=$(( result[i] + $(wc -l < "${files[i]}") ))
            fi
        done
    fi
) | sort -n | tail -n 5 > final_result.txt
