#!/bin/bash

# Grep parameters and options examples
# Demonstrates various grep command line parameters

set -euo pipefail

echo "== Basic grep parameters =="
echo "text with pattern" | grep -i "PATTERN"
echo "line1\nline2\nline3" | grep -v "line2"
echo "match\nno match\nmatch again" | grep -c "match"

echo "== Context parameters =="
echo -e "line1\nline2\nTARGET\nline4\nline5" | grep -A 2 "TARGET"
echo -e "line1\nline2\nTARGET\nline4\nline5" | grep -B 2 "TARGET"
echo -e "line1\nline2\nTARGET\nline4\nline5" | grep -C 1 "TARGET"

echo "== File handling parameters =="
echo "content" > temp_file.txt
grep -H "content" temp_file.txt
grep -h "content" temp_file.txt
grep -l "content" temp_file.txt
grep -L "nonexistent" temp_file.txt

echo "== Output formatting parameters =="
echo "text with pattern in it" | grep -o "pattern"
echo "text with pattern in it" | grep -b "pattern"
echo "text with pattern in it" | grep -n "pattern"

echo "== Recursive and include/exclude parameters =="
mkdir -p test_dir
echo "pattern here" > test_dir/file1.txt
echo "no pattern" > test_dir/file2.txt
grep -r "pattern" test_dir
grep -r "pattern" test_dir --include="*.txt"
grep -r "pattern" test_dir --exclude="*.bak"

echo "== Advanced parameters =="
echo -e "match1\nmatch2\nmatch3\nmatch4" | grep -m 2 "match"
echo "text with pattern in it" | grep -q "pattern" && echo "found" || echo "not found"
grep -Z -l "pattern" temp_file.txt | tr '\0' '\n'

# Cleanup
rm -f temp_file.txt
rm -rf test_dir
