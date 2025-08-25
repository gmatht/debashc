#!/bin/bash

# 21. Complex string interpolation with multiple variables
message="Hello ${USER:-guest} from ${HOSTNAME:-localhost}"
echo "$message"
