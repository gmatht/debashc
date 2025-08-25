#!/bin/bash

# 13. Nested subshells with complex command chains
(
    (
        (
            echo "Level 3"
            (echo "Level 4"; echo "Still level 4")
        ) | grep "Level"
    ) | sed 's/Level/Depth/'
) | wc -l
