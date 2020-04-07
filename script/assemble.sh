#!/bin/bash

if [ -n "$1" ]; then
	NAME=$1
else
	NAME=tmp
fi

cd output

OPTION="-no-pie"
gcc $NAME.s -o $NAME $OPTION
./$NAME
RESULT="$?"

echo "$RESULT"
