#!/bin/bash

# 19. Function call with complex argument processing
complex_function \
    --long-option="value with spaces" \
    --array-option=("item1" "item2" "item3") \
    --flag \
    "positional argument" \
    "${var:-default}" \
    "$(echo "computed")"
