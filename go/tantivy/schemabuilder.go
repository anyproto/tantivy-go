package tantivy

//#include "bindings.h"
import "C"
import (
	"errors"
	"fmt"
)

type (
	SchemaBuilder struct{ ptr *C.SchemaBuilder }
	Schema        struct{ ptr *C.Schema }
)

const (
	IndexRecordOptionBasic = iota
	IndexRecordOptionWithFreqs
	IndexRecordOptionWithFreqsAndPositions
)

const DefaultTokenizer = "default"

const (
	Arabic     = "ar"
	Danish     = "da"
	Dutch      = "nl"
	English    = "en"
	Finnish    = "fi"
	French     = "fr"
	German     = "de"
	Greek      = "el"
	Hungarian  = "hu"
	Italian    = "it"
	Norwegian  = "no"
	Portuguese = "pt"
	Romanian   = "ro"
	Russian    = "ru"
	Spanish    = "es"
	Swedish    = "sv"
	Tamil      = "ta"
	Turkish    = "tr"
)

func NewSchemaBuilder() (*SchemaBuilder, error) {
	ptr := C.schema_builder_new()
	if ptr == nil {
		return nil, errors.New("failed to create schema builder")
	}
	return &SchemaBuilder{ptr: ptr}, nil
}

func (b *SchemaBuilder) AddTextField(
	name string,
	stored bool,
	isText bool,
	indexRecordOption int,
	tokenizer string,
) error {
	cName := C.CString(name)
	cTokenizer := C.CString(tokenizer)
	defer C.string_free(cName)
	defer C.string_free(cTokenizer)
	var errBuffer *C.char
	C.schema_builder_add_text_field(
		b.ptr,
		cName,
		C._Bool(stored),
		C._Bool(isText),
		C.ulong(indexRecordOption),
		cTokenizer,
		&errBuffer,
	)
	errorMessage := C.GoString(errBuffer)
	defer C.string_free(errBuffer)

	if len(errorMessage) == 0 {
		return nil
	} else {
		return fmt.Errorf(errorMessage)
	}
}

func (b *SchemaBuilder) BuildSchema() (*Schema, error) {
	var errBuffer *C.char
	ptr := C.schema_builder_build(b.ptr, &errBuffer)
	if ptr == nil {
		defer C.string_free(errBuffer)
		return nil, errors.New(C.GoString(errBuffer))
	}
	return &Schema{ptr: ptr}, nil
}
