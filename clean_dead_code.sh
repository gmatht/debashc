#!/bin/bash

# Dead Code Cleaner for Rust Projects
# This script shows cargo check output and can optionally remove dead code

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

echo -e "${BLUE}🔍 Dead Code Cleaner for Rust Projects${NC}"
echo "=============================================="

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

# Extract file paths
echo ""
echo -e "${BLUE}📋 Files with dead code:${NC}"
echo "================================"

# Use a more robust approach to extract file paths without problematic grep patterns
grep "never used" cargo_check_output.tmp | while read line; do
    # Check if line contains file path information using string operations instead of grep
    if [[ "$line" == *"-->"* ]]; then
        # Extract file path using awk and clean it up
        file_path=$(echo "$line" | awk -F"-->" '{print $2}' | awk -F":" '{print $1}' | sed 's/^[[:space:]]*//;s/[[:space:]]*$//' 2>/dev/null)
        # Check if it's a source file - handle both Windows and Unix paths
        if [ -n "$file_path" ] && [ "$file_path" != "$line" ] && ([[ "$file_path" == src* ]] || [[ "$file_path" == src\\* ]]); then
            echo -e "${YELLOW}  📁 $file_path${NC}"
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
    
    # Get unique files with dead code using a more robust approach for Windows paths
    echo -e "${BLUE}  Extracting files with dead code...${NC}"
    # Extract files with dead code - handle both Windows and Unix paths
    echo -e "${BLUE}  Raw cargo check output for debugging:${NC}"
    grep "never used" cargo_check_output.tmp | head -5
    
    # The file paths are on the next line after "never used" warnings
    # Use a Windows-compatible approach with awk
    files_to_process=$(awk '
        /warning:.*never used/ {
            # Read the next line to get the file path
            getline next_line
            if (next_line ~ /-->/) {
                # Extract file path from the --> line
                split(next_line, parts, "-->")
                if (length(parts) >= 2) {
                    # Extract just the file path part (before the colon)
                    split(parts[2], file_parts, ":")
                    if (length(file_parts) >= 1) {
                        # Clean up the file path
                        gsub(/^[[:space:]]+|[[:space:]]+$/, "", file_parts[1])
                        if (file_parts[1] ~ /^src/) {
                            print file_parts[1]
                        }
                    }
                }
            }
        }
    ' cargo_check_output.tmp | sort -u)
    
    echo -e "${BLUE}  Extracted file paths:${NC}"
    echo "$files_to_process"
    
    if [ -z "$files_to_process" ]; then
        echo -e "${YELLOW}  No source files with dead code found to process${NC}"
    else
        echo -e "${BLUE}  Found files to process:${NC}"
        echo "$files_to_process" | while read file_path; do
            echo -e "${BLUE}    - $file_path${NC}"
        done
        
        for file_path in $files_to_process; do
            if [ -f "$file_path" ]; then
                echo -e "${BLUE}  Processing: $file_path${NC}"
                
                if [ "$DRY_RUN" = "false" ]; then
                    # Create backup
                    backup_file="$BACKUP_DIR/$(basename "$file_path")"
                    cp "$file_path" "$backup_file"
                    echo -e "${GREEN}    ✅ Backup created: $backup_file${NC}"
                fi
                
                # Get functions to remove from this file - handle Windows paths properly
                echo -e "${BLUE}    Looking for functions in: $file_path${NC}"
                
                # Use a Windows-compatible approach to extract functions from this specific file
                functions_to_remove=$(awk -v target_file="$file_path" '
                    /warning:.*never used/ {
                        # Read the next line to get the file path
                        getline next_line
                        if (next_line ~ /-->/) {
                            # Extract file path from the --> line
                            split(next_line, parts, "-->")
                            if (length(parts) >= 2) {
                                # Extract just the file path part (before the colon)
                                split(parts[2], file_parts, ":")
                                if (length(file_parts) >= 1) {
                                    # Clean up the file path
                                    gsub(/^[[:space:]]+|[[:space:]]+$/, "", file_parts[1])
                                    # Check if this warning is for our target file
                                    if (file_parts[1] == target_file) {
                                        # Extract function name from the warning line
                                        if ($0 ~ /`/) {
                                            split($0, func_parts, "`")
                                            if (length(func_parts) >= 2) {
                                                print func_parts[2]
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                ' cargo_check_output.tmp)
                
                if [ -n "$functions_to_remove" ]; then
                    echo -e "${YELLOW}      Found functions to remove: $functions_to_remove${NC}"
                else
                    echo -e "${YELLOW}      No functions found to remove from this file${NC}"
                fi
            
                if [ "$DRY_RUN" = "false" ] && [ -n "$functions_to_remove" ]; then
                    echo -e "${BLUE}    Removing functions: $functions_to_remove${NC}"
                    # Remove dead functions using Python
                    python3 -c "
import re
import sys

def remove_dead_functions(file_path, functions_to_remove):
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    original_content = content
    removed_count = 0
    
    for func_name in functions_to_remove:
        if not func_name or func_name == file_path:
            continue
            
        # Pattern to match function definitions (various Rust function syntaxes)
        patterns = [
            # pub fn func_name(...)
            rf'(\s*)(pub\s+)?fn\s+{re.escape(func_name)}\s*\([^)]*\)\s*->\s*[^{]*\s*{{[^}}]*}}',
            # fn func_name(...)
            rf'(\s*)(pub\s+)?fn\s+{re.escape(func_name)}\s*\([^)]*\)\s*{{[^}}]*}}',
            # pub fn func_name(...) -> Type
            rf'(\s*)(pub\s+)?fn\s+{re.escape(func_name)}\s*\([^)]*\)\s*->\s*[^{]*\s*{{[^}}]*}}',
            # fn func_name(...) -> Type
            rf'(\s*)(pub\s+)?fn\s+{re.escape(func_name)}\s*\([^)]*\)\s*{{[^}}]*}}',
        ]
        
        for pattern in patterns:
            if re.search(pattern, content, re.DOTALL):
                content = re.sub(pattern, '', content, flags=re.DOTALL)
                removed_count += 1
                break
    
    # Clean up extra blank lines
    content = re.sub(r'\n\s*\n\s*\n', '\n\n', content)
    
    return content, removed_count

if __name__ == '__main__':
    if len(sys.argv) < 3:
        print('Usage: python3 script.py <file_path> <func1> <func2> ...')
        sys.exit(1)
    
    file_path = sys.argv[1]
    functions_to_remove = sys.argv[2:]
    
    try:
        new_content, removed_count = remove_dead_functions(file_path, functions_to_remove)
        
        if removed_count > 0:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(new_content)
            print(f'Removed {removed_count} functions from {file_path}')
        else:
            print(f'No functions removed from {file_path}')
            
    except Exception as e:
        print(f'Error processing {file_path}: {e}')
        sys.exit(1)
" "$file_path" $functions_to_remove || echo "    ⚠️  Python processing failed, skipping"
                fi
            fi
        done
    fi
    
    if [ "$DRY_RUN" = "false" ]; then
        echo ""
        echo -e "${BLUE}🔍 Running cargo check again to verify cleanup...${NC}"
        
        # Run cargo check again to see if we fixed the warnings
        if cargo check 2>&1 | grep -q "never used"; then
            echo -e "${YELLOW}⚠️  Some dead code warnings remain. Manual review may be needed.${NC}"
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
    echo "  - Functions removed: $total_warnings"
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
