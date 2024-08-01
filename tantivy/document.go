package tantivy

//#include "bindings.h"
import "C"
import (
	"errors"
	"unsafe"
)

type Document struct{ ptr *C.Document }

// NewDocument creates a new instance of Document.
//
// Returns:
//   - *Document: a pointer to a newly created Document instance.
func NewDocument() *Document {
	ptr := C.document_create()
	return &Document{ptr: ptr}
}

// AddField adds a field with the specified name and value to the document using the given index.
// Returns an error if adding the field fails.
//
// Parameters:
//   - fieldName: the name of the field to add
//   - fieldValue: the value of the field to add
//   - index: the index to use for adding the field
//
// Returns:
//   - error: an error if adding the field fails, or nil if the operation is successful
func (d *Document) AddField(fieldName, fieldValue string, index *Index) error {
	cFieldName := C.CString(fieldName)
	defer C.string_free(cFieldName)
	cFieldValue := C.CString(fieldValue)
	defer C.string_free(cFieldValue)
	var errBuffer *C.char
	C.document_add_field(d.ptr, cFieldName, cFieldValue, index.ptr, &errBuffer)

	return tryExtractError(errBuffer)
}

// ToJson converts the document to its JSON representation based on the provided schema.
// Optionally, specific fields can be included in the JSON output.
//
// Parameters:
//   - schema: the schema to use for converting the document to JSON
//   - includeFields: optional variadic parameter specifying the fields to include in the JSON output
//
// Returns:
//   - string: the JSON representation of the document
//   - error: an error if the conversion fails, or nil if the operation is successful
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

// ToModel converts a document to a model of type T using the provided schema and a conversion function.
//
// Parameters:
//   - doc: the document to convert
//   - schema: the schema to use for converting the document to JSON
//   - includeFields: optional fields to include in the JSON output
//   - f: a function that takes a JSON string and converts it to a model of type T
//
// Returns:
//   - T: the model of type T resulting from the conversion
//   - error: an error if the conversion fails, or nil if the operation is successful
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
