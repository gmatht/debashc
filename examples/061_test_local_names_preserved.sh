#!/bin/bash

function test_math() {
    local first_number=$1
    local second_number=$2
    local operation=$3
    
    case $operation in
        "add")
            echo $((first_number + second_number))
            ;;
        "subtract")
            echo $((first_number - second_number))
            ;;
        "multiply")
            echo $((first_number * second_number))
            ;;
        *)
            echo "Unknown operation: $operation"
            ;;
    esac
}

function test_strings() {
    local input_string=$1
    local search_pattern=$2
    local replacement=$3
    
    case $search_pattern in
        "start")
            echo "Replacing start of: $input_string with: $replacement"
            ;;
        "end")
            echo "Replacing end of: $input_string with: $replacement"
            ;;
        "middle")
            echo "Replacing middle of: $input_string with: $replacement"
            ;;
        *)
            echo "Unknown pattern: $search_pattern for string: $input_string"
            ;;
    esac
}

function test_arrays() {
    local array_name=$1
    local index=$2
    local new_value=$3
    
    case $index in
        "first")
            echo "Setting first element of $array_name to $new_value"
            ;;
        "last")
            echo "Setting last element of $array_name to $new_value"
            ;;
        *)
            echo "Setting element $index of $array_name to $new_value"
            ;;
    esac
}

# Test math function with meaningful local variable names
test_math 10 5 "add"
test_math 10 5 "multiply"

# Test string function with meaningful local variable names
test_strings "hello world" "start" "hi"
test_strings "hello world" "end" "bye"

# Test array function with meaningful local variable names
test_arrays "my_array" "first" "new_value"
test_arrays "my_array" "last" "final_value"
