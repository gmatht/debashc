#!/usr/bin/env bash

# Miscellaneous shell features not covered elsewhere

echo "== Parameter expansion defaults =="
unset MAYBE
echo "${MAYBE:-default}"

echo "== Command substitution =="
now=$(date +%s)
echo "Now: $now"

echo "== Arithmetic expansion =="
x=3; y=7
echo $(( x * (y + 1) ))

echo "== Brace expansion =="
echo {a..c}

echo "== Here-string =="
grep -o foo <<< "foo bar baz"

echo "== Process substitution =="
diff <(printf 'a\n') <(printf 'b\n') || true

echo "== Arrays quick demo =="
arr=(alpha beta gamma)
echo "${arr[2]}"

echo "== Subshell =="
( echo inside-subshell )


