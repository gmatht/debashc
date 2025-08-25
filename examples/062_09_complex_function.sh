#!/bin/bash

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

# Test the complex function
complex_function "test\"quote" "second_param" "third\"param"
