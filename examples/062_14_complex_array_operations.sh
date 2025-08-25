#!/bin/bash

# 14. Complex array operations
echo "Testing complex array operations..."
declare -a array=("item1" "item2" "item3")
array+=("item4")
echo "Array: ${array[@]}"
echo "Length: ${#array[@]}"
echo "First item: ${array[0]}"
