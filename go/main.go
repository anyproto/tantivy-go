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

func getLastError(errorBuffer **C.char) string {
	if *errorBuffer != nil {
		errStr := C.GoString(*errorBuffer)
		C.free_string(*errorBuffer)
		*errorBuffer = nil
		return errStr
	}
	return ""
}

func main() {
	LibInit("debug")
	var errorBuffer *C.char

	path := C.CString("index_directory")
	defer C.free(unsafe.Pointer(path))

	title := C.CString("Example Title")
	defer C.free(unsafe.Pointer(title))

	body := C.CString("Example body content.")
	defer C.free(unsafe.Pointer(body))

	query := C.CString("Example")
	defer C.free(unsafe.Pointer(query))

	// Create schema builder
	builder := C.schema_builder_new(&errorBuffer)
	if builder == nil {
		fmt.Println("Failed to create schema builder:", getLastError(&errorBuffer))
		return
	}

	// Add fields to schema
	titleName := C.CString("title")
	if C.schema_builder_add_text_field(builder, titleName, C._Bool(true), &errorBuffer) != 0 {
		fmt.Println("Failed to add text field:", getLastError(&errorBuffer))
		return
	}
	C.free(unsafe.Pointer(titleName))

	bodyName := C.CString("body")
	if C.schema_builder_add_text_field(builder, bodyName, C._Bool(false), &errorBuffer) != 0 {
		fmt.Println("Failed to add text field:", getLastError(&errorBuffer))
		return
	}
	C.free(unsafe.Pointer(bodyName))

	// Create index with schema builder
	index := C.create_index_with_schema_builder(path, builder, &errorBuffer)
	if index == nil {
		fmt.Println("Failed to create index:", getLastError(&errorBuffer))
		return
	}
	defer C.free_index(index)

	// Create document
	doc := C.create_document()
	if doc == nil {
		fmt.Println("Failed to create document")
		return
	}

	// Add fields to document
	if C.add_field(doc, C.CString("title"), title, index, &errorBuffer) != 0 {
		fmt.Println("Failed to add field to document:", getLastError(&errorBuffer))
		return
	}

	if C.add_field(doc, C.CString("body"), body, index, &errorBuffer) != 0 {
		fmt.Println("Failed to add field to document:", getLastError(&errorBuffer))
		return
	}

	// Add document to index
	if C.add_document(index, doc, &errorBuffer) != 0 {
		fmt.Println("Failed to add document:", getLastError(&errorBuffer))
		return
	}

	// Search index
	result := C.search_index(index, query, &errorBuffer)
	if result == nil {
		fmt.Println("Failed to search index:", getLastError(&errorBuffer))
		return
	}
	defer C.free_search_result(result)

	// Iterate through search results
	for {
		doc := C.get_next_result(result, &errorBuffer)
		if doc == nil {
			break
		}
		// Get JSON representation of the document
		jsonStr := C.get_document_json(doc, &errorBuffer)
		if jsonStr != nil {
			fmt.Println("Document JSON:")
			fmt.Println(C.GoString(jsonStr))
			C.free_string(jsonStr)
		} else {
			fmt.Println("Failed to get document JSON:", getLastError(&errorBuffer))
		}
		C.free_document(doc)
	}
}
