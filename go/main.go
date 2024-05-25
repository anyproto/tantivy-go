package main

/*
#cgo LDFLAGS:-L${SRCDIR}/../target/debug -ltantivy_go -lm -pthread -ldl
#include "bindings.h"
#include <stdlib.h>
*/
import "C"
import (
	"fmt"
	"unsafe"
)

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

// DeleteExample освобождает память, выделенную под Example
func (e *Example) DeleteExample() {
	C.delete_example(e.ptr)
}

func main() {
	example := CreateExample("John")
	fmt.Println(example.GetName())
	example.SetName("Doe")
	fmt.Println(example.GetName())
	example.DeleteExample()
	fmt.Println("Example created, name set, and memory freed successfully")
}
