#!/bin/bash

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

# Test the function
complex_function --flag1 --option1=value1 -abc
