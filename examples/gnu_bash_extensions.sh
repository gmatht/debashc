#!/usr/bin/env bash

# GNU Bash extensions beyond POSIX sh, with real runnable examples.
# Deterministic and self-contained (no network or repo-dependent files).

set -euo pipefail

echo "== Indexed arrays =="
arr=(one two three)
echo "${arr[1]}"        # two
echo "${#arr[@]}"       # 3
for x in "${arr[@]}"; do printf "%s " "$x"; done; echo

echo "== Associative arrays =="
declare -A map
map[foo]=bar
map[answer]=42
echo "${map[foo]}"      # bar
echo "${map[answer]}"   # 42

echo "== [[ pattern and regex ]]"
s="file.txt"
[[ $s == *.txt ]] && echo pattern-match
[[ $s =~ ^file\.[a-z]+$ ]] && echo regex-match

echo "== Arithmetic (( )) =="
#i=0; (( i++ )); (( i > 0 )) && echo FAIL || echo PASS
#echo "i is $i"

echo "== Brace expansion =="
echo {1..5}
echo {a..c}
echo {00..04..2}

echo "== Here-string with grep -o =="
grep -o pattern <<< "some pattern here"

echo "== Process substitution with comm =="
comm -12 <(printf 'a\nb\n') <(printf 'b\nc\n')

echo "== readarray/mapfile =="
mapfile -t lines < <(printf 'x\ny\n')
printf '%s ' "${lines[@]}"; echo

echo "== extglob =="
shopt -s extglob
f1="file.js"; f2="thing.min.js"
[[ $f1 == !(*.min).js ]] && echo f1-ok
[[ $f2 == !(*.min).js ]] || echo f2-filtered

echo "== Case modification in parameter expansion =="
name="world"
echo "${name^^}"        # WORLD
echo "${name,,}"        # world
echo "${name^}"         # World

echo "== Advanced parameter expansion =="
path="/tmp/file.txt"
echo "${path##*/}"       # file.txt
echo "${path%/*}"        # /tmp
s2="abba"; echo "${s2//b/X}"  # aXXa

echo "== ANSI-C quoting =="
echo $'line1\nline2\tTabbed'

echo "== nocasematch =="
shopt -s nocasematch
word="Foo"; [[ $word == foo ]] && echo ci-match

# 1) Indexed arrays
# arr=(one two three)
# echo "${arr[1]}"       # two
# echo "${#arr[@]}"      # 3
# for x in "${arr[@]}"; do echo "$x"; done

# 2) Associative arrays
# declare -A map
# map[foo]=bar
# map[answer]=42
# echo "${map[foo]}"     # bar
# for k in "${!map[@]}"; do echo "$k => ${map[$k]}"; done

# 3) [[ ... ]] test with pattern and regex
# s="file.txt"
# if [[ $s == *.txt ]]; then echo match; fi
# if [[ $s =~ ^file\.[a-z]+$ ]]; then echo regex; fi

# 4) Arithmetic with (( ... ))
# i=0; (( i++ )); if (( i > 0 )); then echo gt0; fi

# 5) Brace expansion
# echo {1..5}
# echo {a..e}
# echo {00..10..2}

# 6) Here-strings
# grep pattern <<< "some pattern here"

# 7) Process substitution
# diff <(sort a.txt) <(sort b.txt)
# paste <(cut -d: -f1 /etc/passwd) <(cut -d: -f7 /etc/passwd)

# 8) readarray/mapfile
# mapfile -t lines < <(grep -v '^#' config.txt)
# printf '%s\n' "${lines[@]}"

# 9) coproc
# coproc NETCAT { nc example.com 80; }
# echo -e "GET / HTTP/1.0\r\n\r\n" >&"${NETCAT[1]}"
# head -n1 <&"${NETCAT[0]}"

# 10) Extended globbing
# shopt -s extglob
# ls !(*.min).js
# case $var in
#   @(foo|bar)) echo alt ;;
#   +([0-9])) echo digits ;;
# esac

# 11) Globstar (recursive **)
# shopt -s globstar
# printf '%s\n' **/*.rs

# 12) Case modification in parameter expansion
# name="world"
# echo "${name^^}"     # WORLD
# echo "${name,,}"     # world
# echo "${name^}"      # World
# echo "${name,}"      # world (no change)

# 13) Advanced parameter expansion
# path="/tmp/file.txt"
# echo "${path##*/}"     # file.txt (strip longest prefix)
# echo "${path%/*}"      # /tmp (strip shortest suffix)
# s="abba"; echo "${s//b/X}"  # aXXa (global replace)

# 14) ANSI-C quoting
# echo $'line1\nline2\tTabbed'

# 15) PIPESTATUS array and pipefail
# set -o pipefail
# false | true | cat
# echo "${PIPESTATUS[@]}"  # e.g., 1 0 0

# 16) select builtin (menus)
# PS3='Choose: '
# select ans in start stop quit; do echo "You chose $ans"; break; done

# 17) nocasematch (case-insensitive [[ matching })
# shopt -s nocasematch
# word="Foo"; if [[ $word == foo ]]; then echo ci-match; fi

# Note: These are Bash features and not guaranteed in POSIX sh.


