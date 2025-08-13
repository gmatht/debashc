#!/bin/bash

#todo: Handle Multi line statements.

# Script to comment out DEBUG: lines in Rust source files
# This script will find all lines containing "DEBUG:" and comment them out if they aren't already commented

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}Starting to comment out DEBUG: lines in src/ directory...${NC}"

# Counter for processed files and lines
total_files=0
total_lines=0

# Check if src directory exists
if [[ ! -d "src" ]]; then
    echo -e "${RED}Error: src/ directory not found!${NC}"
    exit 1
fi

# Find all Rust files in src/ directory
echo -e "${BLUE}Searching for Rust files...${NC}"
rust_files=($(find src/ -name "*.rs" -type f))

if [[ ${#rust_files[@]} -eq 0 ]]; then
    echo -e "${YELLOW}No Rust files found in src/ directory${NC}"
    exit 0
fi

echo -e "${BLUE}Found ${#rust_files[@]} Rust files to process${NC}"

# Process each Rust file
for file in "${rust_files[@]}"; do
    if [[ -f "$file" ]]; then
        echo -e "${YELLOW}Processing: $file${NC}"
        
        # Create a temporary file
        temp_file=$(mktemp 2>/dev/null || echo "${file}.tmp")
        
        # Process the file line by line
        line_count=0
        while IFS= read -r line; do
            # Check if line contains DEBUG: and is not already commented
            if [[ "$line" =~ DEBUG: ]] && [[ ! "$line" =~ ^[[:space:]]*// ]]; then
                # Comment out the line by adding // at the beginning
                echo "// $line" >> "$temp_file"
                ((line_count++))
                echo -e "  ${GREEN}Commented: $line${NC}"
            else
                # Keep the line as is
                echo "$line" >> "$temp_file"
            fi
        done < "$file"
        
        # Replace original file with temporary file
        if [[ $line_count -gt 0 ]]; then
            if mv "$temp_file" "$file" 2>/dev/null; then
                echo -e "  ${GREEN}Modified $file: $line_count DEBUG: lines commented${NC}"
                ((total_lines += line_count))
            else
                echo -e "  ${RED}Error: Failed to update $file${NC}"
                rm -f "$temp_file"
            fi
        else
            rm -f "$temp_file"
            echo -e "  ${BLUE}No changes needed in $file${NC}"
        fi
        
        ((total_files++))
    fi
done

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Processing complete!${NC}"
echo -e "${GREEN}Files processed: $total_files${NC}"
echo -e "${GREEN}Total DEBUG: lines commented: $total_lines${NC}"
echo -e "${GREEN}========================================${NC}"

# Optional: Show a summary of what was changed
if [[ $total_lines -gt 0 ]]; then
    echo -e "${YELLOW}Summary of changes:${NC}"
    echo -e "${YELLOW}All DEBUG: lines in Rust source files have been commented out.${NC}"
    echo -e "${YELLOW}You can uncomment them later by removing the '// ' prefix if needed.${NC}"
fi
