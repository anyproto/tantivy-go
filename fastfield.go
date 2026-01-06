package tantivy_go

// #include "bindings.h"
import "C"
import (
	"errors"
	"fmt"
	"unsafe"
)

// FastFieldResult holds the results of a fast field search.
type FastFieldResult struct {
	Values []string
	Scores []float32
}

// SearchFastField performs a search returning only fast field values without loading full documents.
// The field must be configured with isFast=true in the schema.
func (tc *TantivyContext) SearchFastField(sCtx SearchContext, fastFieldName string) (*FastFieldResult, error) {
	fieldNames, weights := sCtx.GetFieldAndWeights()
	if len(fieldNames) == 0 {
		return nil, fmt.Errorf("fieldNames must not be empty")
	}

	docsLimit := sCtx.GetDocsLimit()
	if docsLimit == 0 {
		return nil, errors.New("docsLimit must be greater than 0")
	}

	fastFieldId, contains := tc.schema.fieldNames[fastFieldName]
	if !contains {
		return nil, errors.New("fast field not found in schema")
	}

	cQuery := C.CString(sCtx.GetQuery())
	defer C.string_free(cQuery)

	fieldNamesPtr, err := tc.extractFields(fieldNames)
	if err != nil {
		return nil, err
	}

	fieldWeightsPtr := make([]C.float, len(fieldNames))
	for j, weight := range weights {
		fieldWeightsPtr[j] = C.float(weight)
	}

	outScores := make([]C.float, docsLimit)
	outValues := make([]*C.char, docsLimit)

	var errBuffer *C.char
	count := C.context_search_fast_field(
		tc.ptr,
		(*C.uint)(unsafe.Pointer(&fieldNamesPtr[0])),
		(*C.float)(unsafe.Pointer(&fieldWeightsPtr[0])),
		C.uintptr_t(len(fieldNames)),
		cQuery,
		C.uint(fastFieldId),
		pointerCType(docsLimit),
		(*C.float)(unsafe.Pointer(&outScores[0])),
		(**C.char)(unsafe.Pointer(&outValues[0])),
		&errBuffer,
	)

	if count == 0 {
		if errBuffer != nil {
			errMsg := C.GoString(errBuffer)
			C.string_free(errBuffer)
			if errMsg != "" {
				return nil, errors.New(errMsg)
			}
		}
		return &FastFieldResult{
			Values: []string{},
			Scores: []float32{},
		}, nil
	}

	result := &FastFieldResult{
		Values: make([]string, count),
		Scores: make([]float32, count),
	}

	for i := 0; i < int(count); i++ {
		result.Scores[i] = float32(outScores[i])
		if outValues[i] != nil {
			result.Values[i] = C.GoString(outValues[i])
		}
	}

	C.fast_field_values_free(
		(**C.char)(unsafe.Pointer(&outValues[0])),
		C.uintptr_t(count),
	)

	return result, nil
}

// SearchFastFieldJson performs a search using JSON query returning only fast field values.
// The field must be configured with isFast=true in the schema.
// Use this with AllQuery or other JSON-based queries.
func (tc *TantivyContext) SearchFastFieldJson(sCtx SearchContext, fastFieldName string) (*FastFieldResult, error) {
	docsLimit := sCtx.GetDocsLimit()
	if docsLimit == 0 {
		return nil, errors.New("docsLimit must be greater than 0")
	}

	fastFieldId, contains := tc.schema.fieldNames[fastFieldName]
	if !contains {
		return nil, errors.New("fast field not found in schema")
	}

	cQuery := C.CString(sCtx.GetQuery())
	defer C.string_free(cQuery)

	outScores := make([]C.float, docsLimit)
	outValues := make([]*C.char, docsLimit)

	var errBuffer *C.char
	count := C.context_search_fast_field_json(
		tc.ptr,
		cQuery,
		C.uint(fastFieldId),
		pointerCType(docsLimit),
		(*C.float)(unsafe.Pointer(&outScores[0])),
		(**C.char)(unsafe.Pointer(&outValues[0])),
		&errBuffer,
	)

	if count == 0 {
		if errBuffer != nil {
			errMsg := C.GoString(errBuffer)
			C.string_free(errBuffer)
			if errMsg != "" {
				return nil, errors.New(errMsg)
			}
		}
		return &FastFieldResult{
			Values: []string{},
			Scores: []float32{},
		}, nil
	}

	result := &FastFieldResult{
		Values: make([]string, count),
		Scores: make([]float32, count),
	}

	for i := 0; i < int(count); i++ {
		result.Scores[i] = float32(outScores[i])
		if outValues[i] != nil {
			result.Values[i] = C.GoString(outValues[i])
		}
	}

	C.fast_field_values_free(
		(**C.char)(unsafe.Pointer(&outValues[0])),
		C.uintptr_t(count),
	)

	return result, nil
}
