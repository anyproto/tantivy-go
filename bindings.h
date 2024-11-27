#include "binding_typedefs.h"
#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#define DOCUMENT_BUDGET_BYTES 50000000

typedef struct Document Document;

typedef struct SearchResult SearchResult;

typedef struct TantivyContext TantivyContext;

SchemaBuilder *schema_builder_new(void);

void schema_builder_add_text_field(SchemaBuilder *builder_ptr,
                                   const char *field_name_ptr,
                                   bool stored,
                                   bool is_text,
                                   bool is_fast,
                                   uintptr_t index_record_option_const,
                                   const char *tokenizer_name_ptr,
                                   char **error_buffer);

Schema *schema_builder_build(SchemaBuilder *builder_ptr, char **error_buffer);

struct TantivyContext *context_create_with_schema(const char *path_ptr,
                                                  Schema *schema_ptr,
                                                  char **error_buffer);

void context_register_text_analyzer_ngram(struct TantivyContext *context_ptr,
                                          const char *tokenizer_name_ptr,
                                          uintptr_t min_gram,
                                          uintptr_t max_gram,
                                          bool prefix_only,
                                          char **error_buffer);

void context_register_text_analyzer_edge_ngram(struct TantivyContext *context_ptr,
                                               const char *tokenizer_name_ptr,
                                               uintptr_t min_gram,
                                               uintptr_t max_gram,
                                               uintptr_t limit,
                                               char **error_buffer);

void context_register_text_analyzer_simple(struct TantivyContext *context_ptr,
                                           const char *tokenizer_name_ptr,
                                           uintptr_t text_limit,
                                           const char *lang_str_ptr,
                                           char **error_buffer);

void context_register_jieba_tokenizer(struct TantivyContext *context_ptr,
                                      const char *tokenizer_name_ptr,
                                      uintptr_t text_limit,
                                      char **error_buffer);

void context_register_text_analyzer_raw(struct TantivyContext *context_ptr,
                                        const char *tokenizer_name_ptr,
                                        char **error_buffer);

void context_add_and_consume_documents(struct TantivyContext *context_ptr,
                                       struct Document **docs_ptr,
                                       uintptr_t docs_len,
                                       char **error_buffer);

void context_delete_documents(struct TantivyContext *context_ptr,
                              const char *field_name_ptr,
                              const char **delete_ids_ptr,
                              uintptr_t delete_ids_len,
                              char **error_buffer);

uint64_t context_num_docs(struct TantivyContext *context_ptr, char **error_buffer);

struct SearchResult *context_search(struct TantivyContext *context_ptr,
                                    const char **field_names_ptr,
                                    float *field_weights_ptr,
                                    uintptr_t field_names_len,
                                    const char *query_ptr,
                                    char **error_buffer,
                                    uintptr_t docs_limit,
                                    bool with_highlights);

struct SearchResult *context_search_json(struct TantivyContext *context_ptr,
                                         const char *query_ptr,
                                         char **error_buffer,
                                         uintptr_t docs_limit,
                                         bool with_highlights);

void context_free(struct TantivyContext *context_ptr);

uintptr_t search_result_get_size(struct SearchResult *result_ptr, char **error_buffer);

struct Document *search_result_get_doc(struct SearchResult *result_ptr,
                                       uintptr_t index,
                                       char **error_buffer);

void search_result_free(struct SearchResult *result_ptr);

struct Document *document_create(void);

void document_add_field(struct Document *doc_ptr,
                        const char *field_name_ptr,
                        const char *field_value_ptr,
                        struct TantivyContext *context_ptr,
                        char **error_buffer);

char *document_as_json(struct Document *doc_ptr,
                       const char **include_fields_ptr,
                       uintptr_t include_fields_len,
                       Schema *schema_ptr,
                       char **error_buffer);

void document_free(struct Document *doc_ptr);

void string_free(char *s);

void init_lib(const char *log_level_ptr,
              char **error_buffer,
              bool clear_on_panic,
              bool utf8_lenient);
