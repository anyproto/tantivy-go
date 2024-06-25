package tantivy

// #include "bindings.h"
import "C"
import "fmt"

func tryExtractError(errBuffer *C.char) error {
	errorMessage := C.GoString(errBuffer)
	defer C.string_free(errBuffer)

	if len(errorMessage) == 0 {
		return nil
	} else {
		return fmt.Errorf(errorMessage)
	}
}