package tantivy

// #include "bindings.h"
import "C"
import (
	"errors"
	"fmt"
	"unsafe"
)

type Index struct{ ptr *C.Index }

func NewIndexWithSchema(path string, schema *Schema) (*Index, error) {
	cPath := C.CString(path)
	defer C.string_free(cPath)
	var errBuffer *C.char
	ptr := C.index_create_with_schema(cPath, schema.ptr, &errBuffer)
	if ptr == nil {
		defer C.string_free(errBuffer)
		return nil, errors.New(C.GoString(errBuffer))
	}
	return &Index{ptr: ptr}, nil
}

func (i *Index) AddAndConsumeDocuments(docs ...*Document) error {
	if len(docs) == 0 {
		return nil
	}
	var errBuffer *C.char
	docsPtr := make([]*C.Document, len(docs))
	for j, doc := range docs {
		docsPtr[j] = doc.ptr
	}
	C.index_add_and_consume_documents(i.ptr, &docsPtr[0], C.uintptr_t(len(docs)), &errBuffer)
	return tryExtractError(errBuffer)
}

func (i *Index) DeleteDocuments(field string, deleteIds ...string) error {
	if len(deleteIds) == 0 {
		return nil
	}
	cField := C.CString(field)
	defer C.string_free(cField)

	deleteIDsPtr := make([]*C.char, len(deleteIds))
	for j, id := range deleteIds {
		cID := C.CString(id)
		defer C.free(unsafe.Pointer(cID))
		deleteIDsPtr[j] = cID
	}
	cDeleteIds := (**C.char)(unsafe.Pointer(&deleteIDsPtr[0]))

	var errBuffer *C.char
	C.index_delete_documents(i.ptr, cField, cDeleteIds, C.uintptr_t(len(deleteIds)), &errBuffer)
	return tryExtractError(errBuffer)
}

func (i *Index) NumDocs() (uint64, error) {
	var errBuffer *C.char
	numDocs := C.index_num_docs(i.ptr, &errBuffer)
	if errBuffer != nil {
		defer C.string_free(errBuffer)
		return 0, errors.New(C.GoString(errBuffer))
	}
	return uint64(numDocs), nil
}

func (i *Index) Search(query string, docsLimit uintptr, withHighlights bool, fieldNames ...string) (*SearchResult, error) {
	if len(fieldNames) == 0 {
		return nil, fmt.Errorf("fieldNames must not be empty")
	}
	cQuery := C.CString(query)
	defer C.string_free(cQuery)

	fieldNamesPtr := make([]*C.char, len(fieldNames))
	for j, id := range fieldNames {
		cId := C.CString(id)
		defer C.free(unsafe.Pointer(cId))
		fieldNamesPtr[j] = cId
	}

	var errBuffer *C.char
	ptr := C.index_search(
		i.ptr,
		(**C.char)(unsafe.Pointer(&fieldNamesPtr[0])),
		C.uintptr_t(len(fieldNames)),
		cQuery,
		&errBuffer,
		C.ulong(docsLimit),
		C.bool(withHighlights),
	)
	if ptr == nil {
		defer C.string_free(errBuffer)
		return nil, errors.New(C.GoString(errBuffer))
	}

	return &SearchResult{ptr: ptr}, nil
}

func (i *Index) Free() {
	C.index_free(i.ptr)
}

func (i *Index) RegisterTextAnalyzerNgram(tokenizerName string, minGram, maxGram uintptr, prefixOnly bool) error {
	cTokenizerName := C.CString(tokenizerName)
	defer C.string_free(cTokenizerName)
	var errBuffer *C.char
	C.index_register_text_analyzer_ngram(i.ptr, cTokenizerName, C.uintptr_t(minGram), C.uintptr_t(maxGram), C.bool(prefixOnly), &errBuffer)

	return tryExtractError(errBuffer)
}

func (i *Index) RegisterTextAnalyzerEdgeNgram(tokenizerName string, minGram, maxGram uintptr, limit uintptr) error {
	cTokenizerName := C.CString(tokenizerName)
	defer C.string_free(cTokenizerName)
	var errBuffer *C.char
	C.index_register_text_analyzer_edge_ngram(i.ptr, cTokenizerName, C.uintptr_t(minGram), C.uintptr_t(maxGram), C.uintptr_t(limit), &errBuffer)
	return tryExtractError(errBuffer)
}

func (i *Index) RegisterTextAnalyzerSimple(tokenizerName string, textLimit uintptr, lang string) error {
	cTokenizerName := C.CString(tokenizerName)
	defer C.string_free(cTokenizerName)
	cLang := C.CString(lang)
	defer C.string_free(cLang)
	var errBuffer *C.char
	C.index_register_text_analyzer_simple(i.ptr, cTokenizerName, C.uintptr_t(textLimit), cLang, &errBuffer)

	return tryExtractError(errBuffer)
}

func (i *Index) RegisterTextAnalyzerRaw(tokenizerName string) error {
	cTokenizerName := C.CString(tokenizerName)
	defer C.string_free(cTokenizerName)
	var errBuffer *C.char
	C.index_register_text_analyzer_raw(i.ptr, cTokenizerName, &errBuffer)

	return tryExtractError(errBuffer)
}

func GetSearchResults[T any](
	searchResult *SearchResult,
	schema *Schema,
	f func(json string) (T, error),
	includeFields ...string,
) ([]T, error) {
	var models []T
	defer searchResult.Free()

	size, err := searchResult.GetSize()
	if err != nil {
		fmt.Println("Failed to get search result size:", err)
		return nil, err
	}

	// Iterate through search results
	for next := range size {
		doc, err := searchResult.Get(next)
		if err != nil {
			break
		}
		model, err := ToModel(doc, schema, includeFields, f)
		if err != nil {
			return nil, err
		}
		models = append(models, model)
		doc.Free()
	}
	return models, nil
}
