#!/bin/bash

# 12. Complex string interpolation with nested expansions
echo "Testing complex string interpolation..."
message="Hello, ${USER:-$(whoami)}! Your home is ${HOME:-$(echo ~)}"
echo "$message"
