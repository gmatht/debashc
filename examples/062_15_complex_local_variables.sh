#!/bin/bash

# 15. Function with complex local variable declarations
function test_locals() {
    local var1="$1"
    local var2="${2:-default_value}"
    local var3="$(echo "$var1" | tr '[:lower:]' '[:upper:]')"
    
    echo "Var1: $var1"
    echo "Var2: $var2"
    echo "Var3: $var3"
}

# Test the function
test_locals "hello" "world"
