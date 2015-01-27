#!/bin/bash

if [ ! -d "termkey-c" ]; then
  git clone --depth 1 git://github.com/mathall/libtermkey.git -b o11c termkey-c
fi
make -j2 -C termkey-c
