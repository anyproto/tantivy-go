//go:build !tantivylocal
// +build !tantivylocal

package tantivy_go

/*
#cgo LDFLAGS: -ltantivy_go -lm -pthread -ldl
#include "bindings.h"
#include <stdlib.h>
*/
import "C"