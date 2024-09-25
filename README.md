# Go Tantivy Bindings

This project provides Go bindings for the [Tantivy](https://github.com/quickwit-oss/tantivy) search engine library. Tantivy is a full-text search engine library written in Rust, and this project aims to make its powerful search capabilities available to Go developers.

## Disclaimer
This project is still in development and might not be ready for production use. The API is subject to change, and the project may contain bugs. Please use with caution.

## Installation

```bash
go get github.com/anyproto/tantivy-go
```

Ensure your libraries are in your `ld` path.

### Example Run
- Run `make download-tantivy-all` inside the `rust` folder
- Run `main.go` in the `example` folder

## Development
Development and compilation are done on MacBooks and for Apple platforms. Therefore, the development steps provided are for macOS.

### Install environment
- [Install rustup](https://rust-lang.github.io/rustup/installation/other.html)
- Install Rust architectures: `make setup`
- Add Android libraries to your path: `export PATH=$PATH:$ANDROID_HOME/tools:$ANDROID_HOME/emulator:$ANDROID_HOME/platform-tools:$ANDROID_HOME/ndk/25.2.9519653/toolchains/llvm/prebuilt/darwin-x86_64/bin`
- Install Windows compiler:  `brew install mingw-w64`
- Install musl: `brew tap messense/macos-cross-toolchains && brew install x86_64-unknown-linux-musl`

### Install rust libraries
Run inside the `rust` folder:

`make install-all` - install release versions for all platforms

`make install-debug-all` - install debug versions for all platforms

`make install-ARCH-GOOS` - install release version for ARCH GOOS

`make install-debug-ARCH-GOOS` - install debug version for ARCH GOOS

### GCC support
To be done

### Validate min macos version

`otool -l libtantivy_go.a  | rg LC_BUILD_VERSION -A4 | rg minos | sort | uniq -c`
Expected output:
```
 880     minos 11.0
```

### Possible troubleshooting
If you experience SIGSEGV issues with musl or windows, try adding these flags to the linker:
```
-extldflags '-static -Wl,-z stack-size=1000000'
```
