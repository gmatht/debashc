#!/bin/bash

# Control flow examples
if [ -f "file.txt" ]; then
    echo "File exists"
else
    echo "File does not exist"
fi

for i in {1..5}; do
    echo "Number: $i"
done

while [ $i -lt 10 ]; do
    echo "Counter: $i"
    i=$((i + 1))
done

function greet() {
    echo "Hello, $1!"
}

greet "World" 