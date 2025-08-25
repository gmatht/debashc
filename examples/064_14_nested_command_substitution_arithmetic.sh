#!/bin/bash

# 14. Nested command substitution with arithmetic
result=$(echo $(( $(wc -l < /etc/passwd) + $(wc -l < /etc/group) )))
echo "Total lines: $result"
