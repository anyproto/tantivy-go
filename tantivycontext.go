package tantivy_go

// #include "bindings.h"
import "C"
import (
	"errors"
	"fmt"
	"sync"
	"unsafe"
)

type TantivyContext struct {
	ptr    *C.TantivyContext
	schema *Schema
	lock   sync.Mutex // tantivy writer commits should be executed exclusively
}

// NewTantivyContextWithSchema creates a new instance of TantivyContext with the provided schema.
//
// Parameters:
//   - path: The path to the index as a string.
//   - schema: A pointer to the Schema to be used.
//
// Returns:
//   - *TantivyContext: A pointer to a newly created TantivyContext instance.
//   - error: An error if the index creation fails.
func NewTantivyContextWithSchema(path string, schema *Schema) (*TantivyContext, error) {
	cPath := C.CString(path)
	defer C.string_free(cPath)
	var errBuffer *C.char
	ptr := C.context_create_with_schema(cPath, schema.ptr, &errBuffer)
	if ptr == nil {
		defer C.string_free(errBuffer)
		return nil, errors.New(C.GoString(errBuffer))
	}
	return &TantivyContext{
		ptr:    ptr,
		schema: schema,
	}, nil
}

// AddAndConsumeDocuments adds and consumes the provided documents to the index.
//
// Parameters:
//   - docs: A variadic parameter of pointers to Document to be added and consumed.
//
// Returns:
//   - error: An error if adding and consuming the documents fails.
func (tc *TantivyContext) AddAndConsumeDocuments(docs ...*Document) error {
	tc.lock.Lock()
	defer tc.lock.Unlock()
	if len(docs) == 0 {
		return nil
	}
	var errBuffer *C.char
	docsPtr := make([]*C.Document, len(docs))
	for j, doc := range docs {
		docsPtr[j] = doc.ptr
	}
	C.context_add_and_consume_documents(tc.ptr, &docsPtr[0], C.uintptr_t(len(docs)), &errBuffer)
	for _, doc := range docs {
		// Free the strings in the document
		// This is necessary because the document is consumed by the index
		// and the strings are not freed by the index
		// We might clone strings on the Rust side to avoid that, but that would be inefficient
		doc.FreeStrings()
	}
	return tryExtractError(errBuffer)
}

// DeleteDocuments deletes documents from the index based on the specified field and IDs.
//
// Parameters:
//   - fieldName: The field name to match against the document IDs.
//   - deleteIds: A variadic parameter of document IDs to be deleted.
//
// Returns:
//   - error: An error if deleting the documents fails.
func (tc *TantivyContext) DeleteDocuments(fieldName string, deleteIds ...string) error {
	tc.lock.Lock()
	defer tc.lock.Unlock()
	if len(deleteIds) == 0 {
		return nil
	}
	fieldId, contains := tc.schema.fieldNames[fieldName]
	if !contains {
		return errors.New("field not found in schema")
	}

	deleteIDsPtr := make([]*C.char, len(deleteIds))
	for j, id := range deleteIds {
		cID := C.CString(id)
		defer C.free(unsafe.Pointer(cID))
		deleteIDsPtr[j] = cID
	}
	cDeleteIds := (**C.char)(unsafe.Pointer(&deleteIDsPtr[0]))

	var errBuffer *C.char
	C.context_delete_documents(tc.ptr, C.uint(fieldId), cDeleteIds, C.uintptr_t(len(deleteIds)), &errBuffer)
	return tryExtractError(errBuffer)
}

// NumDocs returns the number of documents in the index.
//
// Returns:
//   - uint64: The number of documents.
//   - error: An error if retrieving the document count fails.
func (tc *TantivyContext) NumDocs() (uint64, error) {
	var errBuffer *C.char
	numDocs := C.context_num_docs(tc.ptr, &errBuffer)
	if errBuffer != nil {
		defer C.string_free(errBuffer)
		return 0, errors.New(C.GoString(errBuffer))
	}
	return uint64(numDocs), nil
}

// Search performs a search query on the index and returns the search results.
//
// Parameters:
//   - sCtx (SearchContext): The context for the search, containing query string,
//     document limit, highlight option, and field weights.
//
// Returns:
//   - *SearchResult: A pointer to the SearchResult containing the search results.
//   - error: An error if the search fails.
func (tc *TantivyContext) Search(sCtx SearchContext) (*SearchResult, error) {
	fieldNames, weights := sCtx.GetFieldAndWeights()
	if len(fieldNames) == 0 {
		return nil, fmt.Errorf("fieldNames must not be empty")
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

	var errBuffer *C.char
	ptr := C.context_search(
		tc.ptr,
		(*C.uint)(unsafe.Pointer(&fieldNamesPtr[0])),
		(*C.float)(unsafe.Pointer(&fieldWeightsPtr[0])),
		C.uintptr_t(len(fieldNames)),
		cQuery,
		&errBuffer,
		pointerCType(sCtx.GetDocsLimit()),
		C.bool(sCtx.WithHighlights()),
	)
	if ptr == nil {
		defer C.string_free(errBuffer)
		return nil, errors.New(C.GoString(errBuffer))
	}

	return &SearchResult{ptr: ptr}, nil
}

// SearchJson performs a simplified search query on the index and returns the search results.
//
// Parameters:
//   - sCtx (SearchContext): The context for the search, containing query string,
//     document limit, and highlight option.
//
// Returns:
//   - *SearchResult: A pointer to the SearchResult containing the search results.
//   - error: An error if the search fails.
func (tc *TantivyContext) SearchJson(sCtx SearchContext) (*SearchResult, error) {
	// Ensure the query is valid
	cQuery := C.CString(sCtx.GetQuery())
	defer C.string_free(cQuery)

	// Prepare the error buffer
	var errBuffer *C.char

	// Call the C function
	ptr := C.context_search_json(
		tc.ptr,
		cQuery,
		&errBuffer,
		pointerCType(sCtx.GetDocsLimit()),
		C.bool(sCtx.WithHighlights()),
	)
	if ptr == nil {
		defer C.string_free(errBuffer)
		return nil, errors.New(C.GoString(errBuffer))
	}

	return &SearchResult{ptr: ptr}, nil
}

// Close waits till the merging operations are finished and releases all the resources held by the indexWriter
func (tc *TantivyContext) Close() error {
	ptr := tc.ptr
	var errBuffer *C.char
	C.context_wait_and_free(ptr, &errBuffer)
	return tryExtractError(errBuffer)
}

// Deprecated: Use Close() instead.
func (tc *TantivyContext) Free() {
	if err := tc.Close(); err != nil {
		fmt.Println("Failed to wait for merging threads: ", err)
	}
}

// RegisterTextAnalyzerNgram registers a text analyzer using N-grams with the index.
//
// Parameters:
//   - tokenizerName (string): The name of the tokenizer to be used.
//   - minGram (uintptr): The minimum length of the n-grams.
//   - maxGram (uintptr): The maximum length of the n-grams.
//   - prefixOnly (bool): Whether to generate only prefix n-grams.
//
// Returns:
//   - error: An error if the registration fails.
func (tc *TantivyContext) RegisterTextAnalyzerNgram(tokenizerName string, minGram, maxGram uintptr, prefixOnly bool) error {
	cTokenizerName := C.CString(tokenizerName)
	defer C.string_free(cTokenizerName)
	var errBuffer *C.char
	C.context_register_text_analyzer_ngram(tc.ptr, cTokenizerName, C.uintptr_t(minGram), C.uintptr_t(maxGram), C.bool(prefixOnly), &errBuffer)

	return tryExtractError(errBuffer)
}

// RegisterTextAnalyzerEdgeNgram registers a text analyzer using edge n-grams with the index.
//
// Parameters:
//   - tokenizerName (string): The name of the tokenizer to be used.
//   - minGram (uintptr): The minimum length of the edge n-grams.
//   - maxGram (uintptr): The maximum length of the edge n-grams.
//   - limit (uintptr): The maximum number of edge n-grams to generate.
//
// Returns:
//   - error: An error if the registration fails.
func (tc *TantivyContext) RegisterTextAnalyzerEdgeNgram(tokenizerName string, minGram, maxGram uintptr, limit uintptr) error {
	cTokenizerName := C.CString(tokenizerName)
	defer C.string_free(cTokenizerName)
	var errBuffer *C.char
	C.context_register_text_analyzer_edge_ngram(tc.ptr, cTokenizerName, C.uintptr_t(minGram), C.uintptr_t(maxGram), C.uintptr_t(limit), &errBuffer)
	return tryExtractError(errBuffer)
}

// RegisterTextAnalyzerSimple registers a simple text analyzer with the index.
//
// Parameters:
//   - tokenizerName (string): The name of the tokenizer to be used.
//   - textLimit (uintptr): The limit on the length of the text to be analyzed.
//   - lang (string): The language code for the text analyzer.
//
// Returns:
//   - error: An error if the registration fails.
func (tc *TantivyContext) RegisterTextAnalyzerSimple(tokenizerName string, textLimit uintptr, lang Language) error {
	cTokenizerName := C.CString(tokenizerName)
	defer C.string_free(cTokenizerName)
	cLang := C.CString(string(lang))
	defer C.string_free(cLang)
	var errBuffer *C.char
	C.context_register_text_analyzer_simple(tc.ptr, cTokenizerName, C.uintptr_t(textLimit), cLang, &errBuffer)

	return tryExtractError(errBuffer)
}

// RegisterTextAnalyzerJieba registers a jieba text analyzer with the index.
//
// Parameters:
//   - tokenizerName (string): The name of the tokenizer to be used.
//   - textLimit (uintptr): The limit on the length of the text to be analyzed.
//
// Returns:
//   - error: An error if the registration fails.
func (tc *TantivyContext) RegisterTextAnalyzerJieba(tokenizerName string, textLimit uintptr) error {
	cTokenizerName := C.CString(tokenizerName)
	defer C.string_free(cTokenizerName)
	var errBuffer *C.char
	C.context_register_jieba_tokenizer(tc.ptr, cTokenizerName, C.uintptr_t(textLimit), &errBuffer)

	return tryExtractError(errBuffer)
}

// RegisterTextAnalyzerRaw registers a raw text analyzer with the index.
//
// Parameters:
//   - tokenizerName (string): The name of the raw tokenizer to be used.
//
// Returns:
//   - error: An error if the registration fails.
func (tc *TantivyContext) RegisterTextAnalyzerRaw(tokenizerName string) error {
	cTokenizerName := C.CString(tokenizerName)
	defer C.string_free(cTokenizerName)
	var errBuffer *C.char
	C.context_register_text_analyzer_raw(tc.ptr, cTokenizerName, &errBuffer)

	return tryExtractError(errBuffer)
}

// GetSearchResults extracts search results from a SearchResult and converts them into a slice of models.
//
// Parameters:
//   - searchResult (*SearchResult): The search results to process.
//   - schema (*Schema): The schema to use for converting documents to models.
//   - f (func(json string) (T, error)): A function to convert JSON strings to models.
//   - includeFields (...string): Optional list of fields to include in the result.
//
// Returns:
//   - ([]T, error): A slice of models obtained from the search results, and an error if something goes wrong.
func GetSearchResults[T any](
	searchResult *SearchResult,
	tc *TantivyContext,
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
		model, err := ToModel(doc, tc, includeFields, f)
		if err != nil {
			return nil, err
		}
		models = append(models, model)
		doc.Free()
	}
	return models, nil
}

func (tc *TantivyContext) extractFields(fieldNames []string) ([]C.uint, error) {
	if len(fieldNames) == 0 {
		return nil, errors.New("field names is empty")
	}

	includeFieldsPtr := make([]C.uint, len(fieldNames))
	for i, fieldName := range fieldNames {
		fieldId, contains := tc.schema.fieldNames[fieldName]
		if !contains {
			return nil, errors.New("field not found in schema")
		}
		includeFieldsPtr[i] = C.uint(fieldId)
	}
	return includeFieldsPtr, nil
}
