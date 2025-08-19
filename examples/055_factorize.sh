#!/bin/bash

# Number Factorization Calculator
# This script finds the prime factors of given numbers

echo "=== Number Factorization Examples ==="

# Function to factorize a number
factorize() {
    local n=$1
    local divisor=2
    local factors=""
    
    echo -n "Factors of $n: "
    
    while [ $n -gt 1 ]; do
        while [ $((n % divisor)) -eq 0 ]; do
            if [ -z "$factors" ]; then
                factors="$divisor"
            else
                factors="$factors * $divisor"
            fi
            n=$((n / divisor))
        done
        divisor=$((divisor + 1))
        
        # Optimization: stop if divisor^2 > n
        if [ $((divisor * divisor)) -gt $n ]; then
            if [ $n -gt 1 ]; then
                if [ -z "$factors" ]; then
                    factors="$n"
                else
                    factors="$factors * $n"
                fi
            fi
            break
        fi
    done
    
    echo "$factors"
}

# Test with various numbers
factorize 12
factorize 28
factorize 100
factorize 12345

echo "Factorization complete!"
