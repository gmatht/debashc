#!/bin/bash

# 15. Complex pipeline with multiple redirects
grep -v "^#" /etc/passwd | cut -d: -f1,3 | sort -t: -k2 -n | head -5 > users.txt 2> errors.log
echo "Pipeline completed"
