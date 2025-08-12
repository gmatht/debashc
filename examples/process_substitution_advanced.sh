#!/usr/bin/env bash

# Advanced process substitution examples
set -euo pipefail

echo "== More process substitution examples =="
# Compare sorted outputs
diff <(echo -e "a\nc\nb" | sort) <(echo -e "a\nb\nd" | sort) || echo "Files differ"

# Use paste with process substitution
paste <(echo -e "name1\nname2") <(echo -e "value1\nvalue2")
