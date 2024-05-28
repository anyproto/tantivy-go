export MACOSX_DEPLOYMENT_TARGET=12
LIB_PATH ?= go/libs
ANYTYPE_PATH ?= ../anytype-heart/deps/libs
lib:
	@cargo build
# to verify otool -l target/debug/libtantivy_go.a  | rg LC_BUILD_VERSION -A4 | rg minos | sort | uniq -c
# output should be
# 795     minos 11.0
#  36     minos 12.0

install:
	@mkdir -p $(LIB_PATH)
	@mkdir -p $(ANYTYPE_PATH)
	@cp "target/debug/libtantivy_go.a" "$(LIB_PATH)/libtantivy_go.a"
	@cp "target/debug/libtantivy_go.a" "$(ANYTYPE_PATH)/libtantivy_go.a"