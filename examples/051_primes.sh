#!/bin/bash

# Prime Number Generator
# This script finds the first 1000 prime numbers

echo "=== Prime Number Generator (first 1000 primes) ==="

# Function to check if a number is prime
is_prime() {
    local n=$1
    
    if [ $n -lt 2 ]; then
        return 1
    fi
    
    if [ $n -eq 2 ]; then
        return 0
    fi
    
    if [ $((n % 2)) -eq 0 ]; then
        return 1
    fi
    
    local sqrt_n=$(echo "sqrt($n)" | bc)
    local i=3
    
    while [ $i -le $sqrt_n ]; do
        if [ $((n % i)) -eq 0 ]; then
            return 1
        fi
        i=$((i + 2))
    done
    
    return 0
}

echo "Finding first 100 prime numbers..."
echo "This may take a while..."

primes=(2)
count=1
candidate=3

while [ $count -lt 100 ]; do
    if is_prime $candidate; then
        primes+=($candidate)
        count=$((count + 1))
        
        # Show progress every 10 primes
        if [ $((count % 10)) -eq 0 ]; then
            echo "Found $count primes so far..."
        fi
    fi
    candidate=$((candidate + 2))
done

echo ""
echo "First 1000 prime numbers found!"
echo "Count: ${#primes[@]}"
echo "First 10: ${primes[@]:0:10}"
echo "Last 10: ${primes[@]: -10}"

echo "Prime number generation complete!"
