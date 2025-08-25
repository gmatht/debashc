#!/bin/bash

# 4. Complex parameter expansion with nested braces
echo "${var:-${default:-${fallback:-$(echo "computed")}}}"
echo "${array[${index}]:-${default[@]:0:2}}"
echo "${!prefix*[@]:0:3}"
