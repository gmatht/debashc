#!/bin/bash

# 2. Complex nested parameter expansions with conflicting delimiters
echo "Testing complex parameter expansions..."
complex_var="hello world"
echo "${complex_var#*o}"  # Remove shortest match from beginning
echo "${complex_var##*o}" # Remove longest match from beginning
echo "${complex_var%o*}"  # Remove shortest match from end
echo "${complex_var%%o*}" # Remove longest match from end
