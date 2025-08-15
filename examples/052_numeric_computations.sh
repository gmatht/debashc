#!/bin/bash

# Comprehensive Numeric Computation Examples
# This script demonstrates various mathematical algorithms

echo "=== Comprehensive Numeric Computation Examples ==="
echo ""

# Function to calculate Fibonacci numbers
fibonacci() {
    local n=$1
    local a=0
    local b=1
    
    if [ $n -le 1 ]; then
        echo $n
        return
    fi
    
    for ((i=2; i<=n; i++)); do
        local temp=$((a + b))
        a=$b
        b=$temp
    done
    
    echo $b
}

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
                factors="$factors × $divisor"
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
                    factors="$factors × $n"
                fi
            fi
            break
        fi
    done
    
    echo "$factors"
}

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

# Function to find first N primes
find_primes() {
    local count=$1
    local primes=(2)
    local found=1
    local candidate=3
    
    echo "Finding first $count prime numbers..."
    
    while [ $found -lt $count ]; do
        if is_prime $candidate; then
            primes+=($candidate)
            found=$((found + 1))
            
            # Show progress every 100 primes
            if [ $((found % 100)) -eq 0 ]; then
                echo "Found $found primes so far..."
            fi
        fi
        candidate=$((candidate + 2))
    done
    
    echo "First $count primes found!"
    echo "First 10: ${primes[@]:0:10}"
    echo "Last 10: ${primes[@]: -10}"
}

# Function to calculate GCD
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

# Function to calculate LCM (Least Common Multiple)
lcm() {
    local a=$1
    local b=$2
    local gcd_result=$(gcd $a $b)
    echo $((a * b / gcd_result))
}

# Performance measurement function
measure_time() {
    local start_time=$(date +%s%N)
    eval "$1"
    local end_time=$(date +%s%N)
    local duration=$((end_time - start_time))
    echo "Duration: $((duration / 1000000)) ms"
}

echo "1. Fibonacci Sequence (first 20 numbers):"
fib_numbers=""
for i in {0..19}; do
    fib_numbers="$fib_numbers $(fibonacci $i)"
done
echo "   $fib_numbers"
echo ""

echo "2. Number Factorization:"
factorize 12
factorize 28
factorize 100
factorize 12345
echo ""

echo "3. Prime Number Generation:"
find_primes 100  # Reduced to 100 for faster execution
echo ""

echo "4. Greatest Common Divisor Examples:"
echo "   gcd(48, 18) = $(gcd 48 18)"
echo "   gcd(54, 24) = $(gcd 54 24)"
echo "   gcd(7, 13) = $(gcd 7 13)"
echo "   gcd(100, 25) = $(gcd 100 25)"
echo "   gcd(12345, 67890) = $(gcd 12345 67890)"
echo ""

echo "5. Least Common Multiple Examples:"
echo "   lcm(12, 18) = $(lcm 12 18)"
echo "   lcm(15, 20) = $(lcm 15 20)"
echo "   lcm(8, 12) = $(lcm 8 12)"
echo ""

echo "6. Performance Benchmarks:"
echo "   Computing fibonacci(30):"
measure_time "fibonacci 30 > /dev/null"

echo "   Factorizing 12345:"
measure_time "factorize 12345 > /dev/null"

echo "   Finding first 50 primes:"
measure_time "find_primes 50 > /dev/null"

echo ""
echo "=== All numeric computations complete! ==="
echo ""
echo "You can now test these scripts with your translator:"
echo "  ./sh2perl parse --perl examples/fibonacci.sh"
echo "  ./sh2perl parse --rust examples/factorize.sh"
echo "  ./sh2perl parse --python examples/primes.sh"
echo "  ./sh2perl parse --lua examples/gcd.sh"
echo "  ./sh2perl parse --js examples/numeric_computations.sh"
