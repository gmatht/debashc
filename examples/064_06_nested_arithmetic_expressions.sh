#!/bin/bash

# 6. Nested arithmetic expressions
((i = 1 + (2 * 3) / 4))
((j = i++ + ++i))
echo "i=$i, j=$j"
