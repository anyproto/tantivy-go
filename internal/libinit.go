package internal

//#include "../bindings.h"
import "C"
import "fmt"

// LibInit for tests init
func LibInit(cleanOnPanic, utf8Lenient bool, directive ...string) error {
	var initVal string
	if len(directive) == 0 {
		initVal = "info"
	} else {
		initVal = directive[0]
	}

	cInitVal := C.CString(initVal)
	defer C.string_free(cInitVal)
	cCleanOnPanic := C.bool(cleanOnPanic)
	cUtf8Lenient := C.bool(utf8Lenient)
	var errBuffer *C.char
	fmt.Println("### lenient", utf8Lenient)
	C.init_lib(cInitVal, &errBuffer, cCleanOnPanic, cUtf8Lenient)

	errorMessage := C.GoString(errBuffer)
	defer C.string_free(errBuffer)

	if errorMessage != "" {
		return fmt.Errorf(errorMessage)
	}
	return nil
}
