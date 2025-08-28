#!/bin/bash

# Simple shell script example
echo "Hello, World!"
#TODO: Support multi-column output
ls -1
#This should be a single token, not two.
#AST_MUST_CONTAIN: [Literal("-1")]
echo `ls`
#Lets not consider ls -la at the moment as permissions are OS dependent
#ls -la
#grep "pattern" file.txt 