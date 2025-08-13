#!/bin/bash

# Loop examples
for i in {1..5}; do
    echo "Number: $i"
done

for i in {1..3}; do j=$((j+1)); done; echo $j

while [ $i -lt 10 ]; do
    echo "Counter: $i"
    i=$((i + 1))
done
