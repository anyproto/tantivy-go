//go:build arm || 386

package tantivy

import "C"

type pointerCType = C.uint
type pointerGoType = uint64
