#!/bin/bash

# 5. Command substitution within parameter expansion
echo "Testing nested command substitution..."
echo "Current dir: ${PWD:-$(pwd)}" | tr -d '/\\' | grep -o '.....$' #ignore differences between WSL and Windows
#echo "User: ${USER:-$(whoami)}"
