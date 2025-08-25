#!/bin/bash

# 1. Ambiguous operators and precedence issues
# The lexer needs to handle these correctly with proper priorities
echo "Testing ambiguous operators..."
result=$((2**3**2))  # Should be 2**(3**2) = 2^9 = 512, not (2^3)^2 = 64
echo "2**3**2 = $result"
