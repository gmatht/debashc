#!/bin/bash

# 14. Complex redirects with process substitution
diff <(sort file1.txt) <(sort file2.txt) > comparison.txt 2>&1
