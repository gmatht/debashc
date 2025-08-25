#!/bin/bash

# 19. Complex pattern matching with extended globs
for file in *.{txt,log,dat}; do
    case "$file" in
        @(*.txt|*.log)) echo "Text file: $file";;
        *.dat) echo "Data file: $file";;
        *) echo "Other file: $file";;
    esac
done
