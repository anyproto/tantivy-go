//go:build (amd64 || arm64) && unix

package tantivy_go

import "C"

type pointerCType = C.ulong
type pointerGoType = uint64
