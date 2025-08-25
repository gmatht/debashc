#!/bin/bash

# 15. Function definition with complex body and nested constructs
define_complex_function() {
    local name="$1"
    local body="$2"
    
    eval "$name() {
        $body
    }"
}

# Test the function
define_complex_function "test_func" "echo 'Hello from dynamic function'"
test_func
