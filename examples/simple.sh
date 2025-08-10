#!/bin/bash

# This script has invalid control structure syntax
echo "Hello, World!"

# Invalid if statement - missing semicolon
if [ -f "test.txt" then
    echo "File exists"
fi

# Invalid for loop - missing closing brace
for i in {1..5 do
    echo $i
done 