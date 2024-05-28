package tantivy

/*
#cgo LDFLAGS:-L${SRCDIR}/../../target/debug -ltantivy_go -lm -pthread -ldl
#include "bindings.h"
#include <stdlib.h>
*/
import "C"
import "errors"

type (
	SchemaBuilder struct{ ptr *C.SchemaBuilder }
	Schema        struct{ ptr *C.Schema }
)

func NewSchemaBuilder() (*SchemaBuilder, error) {
	var errBuffer *C.char
	ptr := C.schema_builder_new(&errBuffer)
	if ptr == nil {
		defer C.free_string(errBuffer)
		return nil, errors.New(C.GoString(errBuffer))
	}
	return &SchemaBuilder{ptr: ptr}, nil
}

func (b *SchemaBuilder) AddTextField(name string, stored bool) error {
	cName := C.CString(name)
	defer C.free_string(cName)
	var errBuffer *C.char
	res := C.schema_builder_add_text_field(b.ptr, cName, C._Bool(stored), &errBuffer)
	if res != 0 {
		defer C.free_string(errBuffer)
		return errors.New(C.GoString(errBuffer))
	}
	return nil
}

func (b *SchemaBuilder) BuildSchema() (*Schema, error) {
	var errBuffer *C.char
	ptr := C.build_schema(b.ptr, &errBuffer)
	if ptr == nil {
		defer C.free_string(errBuffer)
		return nil, errors.New(C.GoString(errBuffer))
	}
	return &Schema{ptr: ptr}, nil
}

func (s *Schema) Free() {
	C.free_schema(s.ptr)
}
