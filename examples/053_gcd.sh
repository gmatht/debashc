#!/bin/bash

# Greatest Common Divisor Calculator
# This script calculates GCD using Euclidean algorithm

echo "=== Greatest Common Divisor Examples ==="

# Function to calculate GCD using Euclidean algorithm
gcd() {
    local a=$1
    local b=$2
    
    while [ $b -ne 0 ]; do
        local temp=$b
        b=$((a % b))
        a=$temp
    done
    
    echo $a
}

# Test with various number pairs
echo "GCD calculations:"

# Test case 1: 48 and 18
result=$(gcd 48 18)
echo "gcd(48, 18) = $result"

# Test case 2: 54 and 24
result=$(gcd 54 24)
echo "gcd(54, 24) = $result"

# Test case 3: 7 and 13 (coprime)
result=$(gcd 7 13)
echo "gcd(7, 13) = $result"

# Test case 4: 100 and 25
result=$(gcd 100 25)
echo "gcd(100, 25) = $result"

# Test case 5: 12345 and 67890
result=$(gcd 12345 67890)
echo "gcd(12345, 67890) = $result"

# Interactive mode
echo ""
echo "Enter two numbers to calculate their GCD (or press Ctrl+C to exit):"
while true; do
    echo -n "First number: "
    read -r num1
    
    if [ -z "$num1" ]; then
        break
    fi
    
    echo -n "Second number: "
    read -r num2
    
    if [ -z "$num2" ]; then
        break
    fi
    
    if [[ "$num1" =~ ^[0-9]+$ ]] && [[ "$num2" =~ ^[0-9]+$ ]]; then
        result=$(gcd $num1 $num2)
        echo "gcd($num1, $num2) = $result"
    else
        echo "Please enter valid positive integers."
    fi
    
    echo ""
done

echo "GCD calculation complete!"
