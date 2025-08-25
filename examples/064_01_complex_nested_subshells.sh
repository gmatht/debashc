#!/bin/bash

# 1. Complex nested subshells with process substitution
diff <(sort <(grep -v "^#" /etc/passwd | cut -d: -f1)) <(sort <(grep -v "^#" /etc/group | cut -d: -f1))
