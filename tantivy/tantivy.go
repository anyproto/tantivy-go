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
	"fmt"
	"sync"
)

const TokenizerSimple = "simple"
const TokenizerNgram = "ngram"
const TokenizerEdgeNgram = "edge_ngram"
const TokenizerRaw = "raw"

var doOnce sync.Once

// LibInit initializes the library with an optional directive.
//
// Parameters:
//   - directive: A variadic parameter that allows specifying an initialization directive.
//     If no directive is provided, the default value "info" is used.
//
// Returns:
// - An error if the initialization fails.
func LibInit(directive ...string) error {
	var initVal string
	var err error
	doOnce.Do(func() {
		if len(directive) == 0 {
			initVal = "info"
		} else {
			initVal = directive[0]
		}

		cInitVal := C.CString(initVal)
		defer C.string_free(cInitVal)
		var errBuffer *C.char
		C.init_lib(cInitVal, &errBuffer)

		errorMessage := C.GoString(errBuffer)
		defer C.string_free(errBuffer)

		if errorMessage != "" {
			err = fmt.Errorf(errorMessage)
		}
	})
	return err
}
