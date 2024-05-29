export MACOSX_DEPLOYMENT_TARGET := 12
MUSL_COMPILER_URL := https://pub-c60a000d68b544109df4fe5837762101.r2.dev/linux-compiler-musl-x86.zip
MUSL_COMPILER_DIR := linux-compiler-musl-x86
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
	@for target in $(TARGETS); do \
    		echo "Сборка для таргета: $$target"; \
    		if [ "$$target" = "x86_64-unknown-linux-musl" ]; then \
    			env TARGET_CC=$(MUSL_COMPILER_DIR)/bin/x86_64-linux-musl-gcc  cargo build --release --target $$target; \
    		else \
    			cargo build --release --target $$target; \
    		fi \
    	done
	@echo "Сборка завершена для всех таргетов."

copy-all:
	@for target in $(TARGETS); do \
		mkdir -p go/libs/$$target; \
		cp target/$$target/release/libtantivy_go.a go/libs/$$target/; \
	done
	@echo "Копирование завершено."

install-musl:
	curl -L $(MUSL_COMPILER_URL) -o /tmp/linux-compiler-musl-x86.zip
	unzip /tmp/linux-compiler-musl-x86.zip -d .

