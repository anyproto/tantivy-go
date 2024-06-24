package tantivy

//#include "bindings.h"
import "C"
import (
	"errors"
	"fmt"
	"unsafe"
)

type Document struct{ ptr *C.Document }

func NewDocument() *Document {
	ptr := C.document_create()
	return &Document{ptr: ptr}
}

func (d *Document) AddField(fieldName, fieldValue string, index *Index) error {
	cFieldName := C.CString(fieldName)
	defer C.string_free(cFieldName)
	cFieldValue := C.CString(fieldValue)
	defer C.string_free(cFieldValue)
	var errBuffer *C.char
	C.document_add_field(d.ptr, cFieldName, cFieldValue, index.ptr, &errBuffer)

	errorMessage := C.GoString(errBuffer)
	defer C.string_free(errBuffer)

	if len(errorMessage) == 0 {
		return nil
	} else {
		return fmt.Errorf(errorMessage)
	}
}

func (d *Document) ToJson(schema *Schema, includeFields ...string) (string, error) {
	var errBuffer *C.char

	includeFieldsPtr := make([]*C.char, len(includeFields))
	for i, field := range includeFields {
		includedField := C.CString(field)
		defer C.free(unsafe.Pointer(includedField))
		includeFieldsPtr[i] = includedField
	}

	cStr := C.document_as_json(d.ptr, (**C.char)(unsafe.Pointer(&includeFieldsPtr[0])), C.uintptr_t(len(includeFields)), schema.ptr, &errBuffer)
	if cStr == nil {
		errorMessage := C.GoString(errBuffer)
		defer C.string_free(errBuffer)
		return "", errors.New(errorMessage)
	}
	defer C.string_free(cStr)

	return C.GoString(cStr), nil
}

func ToModel[T any](doc *Document, schema *Schema, includeFields []string, f func(json string) (T, error)) (T, error) {
	json, err := doc.ToJson(schema, includeFields...)
	if err != nil {
		var zero T
		return zero, err
	}
	return f(json)
}

func (d *Document) Free() {
	C.document_free(d.ptr)
}
