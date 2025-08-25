#!/bin/bash

# 3. Nested command substitutions with complex quoting
output=$(echo "Result: $(echo "Nested: $(echo "Deep: $(echo "Level 4")")")")
echo "$output"
