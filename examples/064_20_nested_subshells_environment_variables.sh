#!/bin/bash

# 20. Nested subshells with environment variables
(
    export DEBUG=1
    export LOG_LEVEL=verbose
    
    (
        unset DEBUG
        echo "Inner: LOG_LEVEL=$LOG_LEVEL, DEBUG=${DEBUG:-unset}"
    )
    
    echo "Outer: LOG_LEVEL=$LOG_LEVEL, DEBUG=$DEBUG"
)
