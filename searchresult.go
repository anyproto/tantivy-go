package tantivy_go

//#include "bindings.h"
import "C"
import (
	"errors"
	"fmt"
)

type SearchResult struct{ ptr *C.SearchResult }

// Get retrieves a document from the search result at the specified index.
//
// Parameters:
// - index: The index of the document to retrieve.
//
// Returns:
// - A pointer to the Document if successful, or nil if not found.
// - An error if there was an issue retrieving the document.
func (r *SearchResult) Get(index uint64) (*Document, error) {
	var errBuffer *C.char
	ptr := C.search_result_get_doc(r.ptr, C.uintptr_t(index), &errBuffer)
	if ptr == nil {
		defer C.string_free(errBuffer)
		return nil, errors.New(C.GoString(errBuffer))
	}
	return &Document{ptr: ptr}, nil
}

// GetSize returns the number of documents in the search result.
//
// Returns:
// - The size of the search result if successful.
// - An error if there was an issue getting the size.
func (r *SearchResult) GetSize() (uint64, error) {
	var errBuffer *C.char

	size := C.search_result_get_size(r.ptr, &errBuffer)

	errorMessage := C.GoString(errBuffer)
	defer C.string_free(errBuffer)

	if errorMessage == "" {
		return uint64(size), nil
	} else {
		return uint64(0), fmt.Errorf(errorMessage)
	}
}

func (r *SearchResult) Free() {
	C.search_result_free(r.ptr)
}
