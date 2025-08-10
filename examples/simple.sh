#!/bin/bash

# This script demonstrates basic shell functionality
echo "Hello, World!"

# Valid if statement
if [ -f "test.txt" ]; then
    echo "File exists"
fi

# Valid for loop
for i in {1..5}; do
    echo $i
done 