#!/bin/bash

# a small script for precompiling a specified shader and putting it into the compiled/ directory

filename="$1"
name="${filename%.*}"
extension="${filename#*.}"

glslc $filename
if [ "$?" == 0 ]; then
  mv a.spv "compiled/${name}.${extension}.spv"
else
  echo "Compilation failed."
fi
