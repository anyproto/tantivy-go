all:
	@set -e;

setup:
	@rustup target add x86_64-unknown-linux-musl
	@rustup target add armv7-linux-androideabi
	@rustup target add i686-linux-android
	@rustup target add aarch64-linux-android
	@rustup target add x86_64-linux-android
	@rustup target add aarch64-apple-ios
	@rustup target add aarch64-apple-ios-sim
	@rustup target add x86_64-apple-ios
	@rustup target add x86_64-apple-darwin
	@rustup target add x86_64-apple-darwin
	@rustup target add x86_64-pc-windows-gnu

build-linux-amd64-musl:
	env TARGET_CC=x86_64-linux-musl-gcc cargo build --release --target x86_64-unknown-linux-musl

install-linux-amd64-musl: build-linux-amd64-musl
	@mkdir -p ../libs/linux-amd64-musl
	@cp target/x86_64-unknown-linux-musl/release/libtantivy_go.a ../libs/linux-amd64-musl

build-debug-linux-amd64-musl:
	env TARGET_CC=x86_64-linux-musl-gcc cargo build --target x86_64-unknown-linux-musl

install-debug-linux-amd64-musl: build-debug-linux-amd64-musl
	@mkdir -p ../libs/linux-amd64-musl
	@cp target/x86_64-unknown-linux-musl/debug/libtantivy_go.a ../libs/linux-amd64-musl

build-android-arm:
	env TARGET_CC=armv7a-linux-androideabi26-clang cargo build --release --target armv7-linux-androideabi

install-android-arm: build-android-arm
	@mkdir -p ../libs/android-arm
	@cp target/armv7-linux-androideabi/release/libtantivy_go.a ../libs/android-arm

build-debug-android-arm:
	env TARGET_CC=armv7a-linux-androideabi26-clang cargo build --target armv7-linux-androideabi

install-debug-android-arm: build-debug-android-arm
	@mkdir -p ../libs/android-arm
	@cp target/armv7-linux-androideabi/debug/libtantivy_go.a ../libs/android-arm

build-android-386:
	env TARGET_CC=i686-linux-android26-clang cargo build --release --target i686-linux-android

install-android-386: build-android-386
	@mkdir -p ../libs/android-386
	@cp target/i686-linux-android/release/libtantivy_go.a ../libs/android-386

build-debug-android-386:
	env TARGET_CC=i686-linux-android26-clang cargo build --target i686-linux-android

install-debug-android-386: build-debug-android-386
	@mkdir -p ../libs/android-386
	@cp target/i686-linux-android/debug/libtantivy_go.a ../libs/android-386

build-android-arm64:
	cargo build --release --target aarch64-linux-android

install-android-arm64: build-android-arm64
	@mkdir -p ../libs/android-arm64
	@cp target/aarch64-linux-android/release/libtantivy_go.a ../libs/android-arm64

build-debug-android-arm64:
	cargo build --target aarch64-linux-android

install-debug-android-arm64: build-debug-android-arm64
	@mkdir -p ../libs/android-arm64
	@cp target/aarch64-linux-android/debug/libtantivy_go.a ../libs/android-arm64

build-android-amd64:
	cargo build --release --target x86_64-linux-android

install-android-amd64: build-android-amd64
	@mkdir -p ../libs/android-amd64
	@cp target/x86_64-linux-android/release/libtantivy_go.a ../libs/android-amd64

build-debug-android-amd64:
	cargo build --target x86_64-linux-android

install-debug-android-amd64: build-debug-android-amd64
	@mkdir -p ../libs/android-amd64
	@cp target/x86_64-linux-android/debug/libtantivy_go.a ../libs/android-amd64

build-ios-arm64:
	env IPHONEOS_DEPLOYMENT_TARGET=15.0 cargo build --release --target aarch64-apple-ios

install-ios-arm64: build-ios-arm64
	@mkdir -p ../libs/ios-arm64
	@cp target/aarch64-apple-ios/release/libtantivy_go.a ../libs/ios-arm64

build-debug-ios-arm64:
	env IPHONEOS_DEPLOYMENT_TARGET=15.0 cargo build --target aarch64-apple-ios

install-debug-ios-arm64: build-debug-ios-arm64
	@mkdir -p ../libs/ios-arm64
	@cp target/aarch64-apple-ios/debug/libtantivy_go.a ../libs/ios-arm64

build-ios-arm64-sim:
	env IPHONEOS_DEPLOYMENT_TARGET=15.0 cargo build --release --target aarch64-apple-ios-sim

install-ios-arm64-sim: build-ios-arm64-sim
	@mkdir -p ../libs/ios-arm64-sim
	@cp target/aarch64-apple-ios-sim/release/libtantivy_go.a ../libs/ios-arm64-sim

build-debug-ios-arm64-sim:
	env IPHONEOS_DEPLOYMENT_TARGET=15.0 cargo build --target aarch64-apple-ios-sim

install-debug-ios-arm64-sim: build-debug-ios-arm64-sim
	@mkdir -p ../libs/ios-arm64-sim
	@cp target/aarch64-apple-ios-sim/debug/libtantivy_go.a ../libs/ios-arm64-sim

build-ios-amd64:
	env IPHONEOS_DEPLOYMENT_TARGET=15.0 cargo build --release --target x86_64-apple-ios

install-ios-amd64: build-ios-amd64
	@mkdir -p ../libs/ios-amd64
	@cp target/x86_64-apple-ios/release/libtantivy_go.a ../libs/ios-amd64

build-debug-ios-amd64:
	env IPHONEOS_DEPLOYMENT_TARGET=15.0 cargo build --target x86_64-apple-ios

install-debug-ios-amd64: build-debug-ios-amd64
	@mkdir -p ../libs/ios-amd64
	@cp target/x86_64-apple-ios/debug/libtantivy_go.a ../libs/ios-amd64

build-darwin-amd64:
	ENV MACOSX_DEPLOYMENT_TARGET=10.15 cargo build --release --target x86_64-apple-darwin

install-darwin-amd64: build-darwin-amd64
	@mkdir -p ../libs/darwin-amd64
	@cp target/x86_64-apple-darwin/release/libtantivy_go.a ../libs/darwin-amd64

build-debug-darwin-amd64:
	ENV MACOSX_DEPLOYMENT_TARGET=10.15 cargo build --target x86_64-apple-darwin

install-debug-darwin-amd64: build-debug-darwin-amd64
	@mkdir -p ../libs/darwin-amd64
	@cp target/x86_64-apple-darwin/debug/libtantivy_go.a ../libs/darwin-amd64

build-darwin-arm64:
	ENV MACOSX_DEPLOYMENT_TARGET=10.15 cargo build --release --target aarch64-apple-darwin

install-darwin-arm64: build-darwin-arm64
	@mkdir -p ../libs/darwin-arm64
	@cp target/aarch64-apple-darwin/release/libtantivy_go.a ../libs/darwin-arm64

build-debug-darwin-arm64:
	ENV MACOSX_DEPLOYMENT_TARGET=10.15 cargo build --target aarch64-apple-darwin

install-debug-darwin-arm64: build-debug-darwin-arm64
	@mkdir -p ../libs/darwin-arm64
	@cp target/aarch64-apple-darwin/debug/libtantivy_go.a ../libs/darwin-arm64

build-windows-amd64:
	cargo build --release --target x86_64-pc-windows-gnu

install-windows-amd64: build-windows-amd64
	@mkdir -p ../libs/windows-amd64
	@cp target/x86_64-pc-windows-gnu/release/libtantivy_go.a ../libs/windows-amd64

build-debug-windows-amd64:
	cargo build --target x86_64-pc-windows-gnu

install-debug-windows-amd64: build-debug-windows-amd64
	@mkdir -p ../libs/windows-amd64
	@cp target/x86_64-pc-windows-gnu/debug/libtantivy_go.a ../libs/windows-amd64

install-all: \
    install-linux-amd64-musl \
    install-android-arm \
    install-android-386 \
    install-android-arm64 \
    install-android-amd64 \
    install-ios-arm64 \
    install-ios-arm64-sim \
    install-ios-amd64 \
    install-darwin-amd64 \
    install-darwin-arm64 \
    install-windows-amd64

install-debug-all: \
    install-debug-linux-amd64-musl \
    install-debug-android-arm \
    install-debug-android-386 \
    install-debug-android-arm64 \
    install-debug-android-amd64 \
    install-debug-ios-arm64 \
    install-debug-ios-arm64-sim \
    install-debug-ios-amd64 \
    install-debug-darwin-amd64 \
    install-debug-darwin-arm64 \
    install-debug-windows-amd64

### Download release
REPO := anyproto/tantivy-go
VERSION := v0.1.0
OUTPUT_DIR := ../libs

TANTIVY_LIBS := android-386.tar.gz \
         android-amd64.tar.gz \
         android-arm.tar.gz \
         android-arm64.tar.gz \
         darwin-amd64.tar.gz \
         darwin-arm64.tar.gz \
         ios-amd64.tar.gz \
         ios-arm64.tar.gz \
         ios-arm64-sim.tar.gz \
         linux-amd64-musl.tar.gz \
         windows-amd64.tar.gz

define download_tantivy_lib
	curl -L -o $(OUTPUT_DIR)/$(1) https://github.com/$(REPO)/releases/download/$(VERSION)/$(1)
endef

download-tantivy-all: $(TANTIVY_LIBS)

$(TANTIVY_LIBS):
	@mkdir -p $(OUTPUT_DIR)/$(shell echo $@ | cut -d'.' -f1)
	$(call download_tantivy_lib,$@)
	@tar -C $(OUTPUT_DIR)/$(shell echo $@ | cut -d'.' -f1) -xvzf $(OUTPUT_DIR)/$@
	@rm -f $(OUTPUT_DIR)/$@
	@echo "Extracted $@"
