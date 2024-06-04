#export MACOSX_DEPLOYMENT_TARGET := 13
LIB_PATH ?= go/libs
ANYTYPE_PATH ?= ../anytype-heart/deps/libs

# Define the mapping from targets to GOOS-GOARCH
TARGET_TO_GOOS_GOARCH = \
    x86_64-unknown-linux-musl:linux-amd64-musl \
    armv7-linux-androideabi:android-arm \
    i686-linux-android:android-386 \
    aarch64-linux-android:android-arm64 \
    x86_64-linux-android:android-amd64 \
    aarch64-apple-ios:ios-arm64 \
    x86_64-apple-ios:ios-amd64 \
    x86_64-apple-darwin:darwin-amd64 \
    aarch64-apple-darwin:darwin-arm64 \
    x86_64-pc-windows-gnu:windows-amd64


# to verify otool -l target/debug/libtantivy_go.a  | rg LC_BUILD_VERSION -A4 | rg minos | sort | uniq -c
# output should be
# 795     minos 11.0
#  36     minos 12.0


setup:
	@for target in $(TARGET_TO_GOOS_GOARCH) ; do \
  		 target=$$(echo $$target | cut -d: -f1); \
         rustup target add $$target ; \
     done

build-all:
	@set -e; \
	env TARGET_CC=x86_64-linux-musl-gcc cargo build --release --target x86_64-unknown-linux-musl; \
	env TARGET_CC=armv7a-linux-androideabi26-clang cargo build --release --target armv7-linux-androideabi; \
	env TARGET_CC=i686-linux-android26-clang cargo build --release --target i686-linux-android; \
	cargo build --release --target aarch64-linux-android; \
	cargo build --release --target x86_64-linux-android; \
	cargo build --release --target aarch64-apple-ios; \
	cargo build --release --target x86_64-apple-ios; \
	cargo build --release --target x86_64-apple-darwin; \
	cargo build --release --target aarch64-apple-darwin; \
	cargo build --release --target x86_64-pc-windows-gnu


#архитектуры обозвать по гошному
#gcc и мусл соберется?
# Function to convert target to GOOS-GOARCH

copy-all:
	@echo "TARGET_TO_GOOS_GOARCH: $(TARGET_TO_GOOS_GOARCH)"
	@set -e; \
	for kv in $(TARGET_TO_GOOS_GOARCH); do \
		target=$$(echo $$kv | cut -d: -f1); \
		goos_goarch=$$(echo $$kv | cut -d: -f2); \
		echo "Target: $$target -> GOOS-GOARCH: $$goos_goarch"; \
		if [ -n "$$goos_goarch" ]; then \
			mkdir -p go/libs/$$goos_goarch; \
			mkdir -p ../anytype-heart/deps/libs/$$goos_goarch; \
			cp target/$$target/release/libtantivy_go.a go/libs/$$goos_goarch/; \
			cp target/$$target/release/libtantivy_go.a ../anytype-heart/deps/libs/$$goos_goarch/; \
		else \
			echo "No mapping for target: $$target"; \
		fi; \
	done

install-musl:
	brew tap messense/macos-cross-toolchains && brew install x86_64-unknown-linux-musl

