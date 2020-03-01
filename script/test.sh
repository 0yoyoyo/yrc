#!/bin/bash

cd output

gcc tmp.s -o tmp
./tmp
actual="$?"

echo "$actual"
