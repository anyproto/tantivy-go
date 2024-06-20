package tantivy

/*
#cgo windows,amd64 LDFLAGS:-L${SRCDIR}/../libs/windows-amd64 -ltantivy_go -lm -pthread -lws2_32 -lbcrypt -lwsock32 -lntdll -luserenv -lsynchronization
#cgo darwin,amd64 LDFLAGS:-L${SRCDIR}/../libs/darwin-amd64 -ltantivy_go -lm -pthread -framework CoreFoundation -framework Security -ldl
#cgo darwin,arm64 LDFLAGS:-L${SRCDIR}/../libs/darwin-arm64 -ltantivy_go -lm -pthread -ldl
#cgo ios,arm64 LDFLAGS:-L${SRCDIR}/../libs/ios-arm64 -ltantivy_go -lm -pthread -ldl
#cgo ios,amd64 LDFLAGS:-L${SRCDIR}/../libs/ios-amd64 -ltantivy_go -lm -pthread -ldl
#cgo android,arm LDFLAGS:-L${SRCDIR}/../libs/android-arm -ltantivy_go -lm -pthread -ldl
#cgo android,386 LDFLAGS:-L${SRCDIR}/../libs/android-386 -ltantivy_go -lm -pthread -ldl
#cgo android,amd64 LDFLAGS:-L${SRCDIR}/../libs/android-amd64 -ltantivy_go -lm -pthread -ldl
#cgo android,arm64 LDFLAGS:-L${SRCDIR}/../libs/android-arm64 -ltantivy_go -lm -pthread -ldl
#cgo linux,amd64 LDFLAGS:-L${SRCDIR}/../libs/linux-amd64-musl -Wl,--allow-multiple-definition -ltantivy_go -lm
#include "bindings.h"
#include <stdlib.h>
*/
import "C"
import (
	"os"
	"sync"
)

const SimpleTokenizer = "simple"
const NgramTokenizer = "ngram"
const EdgeNgramTokenizer = "edge_ngram"
const RawTokenizer = "raw"

var doOnce sync.Once

func LibInit(directive ...string) {
	var initVal string
	doOnce.Do(func() {
		if len(directive) == 0 {
			initVal = "info"
		} else {
			initVal = directive[0]
		}
		os.Setenv("ELV_RUST_LOG", initVal)
		os.Setenv("RUST_BACKTRACE", "full")
		C.init_lib()
	})
}

func Init() uint8 {
	return uint8(C.init_lib())
}
