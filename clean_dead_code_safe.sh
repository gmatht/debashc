#!/bin/bash

# Safe Dead Code Cleaner for Rust Projects
# This script safely identifies and can optionally remove dead code without corrupting files

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
BACKUP_DIR="backup_$(date +%Y%m%d_%H%M%S)"
REMOVE_DEAD_CODE=false
DRY_RUN=false

# Function to show usage
show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -r, --remove     Actually remove dead code (creates backup first)"
    echo "  -d, --dry-run    Show what would be removed without actually removing"
    echo "  -h, --help       Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0               # Show dead code analysis only"
    echo "  $0 --dry-run     # Show what would be removed"
    echo "  $0 --remove      # Actually remove dead code (with backup)"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -r|--remove)
            REMOVE_DEAD_CODE=true
            shift
            ;;
        -d|--dry-run)
            DRY_RUN=true
            shift
            ;;
        -h|--help)
            show_usage
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

echo -e "${BLUE}🔍 Safe Dead Code Cleaner for Rust Projects${NC}"
echo "=================================================="

# Check if we're in a Rust project
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}❌ Error: Not in a Rust project directory (Cargo.toml not found)${NC}"
    exit 1
fi

# Check if cargo is available
if ! command -v cargo >/dev/null 2>&1; then
    echo -e "${RED}❌ Error: cargo command not found${NC}"
    exit 1
fi

echo -e "${YELLOW}📋 Running cargo check to identify dead code...${NC}"
echo ""

# Run cargo check and capture output
cargo check 2>&1 | tee cargo_check_output.tmp || echo "Cargo check completed with warnings (expected)"

# Count total warnings
total_warnings=$(grep -c "never used" cargo_check_output.tmp 2>/dev/null || echo "0")
echo -e "${BLUE}📊 Total 'never used' warnings: $total_warnings${NC}"

if [ "$total_warnings" = "0" ]; then
    echo -e "${GREEN}✅ No dead code found! Your project is clean.${NC}"
    rm -f cargo_check_output.tmp
    exit 0
fi

echo ""
echo -e "${BLUE}📋 Dead code warnings:${NC}"
echo "=============================="

# Show the warnings
grep -A 1 -B 1 "never used" cargo_check_output.tmp

echo ""
echo -e "${BLUE}📋 Summary of unused items:${NC}"
echo "=================================="

# Extract function names using a more robust approach
grep "never used" cargo_check_output.tmp | while read line; do
    if echo "$line" | grep -q "warning:"; then
        # Extract function name between backticks using awk
        function_name=$(echo "$line" | awk -F"\`" '{print $2}' 2>/dev/null)
        if [ -n "$function_name" ] && [ "$function_name" != "$line" ]; then
            echo -e "${YELLOW}  📝 $function_name${NC}"
        fi
    fi
done

# Handle removal if requested
if [ "$REMOVE_DEAD_CODE" = "true" ] || [ "$DRY_RUN" = "true" ]; then
    echo ""
    if [ "$DRY_RUN" = "true" ]; then
        echo -e "${YELLOW}🔍 DRY RUN MODE - No files will be modified${NC}"
    else
        echo -e "${RED}⚠️  REMOVAL MODE - Dead code will be removed!${NC}"
        echo -e "${BLUE}💾 Creating backup in $BACKUP_DIR${NC}"
        mkdir -p "$BACKUP_DIR"
    fi
    
    echo ""
    echo -e "${BLUE}🔧 Processing files to remove dead code...${NC}"
    
    # Create a Python script to safely extract and remove dead code
    cat > /tmp/safe_dead_code_cleaner.py << 'EOF'
import re
import sys
import os
from pathlib import Path

def parse_cargo_output():
    """Parse cargo check output to extract file paths and function names."""
    try:
        with open('cargo_check_output.tmp', 'r', encoding='utf-8') as f:
            content = f.read()
        
        lines = content.split('\n')
        file_functions = {}
        
        for i, line in enumerate(lines):
            if 'warning:' in line and 'never used' in line:
                # Extract function name from the warning line
                func_match = re.search(r'`([^`]+)`', line)
                if func_match:
                    func_name = func_match.group(1)
                    
                    # Look at the next line for the file path
                    if i + 1 < len(lines):
                        next_line = lines[i + 1]
                        if '-->' in next_line:
                            # Extract file path from --> line
                            match = re.search(r'--> (.+?):', next_line)
                            if match:
                                file_path = match.group(1).strip()
                                # Normalize path separators
                                file_path = file_path.replace('\\', '/')
                                
                                if file_path.startswith('src'):
                                    if file_path not in file_functions:
                                        file_functions[file_path] = []
                                    file_functions[file_path].append(func_name)
        
        return file_functions
    except Exception as e:
        print(f'Error parsing cargo output: {e}', file=sys.stderr)
        return {}

def find_function_boundaries(content, func_name):
    """Find the start and end boundaries of a function."""
    # Pattern to match function definition
    func_pattern = r'(\s*)(pub\s+)?fn\s+' + re.escape(func_name) + r'\s*\([^)]*\)(?:\s*->\s*[^{]*)?\s*\{'
    match = re.search(func_pattern, content)
    
    if not match:
        return None, None
    
    start_pos = match.start()
    
    # Find the matching closing brace by counting braces
    brace_count = 0
    in_string = False
    in_char = False
    escape_next = False
    
    for i in range(start_pos, len(content)):
        char = content[i]
        
        if escape_next:
            escape_next = False
            continue
        
        if char == '\\':
            escape_next = True
            continue
        
        if char == '"' and not in_string:
            in_string = not in_string
            continue
        
        if char == '\'' and not in_string:
            in_char = not in_char
            continue
        
        if in_string or in_char:
            continue
        
        if char == '{':
            brace_count += 1
        elif char == '}':
            brace_count -= 1
            if brace_count == 0:
                # Found the end of the function
                end_pos = i + 1
                return start_pos, end_pos
    
    return None, None

def safely_remove_functions(file_path, functions_to_remove):
    """Safely remove functions from a file."""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        original_content = content
        removed_count = 0
        modifications = []
        
        # Sort functions by position (reverse order to avoid offset issues)
        function_positions = []
        for func_name in functions_to_remove:
            start_pos, end_pos = find_function_boundaries(content, func_name)
            if start_pos is not None and end_pos is not None:
                function_positions.append((start_pos, end_pos, func_name))
        
        # Sort by start position in reverse order
        function_positions.sort(key=lambda x: x[0], reverse=True)
        
        for start_pos, end_pos, func_name in function_positions:
            # Remove the function
            before_func = content[:start_pos]
            after_func = content[end_pos:]
            
            # Clean up extra whitespace and newlines
            after_func = re.sub(r'^\s*\n', '', after_func)
            
            content = before_func + after_func
            removed_count += 1
            modifications.append(func_name)
        
        # Clean up extra blank lines
        content = re.sub(r'\n\s*\n\s*\n', '\n\n', content)
        
        return content, removed_count, modifications
    except Exception as e:
        print(f'Error processing {file_path}: {e}', file=sys.stderr)
        return None, 0, []

def main():
    if len(sys.argv) < 2:
        print('Usage: python3 script.py <command> [args...]')
        sys.exit(1)
    
    command = sys.argv[1]
    
    if command == 'extract':
        file_functions = parse_cargo_output()
        for file_path, functions in file_functions.items():
            print(f"{file_path}:{','.join(functions)}")
    
    elif command == 'remove':
        if len(sys.argv) < 4:
            print('Usage: python3 script.py remove <file_path> <func1,func2,...>')
            sys.exit(1)
        
        file_path = sys.argv[2]
        functions_str = sys.argv[3]
        functions_to_remove = functions_str.split(',')
        
        new_content, removed_count, modifications = safely_remove_functions(file_path, functions_to_remove)
        
        if new_content is not None and removed_count > 0:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(new_content)
            print(f'Removed {removed_count} functions from {file_path}: {", ".join(modifications)}')
        else:
            print(f'No functions removed from {file_path}')
    
    else:
        print(f'Unknown command: {command}')
        sys.exit(1)

if __name__ == '__main__':
    main()
EOF

    # Extract file paths and functions
    echo -e "${BLUE}  Extracting dead code information...${NC}"
    
    # Get the mapping of files to functions
    file_functions_map=$(python3 /tmp/safe_dead_code_cleaner.py extract)
    
    if [ -z "$file_functions_map" ]; then
        echo -e "${YELLOW}  No source files with dead code found to process${NC}"
    else
        echo -e "${BLUE}  Found files to process:${NC}"
        
        total_functions_removed=0
        
        for line in $file_functions_map; do
            file_path=$(echo "$line" | cut -d: -f1)
            functions=$(echo "$line" | cut -d: -f2-)
            
            echo -e "${BLUE}    Processing: $file_path${NC}"
            echo -e "${BLUE}      Functions to remove: $functions${NC}"
            
            if [ "$DRY_RUN" = "false" ]; then
                # Create backup
                backup_file="$BACKUP_DIR/$(basename "$file_path")"
                cp "$file_path" "$backup_file"
                echo -e "${GREEN}        ✅ Backup created: $backup_file${NC}"
                
                # Remove functions
                echo -e "${BLUE}      Removing functions...${NC}"
                result=$(python3 /tmp/safe_dead_code_cleaner.py remove "$file_path" "$functions")
                echo -e "${GREEN}        $result${NC}"
                
                # Extract count from result
                removed_count=$(echo "$result" | grep -o '[0-9]\+' | head -1)
                if [ -n "$removed_count" ]; then
                    total_functions_removed=$((total_functions_removed + removed_count))
                fi
            fi
        done
        
        echo ""
        echo -e "${BLUE}📊 Summary of removal:${NC}"
        echo "========================"
        echo -e "${GREEN}  Total functions removed: $total_functions_removed${NC}"
    fi
    
    if [ "$DRY_RUN" = "false" ]; then
        echo ""
        echo -e "${BLUE}🔍 Running cargo check again to verify cleanup...${NC}"
        
        # Run cargo check again to see if we fixed the warnings
        remaining_warnings=$(cargo check 2>&1 | grep -c "never used" || echo "0")
        
        if [ "$remaining_warnings" -gt 0 ]; then
            echo -e "${YELLOW}⚠️  $remaining_warnings dead code warnings remain. Manual review may be needed.${NC}"
        else
            echo -e "${GREEN}✅ All dead code warnings resolved!${NC}"
        fi
    fi
fi

echo ""
echo -e "${BLUE}💡 Recommendations:${NC}"
echo "=================="
echo "1. Review the functions above to see if they're actually needed"
echo "2. If they're not needed, you can remove them manually"
echo "3. If they are needed, check why they're not being called"
echo "4. Consider adding #[allow(dead_code)] if they're intentionally unused"

if [ "$REMOVE_DEAD_CODE" = "true" ]; then
    echo ""
    echo -e "${GREEN}🎉 Dead code cleanup completed!${NC}"
    echo "  - Backup created in: $BACKUP_DIR"
    echo "  - Functions removed: $total_functions_removed"
    echo ""
    echo -e "${BLUE}💡 Tip: Review the changes and run tests to ensure nothing important was removed${NC}"
elif [ "$DRY_RUN" = "true" ]; then
    echo ""
    echo -e "${YELLOW}🔍 DRY RUN completed. Run with --remove to actually remove dead code.${NC}"
else
    echo ""
    echo -e "${GREEN}🎉 Dead code analysis completed!${NC}"
    echo ""
    echo -e "${BLUE}💡 Tip: Use --dry-run to see what would be removed, or --remove to actually remove it${NC}"
fi

# Clean up
rm -f cargo_check_output.tmp
rm -f /tmp/safe_dead_code_cleaner.py
