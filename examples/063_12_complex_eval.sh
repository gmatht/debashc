#!/bin/bash

# 12. Complex eval with nested expansions
eval "result=\$(( \${var:-0} + \${array[\${index:-0}]:-0} ))"
echo "Eval result: $result"
