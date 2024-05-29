package tantivy

/*
#cgo windows,amd64 LDFLAGS:-L${SRCDIR}/../libs/x86_64-pc-windows-gnu -ltantivy_jpc -lm -pthread -lws2_32 -lbcrypt -lwsock32 -lntdll -luserenv -lsynchronization
#cgo darwin,amd64 LDFLAGS:-L${SRCDIR}/../libs/x86_64-apple-darwin -ltantivy_jpc -lm -pthread -framework CoreFoundation -framework Security -ldl
#cgo darwin,arm64 LDFLAGS:-L${SRCDIR}/../libs/aarch64-apple-darwin -ltantivy_jpc -lm -pthread -ldl
#cgo ios,arm64 LDFLAGS:-L${SRCDIR}/../libs/aarch64-apple-ios -ltantivy_jpc -lm -pthread -ldl
#cgo ios,amd64 LDFLAGS:-L${SRCDIR}/../libs/x86_64-apple-ios -ltantivy_jpc -lm -pthread -ldl
#cgo android,arm LDFLAGS:-L${SRCDIR}/../libs/armv7-linux-androideabi -ltantivy_jpc -lm -pthread -ldl
#cgo android,386 LDFLAGS:-L${SRCDIR}/../libs/i686-linux-android -ltantivy_jpc -lm -pthread -ldl
#cgo android,amd64 LDFLAGS:-L${SRCDIR}/../libs/x86_64-linux-android -ltantivy_jpc -lm -pthread -ldl
#cgo android,arm64 LDFLAGS:-L${SRCDIR}/../libs/aarch64-linux-android -ltantivy_jpc -lm -pthread -ldl
#cgo linux,amd64 LDFLAGS:-L${SRCDIR}/../libs/i686-linux-android -Wl,--allow-multiple-definition -ltantivy_jpc -lm
#include "bindings.h"
#include <stdlib.h>
*/
import "C"
import (
	"os"
	"sync"
)

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
		C.init_lib()
	})
}

func Init() uint8 {
	return uint8(C.init_lib())
}
