#!/bin/bash

# Demonstration of common grep parameters (commented examples).
# This file is intentionally non-executable (only comments) so tests can
# include it without requiring specific files or outputs.

# Basic usage
# grep "pattern" file.txt

# Case-insensitive search
# grep -i "pattern" file.txt

# Invert match (lines NOT matching)
# grep -v "pattern" file.txt

# Show line numbers
# grep -n "pattern" file.txt

# Count matching lines only
# grep -c "pattern" file.txt

# Only print the matching part of the line
# grep -o "pat[0-9]\+" file.txt

# Extended regular expressions (ERE)
# grep -E "(foo|bar)[0-9]+" file.txt

# Fixed strings (no regex)
# grep -F "a+b*c?" file.txt

# Match whole words
# grep -w "word" file.txt

# Match whole lines
# grep -x "exact whole line" file.txt

# Recursive search in directories
# grep -r "pattern" ./src

# Print file names with matches
# grep -l "pattern" *.txt

# Print file names without matches
# grep -L "pattern" *.txt

# Context lines: after, before, and both
# grep -A 2 "pattern" file.txt   # 2 lines After
# grep -B 2 "pattern" file.txt   # 2 lines Before
# grep -C 2 "pattern" file.txt   # 2 lines of Context (before & after)

# Limit number of matches per file
# grep -m 1 "pattern" file.txt

# Show byte offset with output lines
# grep -b "pattern" file.txt

# Suppress filename prefix on output (useful with multiple files)
# grep -h "pattern" file1.txt file2.txt

# Show filenames only (even with single file)
# grep -H "pattern" file.txt

# Null-terminated output (useful for xargs -0)
# grep -Z -l "pattern" **/*.txt

# Colorize matches (if your grep supports it)
# GREP_OPTIONS=  # ignore deprecated var if set
# grep --color=always "pattern" file.txt

# Binary files: treat as text
# grep -a "pattern" binary.dat

# Exclude files or directories
# grep -r --exclude "*.min.js" --exclude-dir node_modules "pattern" .

# Use multiple patterns
# grep -E "error|warning|notice" app.log

# Read patterns from file (one per line)
# grep -f patterns.txt file.txt

# Quiet mode (exit status only, no output)
# grep -q "pattern" file.txt && echo "found"

# End of grep parameter demonstrations


