#!/usr/bin/env bash

# ANSI-C quoting and special character examples
# Demonstrates escape sequences and special character handling

set -euo pipefail

echo "== ANSI-C quoting =="
echo $'line1\nline2\tTabbed'

echo "== Escape sequences =="
echo $'bell\a'
echo $'backspace\b'
echo $'formfeed\f'
echo $'newline\n'
echo $'carriage\rreturn'
echo $'tab\tseparated'
echo $'vertical\vtab'

echo "== Unicode and hex =="
echo $'\u0048\u0065\u006c\u006c\u006f'  # Hello
echo $'\x48\x65\x6c\x6c\x6f'            # Hello

echo "== Practical examples =="
# Create a formatted table
printf $'%-10s %-10s %s\n' "Name" "Age" "City"
printf $'%-10s %-10s %s\n' "John" "25" "NYC"
printf $'%-10s %-10s %s\n' "Jane" "30" "LA"
