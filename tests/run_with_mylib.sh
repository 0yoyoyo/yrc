#!/bin/bash

if [ -n "$1" ]; then
	NAME=$1
else
    echo "target file is needed!"
    exit -1
fi

EXEC=mylibexec

gcc $NAME -o $EXEC -L mylib/target/debug/ -lmylib
export LD_LIBRARY_PATH=mylib/target/debug/
./$EXEC
rm $EXEC
