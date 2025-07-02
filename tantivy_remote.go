//go:build !tantivylocal
// +build !tantivylocal

package tantivy_go

/*
#cgo windows,amd64 LDFLAGS: -ltantivy_go -lm -pthread -lws2_32 -lbcrypt -lntdll -luserenv
#cgo darwin,amd64 LDFLAGS: -ltantivy_go -lm -pthread -framework CoreFoundation -framework Security -ldl
#cgo darwin,arm64 LDFLAGS: -ltantivy_go -lm -pthread -ldl
#cgo ios,arm64 LDFLAGS: -ltantivy_go -lm -pthread -ldl
#cgo ios,amd64 LDFLAGS: -ltantivy_go -lm -pthread -ldl
#cgo android LDFLAGS: -ltantivy_go -lm -pthread -ldl
#cgo linux,amd64 LDFLAGS: -Wl,--allow-multiple-definition -ltantivy_go -lm
#cgo linux,arm64 LDFLAGS: -Wl,--allow-multiple-definition -ltantivy_go -lm
#include "bindings.h"
#include <stdlib.h>
*/
import "C"
