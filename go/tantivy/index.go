package tantivy

/*
#cgo LDFLAGS:-L${SRCDIR}/../../target/debug -ltantivy_go -lm -pthread -ldl
#include "bindings.h"
#include <stdlib.h>
*/
import "C"
import "errors"

type Index struct{ ptr *C.Index }

func NewIndexWithSchema(path string, schema *Schema) (*Index, error) {
	cPath := C.CString(path)
	defer C.free_string(cPath)
	var errBuffer *C.char
	ptr := C.create_index_with_schema(cPath, schema.ptr, &errBuffer)
	if ptr == nil {
		defer C.free_string(errBuffer)
		return nil, errors.New(C.GoString(errBuffer))
	}
	return &Index{ptr: ptr}, nil
}

func (i *Index) AddAndConsumeDocument(doc *TantivyDocument) error {
	var errBuffer *C.char
	res := C.add_document(i.ptr, doc.ptr, &errBuffer)
	if res != 0 {
		defer C.free_string(errBuffer)
		return errors.New(C.GoString(errBuffer))
	}
	return nil
}

func (i *Index) Search(query string) (*SearchResult, error) {
	cQuery := C.CString(query)
	defer C.free_string(cQuery)
	var errBuffer *C.char
	ptr := C.search_index(i.ptr, cQuery, &errBuffer)
	if ptr == nil {
		defer C.free_string(errBuffer)
		return nil, errors.New(C.GoString(errBuffer))
	}
	return &SearchResult{ptr: ptr}, nil
}

func (i *Index) Free() {
	C.free_index(i.ptr)
}
