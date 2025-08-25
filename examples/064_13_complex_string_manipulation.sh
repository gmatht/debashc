#!/bin/bash

# 13. Complex string manipulation with parameter expansion
filename="my_file.txt"
basename="${filename%.*}"           # Remove extension
extension="${filename##*.}"         # Get extension
uppercase="${filename^^}"           # Convert to uppercase
lowercase="${filename,,}"           # Convert to lowercase

echo "Basename: $basename"
echo "Extension: $extension"
echo "Uppercase: $uppercase"
echo "Lowercase: $lowercase"
