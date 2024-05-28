package tantivy

/*
#cgo LDFLAGS:-L${SRCDIR}/../../target/debug -ltantivy_go -lm -pthread -ldl
#include "bindings.h"
#include <stdlib.h>
*/
import "C"
import "errors"

type TantivyDocument struct{ ptr *C.TantivyDocument }

func NewDocument() *TantivyDocument {
	ptr := C.create_document()
	return &TantivyDocument{ptr: ptr}
}

func (d *TantivyDocument) AddField(fieldName, fieldValue string, index *Index) error {
	cFieldName := C.CString(fieldName)
	defer C.free_string(cFieldName)
	cFieldValue := C.CString(fieldValue)
	defer C.free_string(cFieldValue)
	var errBuffer *C.char
	res := C.add_field(d.ptr, cFieldName, cFieldValue, index.ptr, &errBuffer)
	if res != 0 {
		defer C.free_string(errBuffer)
		return errors.New(C.GoString(errBuffer))
	}
	return nil
}

func (d *TantivyDocument) ToJSON(schema *Schema) (string, error) {
	var errBuffer *C.char
	cStr := C.get_document_json(d.ptr, schema.ptr, &errBuffer)
	if cStr == nil {
		defer C.free_string(errBuffer)
		return "", errors.New(C.GoString(errBuffer))
	}
	defer C.free_string(cStr)
	return C.GoString(cStr), nil
}

func (d *TantivyDocument) Free() {
	C.free_document(d.ptr)
}
