#!/bin/bash

if [ -n "$1" ]; then
	NAME=$1
else
    echo "target file is needed!"
    exit -1
fi

./$NAME
RESULT="$?"

echo "$RESULT"
