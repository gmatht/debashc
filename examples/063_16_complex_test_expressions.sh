#!/bin/bash

# 16. Complex test expressions with multiple operators
if [ -n "$var" -a -f "$file" -o -d "$dir" ] && [ "$(wc -l < "$file")" -gt 10 ]; then
    echo "Complex test passed"
fi
