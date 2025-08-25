#!/bin/bash

# 5. Complex case statement with pattern matching
case "$1" in
    [a-z]*) echo "Lowercase start";;
    [A-Z]*) echo "Uppercase start";;
    [0-9]*) echo "Number start";;
    ?) echo "Single character";;
    *) echo "Something else";;
esac
