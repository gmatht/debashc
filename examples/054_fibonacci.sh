#!/bin/bash

# Fibonacci Sequence Calculator
# This script calculates and displays the first 20 Fibonacci numbers

echo "=== Fibonacci Sequence (first 20 numbers) ==="

# Initialize first two numbers
a=0
b=1

echo "Fibonacci numbers:"
echo -n "$a $b "

# Calculate next 18 numbers
for i in {3..20}; do
    temp=$((a + b))
    echo -n "$temp "
    a=$b
    b=$temp
done

echo ""
echo "Done!"
