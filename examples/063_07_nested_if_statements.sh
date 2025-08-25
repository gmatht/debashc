#!/bin/bash

# 7. Nested if statements with complex conditions
if [[ $var =~ ^[0-9]+$ ]] && (( var > 0 )) && [ -f "$file" ]; then
    if [[ ${array[@]} =~ "value" ]] || (( ${#array[@]} > 5 )); then
        if [ "$(echo "$var" | grep -q "pattern")" ]; then
            echo "Deeply nested condition met"
        fi
    fi
fi
