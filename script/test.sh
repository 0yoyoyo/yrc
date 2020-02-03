#!/bin/bash

cd output

expected="$1"
input="$2"

gcc tmp.s -o tmp
./tmp
actual="$?"

if [ "$input" = "$actual" ]; then
    echo "$input => $actual"
else
    echo "$input => $expected expected, but got $actual"
    exit 1
fi

echo OK
