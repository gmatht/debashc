#!/bin/bash

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
