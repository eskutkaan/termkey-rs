#!/bin/bash

repository="https://github.com/mathall/libtermkey.git"
git_ref="${1-master}"
work_dir="termkey-c/$git_ref"

if [ ! -d "$work_dir" ]; then
  git clone --depth 1 $repository -b $git_ref $work_dir
fi

make -j2 -C $work_dir
