#!/bin/bash

# 16. Function with complex argument handling
process_files() {
    local -a files=("$@")
    local count=0
    
    for file in "${files[@]}"; do
        if [[ -f "$file" ]]; then
            ((count++))
            echo "Processing: $file"
        fi
    done
    
    echo "Total files processed: $count"
}

# Test the function
process_files "file1.txt" "file2.txt" "nonexistent.txt"
