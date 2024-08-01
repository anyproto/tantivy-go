//go:build (amd64 || arm64) && unix

package tantivy

import "C"

type pointerCType = C.ulong
type pointerGoType = uint64
