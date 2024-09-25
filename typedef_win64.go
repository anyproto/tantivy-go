//go:build windows

package tantivy_go

import "C"

type pointerCType = C.ulonglong
type pointerGoType = uint64
