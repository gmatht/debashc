#!/bin/bash

# Basic grep usage examples
# Demonstrates fundamental grep operations

# Basic usage
grep "pattern" /dev/null || echo "No matches found"

# Case-insensitive search
echo "HELLO world" | grep -i "hello"

# Invert match (lines NOT matching)
echo -e "line1\nline2\nline3" | grep -v "line2"

# Show line numbers
echo -e "first\nsecond\nthird" | grep -n "second"

# Count matching lines only
echo -e "match\nno match\nmatch again" | grep -c "match"

# Only print the matching part of the line
echo "text with pattern123 in it" | grep -o "pattern[0-9]\+"
