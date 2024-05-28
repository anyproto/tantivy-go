package tantivy

//#include "bindings.h"
import "C"
import "errors"

type SearchResult struct{ ptr *C.SearchResult }

func (r *SearchResult) GetNext() (*TantivyDocument, error) {
	var errBuffer *C.char
	ptr := C.get_next_result(r.ptr, &errBuffer)
	if ptr == nil {
		defer C.free_string(errBuffer)
		return nil, errors.New(C.GoString(errBuffer))
	}
	return &TantivyDocument{ptr: ptr}, nil
}

func (r *SearchResult) Free() {
	C.free_search_result(r.ptr)
}
