// Shell script examples for the Debashc compiler
// This file contains all examples that were previously embedded in WASM
// Generated automatically from examples/ directory

export const examples = {
  'ansi_quoting.sh': `\#!/usr/bin/env bash

\# ANSI-C quoting and special character examples
\# Demonstrates escape sequences and special character handling

set -euo pipefail

echo "== ANSI-C quoting =="
echo \$'line1\\nline2\\tTabbed'

echo "== Escape sequences =="
echo \$'bell\\a'
echo \$'backspace\\b'
echo \$'formfeed\\f'
echo \$'newline\\n'
echo \$'carriage\\rreturn'
echo \$'tab\\tseparated'
echo \$'vertical\\vtab'

echo "== Unicode and hex =="
echo \$'\\u0048\\u0065\\u006c\\u006c\\u006f'  \# Hello
echo \$'\\x48\\x65\\x6c\\x6c\\x6f'            \# Hello

echo "== Practical examples =="
\# Create a formatted table
printf \$'%-10s %-10s %s\\n' "Name" "Age" "City"
printf \$'%-10s %-10s %s\\n' "John" "25" "NYC"
printf \$'%-10s %-10s %s\\n' "Jane" "30" "LA"
`,
  'ansi_quoting_basic.sh': `\#!/usr/bin/env bash

\# Basic ANSI-C quoting examples
set -euo pipefail

echo "== ANSI-C quoting =="
echo \$'line1\\nline2\\tTabbed'
`,
  'ansi_quoting_escape.sh': `\#!/usr/bin/env bash

\# Escape sequence examples
set -euo pipefail

echo "== Escape sequences =="
echo \$'bell\\a'
echo \$'backspace\\b'
echo \$'formfeed\\f'
echo \$'newline\\n'
echo \$'carriage\\rreturn'
echo \$'tab\\tseparated'
echo \$'vertical\\vtab'
`,
  'ansi_quoting_practical.sh': `\#!/usr/bin/env bash

\# Practical ANSI-C quoting examples
set -euo pipefail

echo "== Practical examples =="
\# Create a formatted table
printf \$'%-10s %-10s %s\\n' "Name" "Age" "City"
printf \$'%-10s %-10s %s\\n' "John" "25" "NYC"
printf \$'%-10s %-10s %s\\n' "Jane" "30" "LA"
`,
  'ansi_quoting_unicode.sh': `\#!/usr/bin/env bash

\# Unicode and hex examples
set -euo pipefail

echo "== Unicode and hex =="
echo \$'\\u0048\\u0065\\u006c\\u006c\\u006f'  \# Hello
echo \$'\\x48\\x65\\x6c\\x6c\\x6f'            \# Hello
`,
  'args.sh': `\#!/usr/bin/env bash

\# Demonstrates reading command-line arguments
\# This example is intentionally simple so it parses cleanly

echo "== Argument count =="
echo "\$\#"

echo "== Arguments =="
for a in "\$@"; do
  echo "Arg: \$a"
done



`,
  'arrays.sh': `\#!/usr/bin/env bash

\# Array examples - indexed and associative arrays
\# Demonstrates basic array operations in Bash

set -euo pipefail

echo "== Indexed arrays =="
arr=(one two three )
echo "\${arr[1]}"        \# two
echo "\${\#arr[@]}"       \# 3
for x in "\${arr[@]}"; do printf "%s " "\$x"; done; echo

echo "== Associative arrays =="
declare -A map
map[foo]=bar
map[answer]=42
map[two]="1 + 1"
echo "\${map[foo]}"      \# bar
echo "\${map[answer]}"   \# 42

\# Show all keys and values
for k in "\${!map[@]}"; do echo "\$k => \${map[\$k]}"; done | sort \#Do not care about the order of the elements?
`,
  'arrays_associative.sh': `\#!/usr/bin/env bash

\# Associative array examples
set -euo pipefail

echo "== Associative arrays =="
declare -A map
map[foo]=bar
map[answer]=42
map[two]="1 + 1"
echo "\${map[foo]}"      \# bar
echo "\${map[answer]}"   \# 42

\# Show all keys and values
for k in "\${!map[@]}"; do echo "\$k => \${map[\$k]}"; done | sort
`,
  'arrays_indexed.sh': `\#!/usr/bin/env bash

\# Indexed array examples
set -euo pipefail

echo "== Indexed arrays =="
arr=(one two three )
echo "\${arr[1]}"        \# two
echo "\${\#arr[@]}"       \# 3
for x in "\${arr[@]}"; do printf "%s " "\$x"; done; echo
`,
  'brace_expansion.sh': `\#!/usr/bin/env bash

\# Brace expansion examples
\# Demonstrates various brace expansion patterns in Bash

set -euo pipefail

echo "== Basic brace expansion =="
echo {1..5}
echo {a..c}
echo {00..04..2}

echo "== Advanced brace expansion =="
echo {a,b,c}{1,2,3}
echo {1..10..2}
echo {a..z..3}

echo "== Practical examples =="
\# Create numbered files
touch file_{001..005}.txt
ls file_*.txt
rm file_*.txt
`,
  'brace_expansion_advanced.sh': `\#!/usr/bin/env bash

\# Advanced brace expansion examples
set -euo pipefail

echo "== Advanced brace expansion =="
echo {a,b,c}{1,2,3}
echo {1..10..2}
echo {a..z..3}
`,
  'brace_expansion_basic.sh': `\#!/usr/bin/env bash

\# Basic brace expansion examples
set -euo pipefail

echo "== Basic brace expansion =="
echo {1..5}
echo {a..c}
echo {00..04..2}
`,
  'brace_expansion_practical.sh': `\#!/usr/bin/env bash

\# Practical brace expansion examples
set -euo pipefail

echo "== Practical examples =="
\# Create numbered files
touch file_{001..005}.txt
ls file_*.txt
rm file_*.txt
`,
  'cat_EOF.sh': `cat <<EOF
alpha
beta
gamma ...
EOF

cat <<FISH
oyster
snapper
salmon
FISH
`,
  'cd..sh': `cd ..
ls
`,
  'control_flow.sh': `\#!/bin/bash

\# Control flow examples
if [ -f "file.txt" ]; then
    echo "File exists"
else
    echo "File does not exist"
fi

for i in {1..5}; do
    echo "Number: \$i"
done

while [ \$i -lt 10 ]; do
    echo "Counter: \$i"
    i=\$((i + 1))
done

function greet() {
    echo "Hello, \$1!"
}

greet "World" `,
  'control_flow_function.sh': `\#!/bin/bash

\# Function examples
function greet() {
    echo "Hello, \$1!"
}

greet "World"
`,
  'control_flow_if.sh': `\#!/bin/bash

\# If statement examples
if [ -f "file.txt" ]; then
    echo "File exists"
else
    echo "File does not exist"
fi
`,
  'control_flow_loops.sh': `\#!/bin/bash

\# Loop examples
for i in {1..5}; do
    echo "Number: \$i"
done

for i in {1..3}; do j=\$((j+1)); done; echo \$j

while [ \$i -lt 10 ]; do
    echo "Counter: \$i"
    i=\$((i + 1))
done
`,
  'file.txt': `apple
banana
apple
cherry
banana
apple
date
elderberry
apple
banana
cherry
`,
  'find_example.sh': `\#!/bin/bash

\# Find all .txt files in current directory and subdirectories
find . -name "*.txt" -type f

\# Find files modified in the last 7 days
find . -mtime -7 -type f

\# Find files modified in the last 1 day
find . -mtime -1 -type f

\# Find files modified in the last 1 hour
find . -mmin -60 -type f

\# Find files larger than 1MB
find . -size +1M -type f

\# Find empty files and directories
find . -empty

\# Don't use  yet, they are not portable
\# Find files with specific permissions (executable)
\# find . -perm -u+x -type f

\# Find files by owner
\#find . -user \$USER -type f

\# Find files by group
\#find . -group \$(id -gn) -type f

\# Find files and execute command on them
find . -name "*.log" -exec rm {} \\;

\# Find files and show detailed information
find . -type f -ls

\# Find files excluding certain directories
find . -type f -not -path "./.git/*" -not -path "./node_modules/*"
`,
  'grep_advanced.sh': `\#!/bin/bash

\# Advanced grep features and options
\# Demonstrates specialized grep capabilities

\# Limit number of matches per file
echo -e "match1\\nmatch2\\nmatch3\\nmatch4" | grep -m 2 "match"

\# Show byte offset with output lines
echo "text with pattern in it" | grep -b "pattern"

\# Suppress filename prefix on output
echo "content" > temp_file.txt
grep -h "content" temp_file.txt

\# Show filenames only (even with single file)
grep -H "content" temp_file.txt

\# Null-terminated output (useful for xargs -0)
grep -Z -l "pattern" temp_file.txt | tr '\\0' '\\n'

\# Colorize matches (if your grep supports it)
echo "text with pattern in it" | grep --color=always "pattern" || echo "Color not supported"

\# Quiet mode (exit status only, no output)
grep -q "pattern" temp_file.txt && echo "found" || echo "not found"

\# Cleanup
rm temp_file.txt
`,
  'grep_basic.sh': `\#!/bin/bash

\# Basic grep usage examples
\# Demonstrates fundamental grep operations

\# Basic usage
grep "pattern" /dev/null || echo "No matches found"

\# Case-insensitive search
echo "HELLO world" | grep -i "hello"

\# Invert match (lines NOT matching)
echo -e "line1\\nline2\\nline3" | grep -v "line2"

\# Show line numbers
echo -e "first\\nsecond\\nthird" | grep -n "second"

\# Count matching lines only
echo -e "match\\nno match\\nmatch again" | grep -c "match"

\# Only print the matching part of the line
echo "text with pattern123 in it" | grep -o "pattern[0-9]\\+"
`,
  'grep_context.sh': `\#!/bin/bash

\# Grep context and file operation examples
\# Demonstrates grep's context and file handling capabilities

\# Context lines: after, before, and both
echo -e "line1\\nline2\\nTARGET\\nline4\\nline5" | grep -A 2 "TARGET"
echo -e "line1\\nline2\\nTARGET\\nline4\\nline5" | grep -B 2 "TARGET"
echo -e "line1\\nline2\\nTARGET\\nline4\\nline5" | grep -C 1 "TARGET"

\# Recursive search in current directory
echo "Creating test files..."
echo "pattern in file1" > temp_file1.txt
echo "no pattern in file2" > temp_file2.txt
echo "pattern in file3" > temp_file3.txt

echo "Recursive search results:"
grep -r "pattern" . --include="*.txt"

\# Print file names with matches
grep -l "pattern" *.txt

\# Print file names without matches
grep -L "pattern" *.txt

\# Cleanup
rm temp_file*.txt
`,
  'grep_params.sh': `\#!/bin/bash

\# Grep parameters and options examples
\# Demonstrates various grep command line parameters

set -euo pipefail

echo "== Basic grep parameters =="
echo "text with pattern" | grep -i "PATTERN"
echo "line1\\nline2\\nline3" | grep -v "line2"
echo "match\\nno match\\nmatch again" | grep -c "match"

echo "== Context parameters =="
echo -e "line1\\nline2\\nTARGET\\nline4\\nline5" | grep -A 2 "TARGET"
echo -e "line1\\nline2\\nTARGET\\nline4\\nline5" | grep -B 2 "TARGET"
echo -e "line1\\nline2\\nTARGET\\nline4\\nline5" | grep -C 1 "TARGET"

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
echo -e "match1\\nmatch2\\nmatch3\\nmatch4" | grep -m 2 "match"
echo "text with pattern in it" | grep -q "pattern" && echo "found" || echo "not found"
grep -Z -l "pattern" temp_file.txt | tr '\\0' '\\n'

\# Cleanup
rm -f temp_file.txt
rm -rf test_dir
`,
  'grep_regex.sh': `\#!/bin/bash

\# Grep regex and pattern matching examples
\# Demonstrates advanced grep pattern capabilities

\# Extended regular expressions (ERE)
echo "foo123 bar456" | grep -E "(foo|bar)[0-9]+"

\# Fixed strings (no regex)
echo "a+b*c?" | grep -F "a+b*c?"

\# Match whole words
echo "word wordly subword" | grep -w "word"

\# Match whole lines
echo -e "exact whole line\\npartial line" | grep -x "exact whole line"

\# Multiple patterns
echo -e "error message\\nwarning message\\ninfo message" | grep -E "error|warning"

\# Read patterns from here-string
echo -e "error\\nwarning" | grep -f <(echo -e "error\\nwarning")

\# Complex regex with groups
echo "file123.txt backup456.bak" | grep -E "([a-z]+)([0-9]+)\\.([a-z]+)"
`,
  'home.sh': `[ ~ = "\$HOME" ] && echo 1 || echo -
[ ~/Documents = "\$HOME" ] && echo 2 || echo -
[ ~/Documents = "\$HOME/Documents" ] && echo 3 || echo -`,
  'local.sh': `a=1
echo \$a
(a=2; echo \$a)
(echo \$a)
echo \$a`,
  'misc.sh': `\#!/usr/bin/env bash

echo "== Subshell =="
( echo inside-subshell )

echo "== Simple pipeline =="
echo "alpha beta" | grep beta


`,
  'parameter_expansion.sh': `\#!/usr/bin/env bash

\# Parameter expansion examples
\# Demonstrates advanced parameter manipulation in Bash

set -euo pipefail

echo "== Case modification in parameter expansion =="
name="world"
echo "\${name^^}"        \# WORLD
echo "\${name,,}"        \# world
echo "\${name^}"         \# World

echo "== Advanced parameter expansion =="
path="/tmp/file.txt"
echo "\${path\#\#*/}"       \# file.txt
echo "\${path%/*}"        \# /tmp
s2="abba"; echo "\${s2//b/X}"  \# aXXa

echo "== More parameter expansion =="
var="hello world"
echo "\${var\#hello}"      \#  world
echo "\${var%world}"      \# hello 
echo "\${var//o/0}"       \# hell0 w0rld

echo "== Default values =="
unset maybe
echo "\${maybe:-default}"  \# default
echo "\${maybe:=default}"  \# default (and sets maybe)
echo "\${maybe:?error}"    \# error if unset
`,
  'parameter_expansion_advanced.sh': `\#!/usr/bin/env bash

\# Advanced parameter expansion examples
set -euo pipefail

echo "== Advanced parameter expansion =="
path="/tmp/file.txt"
echo "\${path\#\#*/}"       \# file.txt
echo "\${path%/*}"        \# /tmp
s2="abba"; echo "\${s2//b/X}"  \# aXXa
`,
  'parameter_expansion_case.sh': `\#!/usr/bin/env bash

\# Case modification in parameter expansion
set -euo pipefail

echo "== Case modification in parameter expansion =="
name="world"
echo "\${name^^}"        \# WORLD
echo "\${name,,}"        \# world
echo "\${name^}"         \# World
`,
  'parameter_expansion_defaults.sh': `\#!/usr/bin/env bash

\# Default values in parameter expansion
set -euo pipefail

echo "== Default values =="
unset maybe
echo "\${maybe:-default}"  \# default
echo "\${maybe:=default}"  \# default (and sets maybe)
echo "\${maybe:?error}"    \# error if unset
`,
  'parameter_expansion_more.sh': `\#!/usr/bin/env bash

\# More parameter expansion examples
set -euo pipefail

echo "== More parameter expansion =="
var="hello world"
echo "\${var\#hello}"      \#  world
echo "\${var%world}"      \# hello 
echo "\${var//o/0}"       \# hell0 w0rld
`,
  'pattern_matching.sh': `\#!/usr/bin/env bash

\# Pattern matching and regex examples
\# Demonstrates [[ ]] test operator with patterns and regex

set -euo pipefail

echo "== [[ pattern and regex ]]"
s="file.txt"
[[ \$s == *.txt ]] && echo pattern-match
[[ \$s =~ ^file\\.[a-z]+\$ ]] && echo regex-match

echo "== extglob =="
shopt -s extglob
f1="file.js"; f2="thing.min.js"
[[ \$f1 == !(*.min).js ]] && echo f1-ok
[[ \$f2 == !(*.min).js ]] || echo f2-filtered

echo "== nocasematch =="
shopt -s nocasematch
word="Foo"; [[ \$word == foo ]] && echo ci-match
`,
  'pattern_matching_basic.sh': `\#!/usr/bin/env bash

\# Basic pattern matching examples
set -euo pipefail

echo "== [[ pattern and regex ]]"
s="file.txt"
[[ \$s == *.txt ]] && echo pattern-match
[[ \$s =~ ^file\\.[a-z]+\$ ]] && echo regex-match
`,
  'pattern_matching_extglob.sh': `\#!/usr/bin/env bash

\# Extended glob examples
set -euo pipefail

echo "== extglob =="
shopt -s extglob
f1="file.js"; f2="thing.min.js"
[[ \$f1 == !(*.min).js ]] && echo f1-ok
[[ \$f2 == !(*.min).js ]] || echo f2-filtered
`,
  'pattern_matching_nocase.sh': `\#!/usr/bin/env bash

\# Case-insensitive matching examples
set -euo pipefail

echo "== nocasematch =="
shopt -s nocasematch
word="Foo"; [[ \$word == foo ]] && echo ci-match
`,
  'pipeline.sh': `\#!/bin/bash

\# Pipeline examples
ls | grep "\\.txt\$" | wc -l
cat file.txt | sort | uniq -c | sort -nr
find . -name "*.sh" | xargs grep -l "function" `,
  'process_substitution.sh': `\#!/usr/bin/env bash

\# Process substitution and here-strings
\# Demonstrates advanced input/output redirection in Bash

set -euo pipefail

echo "== Here-string with grep -o =="
grep -o pattern <<< "some pattern here"

echo "== Process substitution with comm =="
comm -12 <(printf 'a\\nb\\n') <(printf 'b\\nc\\n')

echo "== readarray/mapfile =="
mapfile -t lines < <(printf 'x\\ny\\n')
printf '%s ' "\${lines[@]}"; echo

echo "== More process substitution examples =="
\# Compare sorted outputs
diff <(echo -e "a\\nc\\nb" | sort) <(echo -e "a\\nb\\nd" | sort) || echo "Files differ"

\# Use paste with process substitution
paste <(echo -e "name1\\nname2") <(echo -e "value1\\nvalue2")
`,
  'process_substitution_advanced.sh': `\#!/usr/bin/env bash

\# Advanced process substitution examples
set -euo pipefail

echo "== More process substitution examples =="
\# Compare sorted outputs
diff <(echo -e "a\\nc\\nb" | sort) <(echo -e "a\\nb\\nd" | sort) || echo "Files differ"

\# Use paste with process substitution
paste <(echo -e "name1\\nname2") <(echo -e "value1\\nvalue2")
`,
  'process_substitution_comm.sh': `\#!/usr/bin/env bash

\# Process substitution with comm examples
set -euo pipefail

echo "== Process substitution with comm =="
comm -12 <(printf 'a\\nb\\n') <(printf 'b\\nc\\n')
`,
  'process_substitution_here.sh': `\#!/usr/bin/env bash

\# Here-string examples
set -euo pipefail

echo "== Here-string with grep -o =="
grep -o pattern <<< "some pattern here"
`,
  'process_substitution_mapfile.sh': `\#!/usr/bin/env bash

\# mapfile examples
set -euo pipefail

echo "== readarray/mapfile =="
mapfile -t lines < <(printf 'x\\ny\\n')
printf '%s ' "\${lines[@]}"; echo
`,
  'shell_calling_perl.sh': `\#!/bin/bash

\# Example 1: Simple Perl one-liner to print text
echo "=== Example 1: Simple Perl one-liner ==="
perl -e 'print "Hello from Perl!\\n"'

\# Example 2: Perl script with command line arguments
echo -e "\\n=== Example 2: Perl with arguments ==="
perl -e 'foreach \$arg (@ARGV) { print "Argument: \$arg\\n" }' "first" "second" "third"

\# Example 3: Perl script processing shell variables
echo -e "\\n=== Example 3: Perl processing shell variables ==="
SHELL_VAR="Hello World"
perl -e "print \\"Shell variable: \$ENV{SHELL_VAR}\\n\\""

\# Example 4: Perl script reading from shell pipeline
echo -e "\\n=== Example 4: Perl reading from pipeline ==="
echo "apple\\nbanana\\ncherry" | perl -ne 'chomp; print "Fruit: \$_\\n"'

\# Example 5: Complex Perl script with here document
echo -e "\\n=== Example 5: Perl script with here document ==="
perl << 'EOF'
use strict;
use warnings;

my @numbers = (1, 2, 3, 4, 5);
my \$sum = 0;

foreach my \$num (@numbers) {
    \$sum += \$num;
    print "Added \$num, sum is now \$sum\\n";
}

print "Final sum: \$sum\\n";
EOF

`,
  'simple.sh': `\#!/bin/bash

\# This script demonstrates basic shell functionality
echo "Hello, World!"

\# Valid if statement
if [ -f "test.txt" ]; then
    echo "File exists"
fi

\# Valid for loop
for i in {1..5}; do
    echo \$i
done `,
  'simple_backup.sh': `\#!/bin/bash

\# Simple shell script example
echo "Hello, World!"
ls
echo \`ls\`
\#Lets not consider ls -la at the moment as permissions are OS dependent
\#ls -la
\#grep "pattern" file.txt `,
  'subprocess.sh': `(sleep 1; echo a)&
echo b`,
  'test_ls_star_dot_sh.sh': `\#!/usr/bin/env bash
set -euo pipefail

echo "Testing ls * .sh:"
ls * .sh
`,
  'test_quoted.sh': `echo "Hello, World!"
echo 'Single quoted'
echo "String with \\"escaped\\" quotes"
echo "String with 'single' quotes"
`
};

// Helper function to get all example names
export function getExampleNames() {
  return Object.keys(examples);
}

// Helper function to get example by name
export function getExample(name) {
  return examples[name] || null;
}

// Helper function to get examples grouped by category
export function getExamplesByCategory() {
  const categories = {
    'Basic Examples': ['args.sh', 'simple.sh', 'simple_backup.sh', 'misc.sh', 'subprocess.sh', 'test_quoted.sh', 'cat_EOF.sh', 'file.txt', 'cd..sh', 'test_ls_star_dot_sh.sh'],
    'Control Flow': ['control_flow.sh', 'control_flow_if.sh', 'control_flow_loops.sh', 'control_flow_function.sh'],
    'Pipelines': ['pipeline.sh'],
    'Variables': ['local.sh'],
    'Parameter Expansion': ['parameter_expansion.sh', 'parameter_expansion_advanced.sh', 'parameter_expansion_case.sh', 'parameter_expansion_defaults.sh', 'parameter_expansion_more.sh'],
    'Brace Expansion': ['brace_expansion.sh', 'brace_expansion_basic.sh', 'brace_expansion_advanced.sh', 'brace_expansion_practical.sh'],
    'Arrays': ['arrays.sh', 'arrays_indexed.sh', 'arrays_associative.sh'],
    'Pattern Matching': ['pattern_matching.sh', 'pattern_matching_basic.sh', 'pattern_matching_extglob.sh', 'pattern_matching_nocase.sh'],
    'Process Substitution': ['process_substitution.sh', 'process_substitution_advanced.sh', 'process_substitution_comm.sh', 'process_substitution_mapfile.sh', 'process_substitution_here.sh'],
    'ANSI Quoting': ['ansi_quoting.sh', 'ansi_quoting_basic.sh', 'ansi_quoting_escape.sh', 'ansi_quoting_practical.sh', 'ansi_quoting_unicode.sh'],
    'Grep Examples': ['grep_basic.sh', 'grep_advanced.sh', 'grep_context.sh', 'grep_params.sh', 'grep_regex.sh']
  };
  
  return categories;
}

// Helper function to get examples as JSON (for compatibility with existing code)
export function examplesJson() {
  return JSON.stringify(Object.entries(examples).map(([name, content]) => ({
    name,
    content
  })));
}
