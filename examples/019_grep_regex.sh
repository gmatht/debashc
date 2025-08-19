#!/bin/bash

# Grep regex and pattern matching examples
# Demonstrates advanced grep pattern capabilities

# Extended regular expressions (ERE)
echo "foo123 bar456" | grep -E "(foo|bar)[0-9]+"

# Fixed strings (no regex)
echo "a+b*c?" | grep -F "a+b*c?"

# Match whole words
echo "word wordly subword" | grep -w "word"

# Match whole lines
echo -e "exact whole line\npartial line" | grep -x "exact whole line"

# Multiple patterns
echo -e "error message\nwarning message\ninfo message" | grep -E "error|warning"

# Read patterns from here-string
echo -e "error\nwarning" | grep -f <(echo -e "error\nwarning")

# Complex regex with groups
echo "file123.txt backup456.bak" | grep -E "([a-z]+)([0-9]+)\.([a-z]+)"
