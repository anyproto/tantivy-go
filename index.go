package tantivy_go

// #include "bindings.h"
import "C"
import (
	"errors"
	"fmt"
	"unsafe"
)

type Index struct{ ptr *C.Index }

// NewIndexWithSchema creates a new instance of Index with the provided schema.
//
// Parameters:
//   - path: The path to the index as a string.
//   - schema: A pointer to the Schema to be used.
//
// Returns:
//   - *Index: A pointer to a newly created Index instance.
//   - error: An error if the index creation fails.
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

// AddAndConsumeDocuments adds and consumes the provided documents to the index.
//
// Parameters:
//   - docs: A variadic parameter of pointers to Document to be added and consumed.
//
// Returns:
//   - error: An error if adding and consuming the documents fails.
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

// DeleteDocuments deletes documents from the index based on the specified field and IDs.
//
// Parameters:
//   - field: The field name to match against the document IDs.
//   - deleteIds: A variadic parameter of document IDs to be deleted.
//
// Returns:
//   - error: An error if deleting the documents fails.
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

// NumDocs returns the number of documents in the index.
//
// Returns:
//   - uint64: The number of documents.
//   - error: An error if retrieving the document count fails.
func (i *Index) NumDocs() (uint64, error) {
	var errBuffer *C.char
	numDocs := C.index_num_docs(i.ptr, &errBuffer)
	if errBuffer != nil {
		defer C.string_free(errBuffer)
		return 0, errors.New(C.GoString(errBuffer))
	}
	return uint64(numDocs), nil
}

// Search performs a search query on the index and returns the search results.
//
// Parameters:
//   - query (string): The search query string.
//   - docsLimit (uintptr): The maximum number of documents to return.
//   - withHighlights (bool): Whether to include highlights in the results.
//   - fieldNames (...string): The names of the fields to be included in the search.
//
// Returns:
//   - *SearchResult: A pointer to the SearchResult containing the search results.
//   - error: An error if the search fails.
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
		pointerCType(docsLimit),
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
func (i *Index) RegisterTextAnalyzerNgram(tokenizerName string, minGram, maxGram uintptr, prefixOnly bool) error {
	cTokenizerName := C.CString(tokenizerName)
	defer C.string_free(cTokenizerName)
	var errBuffer *C.char
	C.index_register_text_analyzer_ngram(i.ptr, cTokenizerName, C.uintptr_t(minGram), C.uintptr_t(maxGram), C.bool(prefixOnly), &errBuffer)

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
func (i *Index) RegisterTextAnalyzerEdgeNgram(tokenizerName string, minGram, maxGram uintptr, limit uintptr) error {
	cTokenizerName := C.CString(tokenizerName)
	defer C.string_free(cTokenizerName)
	var errBuffer *C.char
	C.index_register_text_analyzer_edge_ngram(i.ptr, cTokenizerName, C.uintptr_t(minGram), C.uintptr_t(maxGram), C.uintptr_t(limit), &errBuffer)
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
func (i *Index) RegisterTextAnalyzerSimple(tokenizerName string, textLimit uintptr, lang string) error {
	cTokenizerName := C.CString(tokenizerName)
	defer C.string_free(cTokenizerName)
	cLang := C.CString(lang)
	defer C.string_free(cLang)
	var errBuffer *C.char
	C.index_register_text_analyzer_simple(i.ptr, cTokenizerName, C.uintptr_t(textLimit), cLang, &errBuffer)

	return tryExtractError(errBuffer)
}

// RegisterTextAnalyzerRaw registers a raw text analyzer with the index.
//
// Parameters:
//   - tokenizerName (string): The name of the raw tokenizer to be used.
//
// Returns:
//   - error: An error if the registration fails.
func (i *Index) RegisterTextAnalyzerRaw(tokenizerName string) error {
	cTokenizerName := C.CString(tokenizerName)
	defer C.string_free(cTokenizerName)
	var errBuffer *C.char
	C.index_register_text_analyzer_raw(i.ptr, cTokenizerName, &errBuffer)

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
