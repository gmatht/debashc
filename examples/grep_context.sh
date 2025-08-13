#!/bin/bash

# Grep context and file operation examples
# Demonstrates grep's context and file handling capabilities

# Context lines: after, before, and both
echo -e "line1\nline2\nTARGET\nline4\nline5" | grep -A 2 "TARGET"
echo -e "line1\nline2\nTARGET\nline4\nline5" | grep -B 2 "TARGET"
echo -e "line1\nline2\nTARGET\nline4\nline5" | grep -C 1 "TARGET"

# Recursive search in current directory
echo "Creating test files..."
echo "pattern in file1" > temp_file1.txt
echo "no pattern in file2" > temp_file2.txt
echo "pattern in file3" > temp_file3.txt

echo "Recursive search results:"
grep -r "pattern" . --include="*.txt"

# Print file names with matches
grep -l "pattern" *.txt

# Print file names without matches
grep -L "pattern" *.txt

# Cleanup
rm temp_file*.txt
