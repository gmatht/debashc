#!/bin/bash

# 18. Array slicing and manipulation
numbers=(1 2 3 4 5 6 7 8 9 10)
middle=("${numbers[@]:3:4}")        # Elements 4-7
first_half=("${numbers[@]:0:5}")   # First 5 elements
last_half=("${numbers[@]:5}")      # Last 5 elements

echo "Middle: ${middle[@]}"
echo "First half: ${first_half[@]}"
echo "Last half: ${last_half[@]}"
