#!/bin/bash

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

# Test the nested functions
outer_func
