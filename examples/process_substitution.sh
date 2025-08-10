#!/usr/bin/env bash

# Process substitution and here-strings
# Demonstrates advanced input/output redirection in Bash

set -euo pipefail

echo "== Here-string with grep -o =="
grep -o pattern <<< "some pattern here"

echo "== Process substitution with comm =="
comm -12 <(printf 'a\nb\n') <(printf 'b\nc\n')

echo "== readarray/mapfile =="
mapfile -t lines < <(printf 'x\ny\n')
printf '%s ' "${lines[@]}"; echo

echo "== More process substitution examples =="
# Compare sorted outputs
diff <(echo -e "a\nc\nb" | sort) <(echo -e "a\nb\nd" | sort) || echo "Files differ"

# Use paste with process substitution
paste <(echo -e "name1\nname2") <(echo -e "value1\nvalue2")
