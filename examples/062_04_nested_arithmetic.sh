#!/bin/bash

# 4. Nested arithmetic expressions with conflicting parentheses
echo "Testing nested arithmetic..."
result=$(( (2 + 3) * (4 - 1) + (5 ** 2) ))
echo "Complex arithmetic: $result"
