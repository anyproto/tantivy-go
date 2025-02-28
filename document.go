package tantivy_go

//#include "bindings.h"
import "C"
import (
	"errors"
	"unsafe"
)

type Document struct {
	ptr    *C.Document
	toFree []func()
}

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
//   - fieldValue: the value of the field to add
//   - index: the index to use for adding the field
//   - fieldName: the name of the field to add
//
// Returns:
//   - error: an error if adding the field fails, or nil if the operation is successful
func (d *Document) AddField(fieldValue string, tc *TantivyContext, fieldName string) error {
	fieldId, contains := tc.schema.fieldNames[fieldName]
	if !contains {
		return errors.New("field not found in schema")
	}
	cFieldValue := C.CString(fieldValue)
	d.toFree = append(d.toFree, func() { C.string_free(cFieldValue) })
	var errBuffer *C.char
	C.document_add_field(d.ptr, C.uint(fieldId), cFieldValue, &errBuffer)

	return tryExtractError(errBuffer)
}

// AddFields adds a field with the specified name and value to the document using the given index.
// Returns an error if adding the field fails.
//
// Parameters:
//   - fieldValue: the value of the field to add
//   - index: the index to use for adding the field
//   - fieldNames: the names of the fields to add
//
// Returns:
//   - error: an error if adding the field fails, or nil if the operation is successful
func (d *Document) AddFields(fieldValue string, tc *TantivyContext, fieldNames ...string) error {
	includeFieldsPtr, err := tc.extractFields(fieldNames)
	if err != nil {
		return err
	}

	cFieldValue := C.CString(fieldValue)
	d.toFree = append(d.toFree, func() { C.string_free(cFieldValue) })
	var errBuffer *C.char
	C.document_add_fields(d.ptr, (*C.uint)(unsafe.Pointer(&includeFieldsPtr[0])), C.uintptr_t(len(includeFieldsPtr)), cFieldValue, &errBuffer)

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
func (d *Document) ToJson(tc *TantivyContext, includeFields ...string) (string, error) {
	var errBuffer *C.char

	includeFieldsPtr, err := tc.extractFields(includeFields)
	if err != nil {
		return "", err
	}

	cStr := C.document_as_json(
		d.ptr,
		(*C.uint)(unsafe.Pointer(&includeFieldsPtr[0])),
		C.uintptr_t(len(includeFields)),
		tc.schema.ptr,
		&errBuffer,
	)

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
func ToModel[T any](doc *Document, tc *TantivyContext, includeFields []string, f func(json string) (T, error)) (T, error) {
	json, err := doc.ToJson(tc, includeFields...)
	if err != nil {
		var zero T
		return zero, err
	}
	return f(json)
}

func (d *Document) Free() {
	C.document_free(d.ptr)
	d.FreeStrings()
}

func (d *Document) FreeStrings() {
	for _, f := range d.toFree {
		f()
	}
}
