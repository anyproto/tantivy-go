package tantivy

//#include "bindings.h"
import "C"
import (
	"errors"
	"fmt"
)

type SearchResult struct{ ptr *C.SearchResult }

func (r *SearchResult) Get(index uint64) (*Document, error) {
	var errBuffer *C.char
	ptr := C.search_result_get_doc(r.ptr, C.uintptr_t(index), &errBuffer)
	if ptr == nil {
		defer C.string_free(errBuffer)
		return nil, errors.New(C.GoString(errBuffer))
	}
	return &Document{ptr: ptr}, nil
}

func (r *SearchResult) GetSize() (uint64, error) {
	var errBuffer *C.char

	size := C.search_result_get_size(r.ptr, &errBuffer)

	errorMessage := C.GoString(errBuffer)
	defer C.string_free(errBuffer) // Освобождение C строки после использования

	if errorMessage == "" {
		return uint64(size), nil
	} else {
		return uint64(0), fmt.Errorf(errorMessage)
	}
}

func (r *SearchResult) Free() {
	C.search_result_free(r.ptr)
}
