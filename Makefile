all:
	@set -e;

install-musl:
	brew tap messense/macos-cross-toolchains && brew install x86_64-unknown-linux-musl

setup:
	@for target in $(TARGET_TO_GOOS_GOARCH) ; do \
  		 target=$$(echo $$target | cut -d: -f1); \
         rustup target add $$target ; \
     done

build-linux-amd64-musl:
	env TARGET_CC=x86_64-linux-musl-gcc cargo build --release --target x86_64-unknown-linux-musl

install-linux-amd64-musl: build-linux-amd64-musl
	@mkdir -p go/libs/linux-amd64-musl
	@cp target/x86_64-unknown-linux-musl/release/libtantivy_go.a go/libs/linux-amd64-musl

build-debug-linux-amd64-musl:
	env TARGET_CC=x86_64-linux-musl-gcc cargo build --target x86_64-unknown-linux-musl

install-debug-linux-amd64-musl: build-debug-linux-amd64-musl
	@mkdir -p go/libs/linux-amd64-musl
	@cp target/x86_64-unknown-linux-musl/debug/libtantivy_go.a go/libs/linux-amd64-musl

build-android-arm:
	env TARGET_CC=armv7a-linux-androideabi26-clang cargo build --release --target armv7-linux-androideabi

install-android-arm: build-android-arm
	@mkdir -p go/libs/android-arm
	@cp target/armv7-linux-androideabi/release/libtantivy_go.a go/libs/android-arm

build-debug-android-arm:
	env TARGET_CC=armv7a-linux-androideabi26-clang cargo build --target armv7-linux-androideabi

install-debug-android-arm: build-debug-android-arm
	@mkdir -p go/libs/android-arm
	@cp target/armv7-linux-androideabi/debug/libtantivy_go.a go/libs/android-arm

build-android-386:
	env TARGET_CC=i686-linux-android26-clang cargo build --release --target i686-linux-android

install-android-386: build-android-386
	@mkdir -p go/libs/android-386
	@cp target/i686-linux-android/release/libtantivy_go.a go/libs/android-386

build-debug-android-386:
	env TARGET_CC=i686-linux-android26-clang cargo build --target i686-linux-android

install-debug-android-386: build-debug-android-386
	@mkdir -p go/libs/android-386
	@cp target/i686-linux-android/debug/libtantivy_go.a go/libs/android-386

build-android-arm64:
	cargo build --release --target aarch64-linux-android

install-android-arm64: build-android-arm64
	@mkdir -p go/libs/android-arm64
	@cp target/aarch64-linux-android/release/libtantivy_go.a go/libs/android-arm64

build-debug-android-arm64:
	cargo build --target aarch64-linux-android

install-debug-android-arm64: build-debug-android-arm64
	@mkdir -p go/libs/android-arm64
	@cp target/aarch64-linux-android/debug/libtantivy_go.a go/libs/android-arm64

build-android-amd64:
	cargo build --release --target x86_64-linux-android

install-android-amd64: build-android-amd64
	@mkdir -p go/libs/android-amd64
	@cp target/x86_64-linux-android/release/libtantivy_go.a go/libs/android-amd64

build-debug-android-amd64:
	cargo build --target x86_64-linux-android

install-debug-android-amd64: build-debug-android-amd64
	@mkdir -p go/libs/android-amd64
	@cp target/x86_64-linux-android/debug/libtantivy_go.a go/libs/android-amd64

build-ios-arm64:
	cargo build --release --target aarch64-apple-ios

install-ios-arm64: build-ios-arm64
	@mkdir -p go/libs/ios-arm64
	@cp target/aarch64-apple-ios/release/libtantivy_go.a go/libs/ios-arm64

build-debug-ios-arm64:
	cargo build --target aarch64-apple-ios

install-debug-ios-arm64: build-debug-ios-arm64
	@mkdir -p go/libs/ios-arm64
	@cp target/aarch64-apple-ios/debug/libtantivy_go.a go/libs/ios-arm64

build-ios-amd64:
	cargo build --release --target x86_64-apple-ios

install-ios-amd64: build-ios-amd64
	@mkdir -p go/libs/ios-amd64
	@cp target/x86_64-apple-ios/release/libtantivy_go.a go/libs/ios-amd64

build-debug-ios-amd64:
	cargo build --target x86_64-apple-ios

install-debug-ios-amd64: build-debug-ios-amd64
	@mkdir -p go/libs/ios-amd64
	@cp target/x86_64-apple-ios/debug/libtantivy_go.a go/libs/ios-amd64

build-darwin-amd64:
	ENV MACOSX_DEPLOYMENT_TARGET=13 cargo build --release --target x86_64-apple-darwin

install-darwin-amd64: build-darwin-amd64
	@mkdir -p go/libs/darwin-amd64
	@cp target/x86_64-apple-darwin/release/libtantivy_go.a go/libs/darwin-amd64

build-debug-darwin-amd64:
	ENV MACOSX_DEPLOYMENT_TARGET=13 cargo build --target x86_64-apple-darwin

install-debug-darwin-amd64: build-debug-darwin-amd64
	@mkdir -p go/libs/darwin-amd64
	@cp target/x86_64-apple-darwin/debug/libtantivy_go.a go/libs/darwin-amd64

build-darwin-arm64:
	ENV MACOSX_DEPLOYMENT_TARGET=13 cargo build --release --target aarch64-apple-darwin

install-darwin-arm64: build-darwin-arm64
	@mkdir -p go/libs/darwin-arm64
	@cp target/aarch64-apple-darwin/release/libtantivy_go.a go/libs/darwin-arm64

build-debug-darwin-arm64:
	ENV MACOSX_DEPLOYMENT_TARGET=13 cargo build --target aarch64-apple-darwin

install-debug-darwin-arm64: build-debug-darwin-arm64
	@mkdir -p go/libs/darwin-arm64
	@cp target/aarch64-apple-darwin/debug/libtantivy_go.a go/libs/darwin-arm64

build-windows-amd64:
	cargo build --release --target x86_64-pc-windows-gnu

install-windows-amd64: build-windows-amd64
	@mkdir -p go/libs/windows-amd64
	@cp target/x86_64-pc-windows-gnu/release/libtantivy_go.a go/libs/windows-amd64

build-debug-windows-amd64:
	cargo build --target x86_64-pc-windows-gnu

install-debug-windows-amd64: build-debug-windows-amd64
	@mkdir -p go/libs/windows-amd64
	@cp target/x86_64-pc-windows-gnu/debug/libtantivy_go.a go/libs/windows-amd64

install-all: \
    install-linux-amd64-musl \
    install-android-arm \
    install-android-386 \
    install-android-arm64 \
    install-android-amd64 \
    install-ios-arm64 \
    install-ios-amd64 \
    install-darwin-amd64 \
    install-darwin-arm64 \
    install-windows-amd64

install-all-debug: \
    install-debug-linux-amd64-musl \
    install-debug-android-arm \
    install-debug-android-386 \
    install-debug-android-arm64 \
    install-debug-android-amd64 \
    install-debug-ios-arm64 \
    install-debug-ios-amd64 \
    install-debug-darwin-amd64 \
    install-debug-darwin-arm64 \
    install-debug-windows-amd64