#!/bin/bash

# 2. Nested brace expansions with ranges and sequences
echo "Files: " file_{a..z}_{1..10,20,30..40}.{txt,log,dat}
