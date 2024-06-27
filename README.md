# Disclaimer
Do not use in production

# tantivy-go
Tantivy go bindings

# To make library
make lib

# Versions
rust 

add android libs to path
like
export PATH=$PATH:$ANDROID_HOME/tools:$ANDROID_HOME/emulator:$ANDROID_HOME/platform-tools:$ANDROID_HOME/ndk/25.2.9519653/toolchains/llvm/prebuilt/darwin-x86_64/bin

brew install mingw-w64

work with gcc without musl?

# to verify otool -l target/debug/libtantivy_go.a  | rg LC_BUILD_VERSION -A4 | rg minos | sort | uniq -c
# output should be
# 795     minos 11.0
#  36     minos 12.0