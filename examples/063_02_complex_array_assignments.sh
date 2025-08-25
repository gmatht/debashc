#!/bin/bash

# 2. Complex array assignments with nested expansions
declare -A matrix
matrix[0,0]=$(( (x + y) * z ))
matrix[1,1]=${array[${index}]}
matrix[2,2]=${!prefix@}
matrix[3,3]=${#array[@]}

echo "Matrix assignments completed"
