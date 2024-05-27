package main

/*
#cgo LDFLAGS:-L${SRCDIR}/../target/debug -ltantivy_go -lm -pthread -ldl
#include "bindings.h"
#include <stdlib.h>
*/
import "C"
import (
	"fmt"
	"os"
	"sync"
	"unsafe"
)

var doOnce sync.Once

func LibInit(directive ...string) {
	var initVal string
	doOnce.Do(func() {
		if len(directive) == 0 {
			initVal = "info"
		} else {
			initVal = directive[0]
		}
		os.Setenv("ELV_RUST_LOG", initVal)
		C.init()
	})
}

func main() {
	LibInit("debug")
	path := C.CString("index_directory")
	defer C.free(unsafe.Pointer(path))

	title := C.CString("Example Title")
	defer C.free(unsafe.Pointer(title))

	body := C.CString("Example body content.")
	defer C.free(unsafe.Pointer(body))

	query := C.CString("Example")
	defer C.free(unsafe.Pointer(query))

	titleSchema := C.CString("title")
	defer C.free(unsafe.Pointer(titleSchema))

	bodySchema := C.CString("body")
	defer C.free(unsafe.Pointer(bodySchema))

	schema := C.schema_builder_new()

	C.schema_builder_add_text_field(schema, titleSchema, C._Bool(true))
	C.schema_builder_add_text_field(schema, bodySchema, C._Bool(false))

	// Create index
	index := C.create_index_with_schema_builder(path, schema)
	if index == nil {
		fmt.Println("Failed to create index")
		return
	}
	defer C.free_index(index)

	// Add document
	success := C.add_document(index, title, body)
	if success == C._Bool(false) {
		fmt.Println("Failed to add document")
		return
	}

	// Search index
	result := C.search_index(index, query)
	if result != nil {
		fmt.Println("Search results:")
		fmt.Println(C.GoString(result))
		C.free_string(result)
	} else {
		fmt.Println("Failed to search index")
	}
}
