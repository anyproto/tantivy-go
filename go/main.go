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

// Example обертка над структурой Example из Си
type Example struct {
	ptr *C.Example
}

// CreateExample создает новый Example
func CreateExample(name string) *Example {
	cName := C.CString(name)
	defer C.free(unsafe.Pointer(cName))

	ptr := C.create_example(cName)
	return &Example{ptr}
}

// SetName устанавливает имя для Example
func (e *Example) SetName(name string) {
	cName := C.CString(name)
	defer C.free(unsafe.Pointer(cName))

	C.example_set_name(e.ptr, cName)
}

func (e *Example) GetName() string {
	// Вызываем функцию example_get_name из C
	namePtr := C.example_get_name((*C.struct_Example)(e.ptr))
	defer C.free(unsafe.Pointer(namePtr)) // Освобождаем память после использования

	// Преобразуем указатель на char в строку Go
	return C.GoString(namePtr)
}

func (e *Example) GetArr() []string {
	var strings []string

	// Call the C function to get the array of C strings.
	cStringArray := C.example_get_arr(e.ptr)

	// Iterate over the array of C strings until we find a NULL pointer.
	for {
		// Dereference the pointer to get a pointer to the current C string.
		cStr := *(**C.char)(unsafe.Pointer(cStringArray))

		// Check if we've reached the NULL pointer indicating the end of the array.
		if cStr == nil {
			break
		}

		// Convert the C string to a Go string and append it to the slice.
		strings = append(strings, C.GoString(cStr))

		// Move to the next C string in the array.
		cStringArray = (**C.char)(unsafe.Pointer(uintptr(unsafe.Pointer(cStringArray)) + unsafe.Sizeof(cStringArray)))
	}

	return strings
}

// DeleteExample освобождает память, выделенную под Example
func (e *Example) DeleteExample() {
	C.delete_example(e.ptr)
}

func main() {
	LibInit("debug")
	example := CreateExample("John")
	fmt.Println(example.GetName())
	example.SetName("Doe")
	fmt.Println(example.GetName())
	fmt.Println(example.GetArr())
	example.DeleteExample()
	fmt.Println("Example created, name set, and memory freed successfully")
}
