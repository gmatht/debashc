#!/bin/bash

# Advanced grep features and options
# Demonstrates specialized grep capabilities

# Limit number of matches per file
echo -e "match1\nmatch2\nmatch3\nmatch4" | grep -m 2 "match"

# Show byte offset with output lines
echo "text with pattern in it" | grep -b "pattern"

# Suppress filename prefix on output
echo "content" > temp_file.txt
grep -h "content" temp_file.txt

# Show filenames only (even with single file)
grep -H "content" temp_file.txt

# Null-terminated output (useful for xargs -0)
grep -Z -l "pattern" temp_file.txt | tr '\0' '\n'

# Colorize matches (if your grep supports it)
echo "text with pattern in it" | grep --color=always "pattern" || echo "Color not supported"

# Quiet mode (exit status only, no output)
grep -q "pattern" temp_file.txt && echo "found" || echo "not found"

# Cleanup
rm temp_file.txt
