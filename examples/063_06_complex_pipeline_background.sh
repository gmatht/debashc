#!/bin/bash

# 6. Complex pipeline with background processes and subshells
(echo "Starting"; sleep 1) &
(echo "Processing"; sleep 2) &
wait
echo "All done"
