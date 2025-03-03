# Go Tantivy Bindings

This project provides Go bindings for the [Tantivy](https://github.com/quickwit-oss/tantivy) search engine library. Tantivy is a full-text search engine library written in Rust, and this project aims to make its powerful search capabilities available to Go developers.

The library is thread safe and can be used in a concurrent environment

# Why

The only available FTS engine in the Golang community is [Bleve](https://github.com/blevesearch/bleve), which is surprisingly slow compared to [Tantivy](https://github.com/quickwit-oss/tantivy).
Check out the last link for details on the performance comparison.

![Search Benchmark](https://github.com/quickwit-oss/tantivy/blob/main/doc/assets/images/searchbenchmark.png)
Credits for the image to the Tantivy team

# Our Journey with Tantivy
We've been running it in [Anytype](https://github.com/anyproto/anytype-heart) for over a year across all major platforms and architectures without issues on 32-bit and 64-bit systems, x86 and ARM64, iOS, Android, PC, macOS, and Linux.

## Features
### Jieba Tokenizer
This library includes the Jieba feature by default, which provides Chinese text segmentation. However, if you do not need this functionality, you can build the library without it to save approximately 5MB of the dictionary.
### Golang API to Create Custom Queries for Tantivy
See `searchquerybuilder.go`

## Search quality testing
[Test quality](testquality/README.md)

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

### Nix

`flake.nix` currently provides two versions of `devShell`: musl and gcc.

This command will make a bash shell with all required build dependencies:

```bash
nix develop .
```

Each `devShell` also contains a script which:

- builds rust into `.a` lib
- copies it to `../anytype-heart`
- builds `anytype-heart` `grpcServer`
- copies `grpcServer` to `../anytype-ts` `anytypeHelper`

> [!TIP]
> To enable musl, set `musl = true;` in `flake.nix`.

If you want to debug `tantivy` from `anytype-ts`, with `musl` or `gcc`, this scripts automates all the flow.

All together it would look like:
```bash
nix develop .
tantivy_compile_all_gcc
# or
tantivy_compile_all_musl
```

To check that it works, run `anytype-ts` and try to search something.

> [!NOTE]
> MacOS (Darwin) nix shell is not supported yet
