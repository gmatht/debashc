#!/bin/bash

# 9. Process substitution in pipeline with complex commands
paste <(cut -d: -f1 /etc/passwd | sort) <(cut -d: -f3 /etc/passwd | sort -n) | head -10
