#!/bin/bash

set -e

if [ -n "$1" ]; then
	NAME=$1
else
	NAME=tmp
fi

cd output

gcc $NAME.s -o $NAME
./$NAME
RESULT="$?"

echo "$RESULT"
