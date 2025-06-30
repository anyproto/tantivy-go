package tantivy_go

import (
	"sync"

	"github.com/anyproto/tantivy-go/internal"
)

// Activate after migration to the go 1.24
/*
#cgo nocallback schema_builder_build
#cgo nocallback context_create_with_schema
#cgo nocallback context_register_text_analyzer_ngram
#cgo nocallback context_register_text_analyzer_edge_ngram
#cgo nocallback context_register_text_analyzer_simple
#cgo nocallback context_register_jieba_tokenizer
#cgo nocallback context_register_text_analyzer_raw
#cgo nocallback context_add_and_consume_documents
#cgo nocallback context_delete_documents
#cgo nocallback context_num_docs
#cgo nocallback context_search
#cgo nocallback context_search_json
#cgo nocallback context_free
#cgo nocallback search_result_get_size
#cgo nocallback search_result_get_doc
#cgo nocallback search_result_free
#cgo nocallback document_create
#cgo nocallback document_add_field
#cgo nocallback document_add_fields
#cgo nocallback document_as_json
#cgo nocallback document_free
#cgo nocallback string_free
#cgo noescape schema_builder_build
#cgo noescape context_create_with_schema
#cgo noescape context_register_text_analyzer_ngram
#cgo noescape context_register_text_analyzer_edge_ngram
#cgo noescape context_register_text_analyzer_simple
#cgo noescape context_register_jieba_tokenizer
#cgo noescape context_register_text_analyzer_raw
#cgo noescape context_add_and_consume_documents
#cgo noescape context_delete_documents
#cgo noescape context_num_docs
#cgo noescape context_search
#cgo noescape context_search_json
#cgo noescape context_free
#cgo noescape search_result_get_size
#cgo noescape search_result_get_doc
#cgo noescape search_result_free
#cgo noescape document_create
#cgo noescape document_add_field
#cgo noescape document_add_fields
#cgo noescape document_as_json
#cgo noescape document_free
#cgo noescape string_free
*/

const TokenizerSimple = "simple_tokenizer"
const TokenizerNgram = "ngram"
const TokenizerJieba = "jieba"
const TokenizerEdgeNgram = "edge_ngram"
const TokenizerRaw = "raw"

var doOnce sync.Once

// LibInit initializes the library with an optional directive.
//
// Parameters:
//   - directive: A variadic parameter that allows specifying an initialization directive.
//     If no directive is provided, the default value "info" is used.
//
// Returns:
// - An error if the initialization fails.
func LibInit(cleanOnPanic, utf8Lenient bool, directive ...string) error {
	var err error
	doOnce.Do(func() {
		err = internal.LibInit(cleanOnPanic, utf8Lenient, directive...)
	})
	return err
}
