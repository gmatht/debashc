#!/usr/bin/env python3
"""
Script to find sequential chunks of output.push_str( patterns in Rust files.
Prints chunks of consecutive lines that all contain output.push_str( calls.
"""

import os
import re
from pathlib import Path
from typing import List, Tuple

def find_sequential_chunks(src_dir: str = "src") -> None:
    """Find and print sequential chunks of output.push_str( patterns."""
    
    src_path = Path(src_dir)
    if not src_path.exists():
        print(f"Error: Source directory '{src_dir}' does not exist")
        return
    
    # Pattern to match output.push_str(, result.push_str(, @ARGV, format!( calls, or Perl string literals
    push_str_pattern = re.compile(r'(output|result)\.push_str\(|@ARGV|format!\(|"[^"]*(\$|ne |eq |defined\(|chomp\(|die\(|scalar\()[^"]*"')
    
    # Find all Rust files
    rust_files = list(src_path.rglob("*.rs"))
    
    for rust_file in rust_files:
        try:
            with open(rust_file, 'r', encoding='utf-8') as f:
                lines = f.readlines()
            
            # Find all lines with output.push_str( patterns
            matching_lines = []
            for line_num, line in enumerate(lines, 1):
                if push_str_pattern.search(line):
                    matching_lines.append((line_num, line.rstrip()))
            
            if not matching_lines:
                continue
            
            print(f"\n=== {rust_file.relative_to(src_path)} ===")
            
            # Group consecutive lines into chunks
            chunks = []
            current_chunk = [matching_lines[0]]
            
            for i in range(1, len(matching_lines)):
                current_line_num = matching_lines[i][0]
                prev_line_num = matching_lines[i-1][0]
                
                # If lines are consecutive (or very close), add to current chunk
                if current_line_num - prev_line_num <= 2:  # Allow 1 line gap
                    current_chunk.append(matching_lines[i])
                else:
                    # Start new chunk
                    if len(current_chunk) > 1:  # Only show chunks with 2+ lines
                        chunks.append(current_chunk)
                    current_chunk = [matching_lines[i]]
            
            # Don't forget the last chunk
            if len(current_chunk) > 1:
                chunks.append(current_chunk)
            
            # Print chunks
            for chunk in chunks:
                print("\n--- CHUNK ---")
                for line_num, line_content in chunk:
                    print(f"{line_num:3d}: {line_content}")
                print()
                
        except Exception as e:
            print(f"Error reading {rust_file}: {e}")

def find_sequential_chunks_detailed(src_dir: str = "src") -> None:
    """Find sequential chunks with more context around each chunk."""
    
    src_path = Path(src_dir)
    if not src_path.exists():
        print(f"Error: Source directory '{src_dir}' does not exist")
        return
    
    # Pattern to match output.push_str(, result.push_str(, @ARGV, format!( calls, or Perl string literals
    push_str_pattern = re.compile(r'(output|result)\.push_str\(|@ARGV|format!\(|"[^"]*(\$|ne |eq |defined\(|chomp\(|die\(|scalar\()[^"]*"')
    
    # Find all Rust files
    rust_files = list(src_path.rglob("*.rs"))
    
    for rust_file in rust_files:
        try:
            with open(rust_file, 'r', encoding='utf-8') as f:
                lines = f.readlines()
            
            # Find all lines with output.push_str( patterns
            matching_lines = []
            for line_num, line in enumerate(lines, 1):
                if push_str_pattern.search(line):
                    matching_lines.append((line_num, line.rstrip()))
            
            if not matching_lines:
                continue
            
            print(f"\n=== {rust_file.relative_to(src_path)} ===")
            
            # Group consecutive lines into chunks
            chunks = []
            current_chunk = [matching_lines[0]]
            
            for i in range(1, len(matching_lines)):
                current_line_num = matching_lines[i][0]
                prev_line_num = matching_lines[i-1][0]
                
                # If lines are consecutive (or very close), add to current chunk
                if current_line_num - prev_line_num <= 2:  # Allow 1 line gap
                    current_chunk.append(matching_lines[i])
                else:
                    # Start new chunk
                    if len(current_chunk) > 1:  # Only show chunks with 2+ lines
                        chunks.append(current_chunk)
                    current_chunk = [matching_lines[i]]
            
            # Don't forget the last chunk
            if len(current_chunk) > 1:
                chunks.append(current_chunk)
            
            # Print chunks with context
            for chunk in chunks:
                print("\n--- CHUNK ---")
                
                # Show context before chunk
                first_line = chunk[0][0]
                if first_line > 1:
                    print(f"{first_line-1:3d}: {lines[first_line-2].rstrip()}")
                
                # Show chunk lines
                for line_num, line_content in chunk:
                    print(f"{line_num:3d}: {line_content}")
                
                # Show context after chunk
                last_line = chunk[-1][0]
                if last_line < len(lines):
                    print(f"{last_line+1:3d}: {lines[last_line].rstrip()}")
                
                print()
                
        except Exception as e:
            print(f"Error reading {rust_file}: {e}")

def find_perl_specific_patterns(src_dir: str = "src") -> None:
    """Find patterns that would need to be changed for Rust output instead of Perl."""
    
    src_path = Path(src_dir)
    if not src_path.exists():
        print(f"Error: Source directory '{src_dir}' does not exist")
        return
    
    # Pattern to match Perl-specific syntax that would need changes for Rust
    perl_specific_pattern = re.compile(r"' =>.*\".*\\\\[^a-z]")
    
    # Find all Rust files
    rust_files = list(src_path.rglob("*.rs"))
    
    print("=== PERL-SPECIFIC PATTERNS THAT WOULD NEED CHANGES FOR RUST ===")
    print("(Regex pattern: ' =>.*\".*\\\\[^a-z])")
    print()
    
    total_matches = 0
    
    for rust_file in rust_files:
        try:
            with open(rust_file, 'r', encoding='utf-8') as f:
                lines = f.readlines()
            
            # Find all lines with Perl-specific patterns
            matching_lines = []
            for line_num, line in enumerate(lines, 1):
                if perl_specific_pattern.search(line):
                    matching_lines.append((line_num, line.rstrip()))
            
            if not matching_lines:
                continue
            
            print(f"\n=== {rust_file.relative_to(src_path)} ===")
            print(f"Found {len(matching_lines)} Perl-specific patterns:")
            
            # Print all matching lines
            for line_num, line_content in matching_lines:
                print(f"{line_num:3d}: {line_content}")
                total_matches += 1
            
        except Exception as e:
            print(f"Error reading {rust_file}: {e}")
    
    print(f"\n=== SUMMARY ===")
    print(f"Total Perl-specific patterns found: {total_matches}")
    print("These patterns would need to be changed for Rust output:")
    print("- Regex pattern escaping (Perl regex → Rust regex)")
    print("- String literal escaping (Perl string → Rust string)")
    print("- Special character escaping (Perl syntax → Rust syntax)")

def find_missing_patterns(src_dir: str = "src", missing_regex: str = None) -> None:
    """Find lines matching regex that are NOT in chunks (consecutive output.push_str, result.push_str calls, @ARGV, format! calls, or Perl string literals)."""
    
    if not missing_regex:
        print("Error: --missing requires a regex pattern")
        return
    
    src_path = Path(src_dir)
    if not src_path.exists():
        print(f"Error: Source directory '{src_dir}' does not exist")
        return
    
    # Pattern to match output.push_str(, result.push_str(, @ARGV, format!( calls, or Perl string literals
    push_str_pattern = re.compile(r'(output|result)\.push_str\(|@ARGV|format!\(|"[^"]*(\$|ne |eq |defined\(|chomp\(|die\(|scalar\()[^"]*"')
    
    # Pattern to match the missing regex
    try:
        missing_pattern = re.compile(missing_regex)
    except re.error as e:
        print(f"Error: Invalid regex pattern '{missing_regex}': {e}")
        return
    
    # Find all Rust files
    rust_files = list(src_path.rglob("*.rs"))
    
    print(f"=== LINES MATCHING '{missing_regex}' THAT ARE NOT IN CHUNKS ===")
    print("(Lines that match the regex but are not consecutive output.push_str, result.push_str calls, @ARGV, format! calls, or Perl string literals)")
    print()
    
    total_matches = 0
    
    for rust_file in rust_files:
        try:
            with open(rust_file, 'r', encoding='utf-8') as f:
                lines = f.readlines()
            
            # Find all lines with output.push_str(, result.push_str(, @ARGV, format!(, or Perl string literal patterns to identify chunks
            chunk_lines = set()
            for line_num, line in enumerate(lines, 1):
                if push_str_pattern.search(line):
                    chunk_lines.add(line_num)
            
            # Find lines that match the missing regex but are not in chunks
            missing_lines = []
            for line_num, line in enumerate(lines, 1):
                if missing_pattern.search(line) and line_num not in chunk_lines:
                    # Check if the match only occurs in a comment
                    line_stripped = line.strip()
                    if line_stripped.startswith('//') or line_stripped.startswith('/*') or line_stripped.startswith('*'):
                        # Skip lines that are entirely comments
                        continue
                    
                    # Check if the match is only in the comment part of the line
                    # Look for // comment marker and see if match is after it
                    comment_pos = line.find('//')
                    if comment_pos != -1:
                        # Check if the match occurs only after the comment marker
                        line_before_comment = line[:comment_pos]
                        if not missing_pattern.search(line_before_comment):
                            # Match only occurs in comment, skip this line
                            continue
                    
                    missing_lines.append((line_num, line.rstrip()))
            
            if not missing_lines:
                continue
            
            print(f"\n=== {rust_file.relative_to(src_path)} ===")
            print(f"Found {len(missing_lines)} lines matching '{missing_regex}' that are not in chunks:")
            
            # Print all matching lines
            for line_num, line_content in missing_lines:
                print(f"{line_num:3d}: {line_content}")
                total_matches += 1
            
        except Exception as e:
            print(f"Error reading {rust_file}: {e}")
    
    print(f"\n=== SUMMARY ===")
    print(f"Total lines matching '{missing_regex}' that are not in chunks: {total_matches}")
    print("These lines match the regex pattern but are not part of consecutive output.push_str, result.push_str calls, @ARGV, format! calls, or Perl string literals.")

def main():
    """Main function."""
    import argparse
    
    parser = argparse.ArgumentParser(description="Find Perl-specific patterns in Rust code")
    parser.add_argument("--src-dir", default="src", help="Source directory to search (default: src)")
    parser.add_argument("--detailed", "-d", action="store_true", help="Show context around chunks")
    parser.add_argument("--min-chunk-size", type=int, default=1, help="Minimum chunk size to show (default: 1)")
    parser.add_argument("--perl-specific", "-p", action="store_true", help="Find Perl-specific patterns that need changes for Rust")
    parser.add_argument("--missing", help="Find lines matching REGEX that are not in chunks (consecutive output.push_str, result.push_str calls, @ARGV, format! calls, or Perl string literals)")
    
    args = parser.parse_args()
    
    if args.missing:
        find_missing_patterns(args.src_dir, args.missing)
    elif args.perl_specific:
        find_perl_specific_patterns(args.src_dir)
    elif args.detailed:
        find_sequential_chunks_detailed(args.src_dir)
    else:
        find_sequential_chunks(args.src_dir)

if __name__ == "__main__":
    main()
