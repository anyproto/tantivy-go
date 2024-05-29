#export MACOSX_DEPLOYMENT_TARGET := 13
LIB_PATH ?= go/libs
ANYTYPE_PATH ?= ../anytype-heart/deps/libs

TARGETS = \
    x86_64-unknown-linux-musl \
    armv7-linux-androideabi \
    i686-linux-android \
    aarch64-linux-android \
    x86_64-linux-android \
    aarch64-apple-ios \
    x86_64-apple-ios \
    x86_64-apple-darwin \
    aarch64-apple-darwin \
    x86_64-pc-windows-gnu \

lib:
	@cargo build
# to verify otool -l target/debug/libtantivy_go.a  | rg LC_BUILD_VERSION -A4 | rg minos | sort | uniq -c
# output should be
# 795     minos 11.0
#  36     minos 12.0

install: lib
	@mkdir -p $(LIB_PATH)
	@mkdir -p $(ANYTYPE_PATH)
	@cp "target/debug/libtantivy_go.a" "$(LIB_PATH)/libtantivy_go.a"
	@cp "target/debug/libtantivy_go.a" "$(ANYTYPE_PATH)/libtantivy_go.a"

setup:
	@for target in $(TARGETS) ; do \
         rustup target add $$target ; \
     done

build-all:
	env TARGET_CC=x86_64-linux-musl-gcc cargo build --release --target x86_64-unknown-linux-musl
	env TARGET_CC=armv7a-linux-androideabi26-clang cargo build --release --target armv7-linux-androideabi
	env TARGET_CC=i686-linux-android26-clang cargo build --release --target i686-linux-android
	cargo build --release --target aarch64-linux-android
	cargo build --release --target x86_64-linux-android
	cargo build --release --target aarch64-apple-ios
	cargo build --release --target x86_64-apple-ios
	cargo build --release --target x86_64-apple-darwin
	cargo build --release --target aarch64-apple-darwin
	cargo build --release --target x86_64-pc-windows-gnu


copy-all:
	@for target in $(TARGETS); do \
		mkdir -p go/libs/$$target; \
		mkdir -p ../anytype-heart/deps/libs/$$target; \
		cp target/$$target/release/libtantivy_go.a go/libs/$$target/; \
		cp target/$$target/release/libtantivy_go.a ../anytype-heart/deps/libs/$$target/; \
	done
	@echo "Копирование завершено."

install-musl:
	brew tap messense/macos-cross-toolchains && brew install x86_64-unknown-linux-musl

