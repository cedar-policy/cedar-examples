#!/bin/bash

curr_dir="$(pwd)"

#Try to set correctly for Mac and Linux machines
if [ "$(uname)" == "Darwin" ]; then
ffi_lib_str="    environment 'CEDAR_JAVA_FFI_LIB', '"$curr_dir"/cedar-java/CedarJavaFFI/target/debug/libcedar_java_ffi.dylib'"
else
ffi_lib_str="    environment 'CEDAR_JAVA_FFI_LIB', '"$curr_dir"/cedar-java/CedarJavaFFI/target/debug/libcedar_java_ffi.so'"
fi

sed "85s;.*;$ffi_lib_str;" "build.gradle" > new_build.gradle
mv new_build.gradle build.gradle
