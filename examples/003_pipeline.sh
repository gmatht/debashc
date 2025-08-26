#!/bin/bash

# Pipeline examples
ls | grep "\.txt$" | wc -l
cat file.txt | sort | uniq -c | sort -nr
find . -name "*.sh" | xargs grep -l "function"  | tr -d "\\\\/"