//go:build windows

package tantivy

import "C"

type pointerCType = C.ulonglong
type pointerGoType = uint64
