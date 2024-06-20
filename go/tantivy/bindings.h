#include <binding_typedefs.h>
#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#define DOCUMENT_BUDGET_BYTES 50000000

typedef struct Document Document;

typedef struct SearchResult SearchResult;

SchemaBuilder *schema_builder_new(void);

int schema_builder_add_text_field(SchemaBuilder *builder_ptr,
                                  const char *field_name_ptr,
                                  bool stored,
                                  bool is_text,
                                  int index_record_option_const,
                                  const char *tokenizer_name_ptr,
                                  char **error_buffer);

Schema *schema_builder_build(SchemaBuilder *builder_ptr, char **error_buffer);

Index *index_create_with_schema(const char *path_ptr, Schema *schema_ptr, char **error_buffer);

int index_register_text_analyzer_ngram(Index *index_ptr,
                                       const char *tokenizer_name_ptr,
                                       uintptr_t min_gram,
                                       uintptr_t max_gram,
                                       bool prefix_only,
                                       char **error_buffer);

int index_register_text_analyzer_edge_ngram(Index *index_ptr,
                                            const char *tokenizer_name_ptr,
                                            uintptr_t min_gram,
                                            uintptr_t max_gram,
                                            uintptr_t limit,
                                            char **error_buffer);

int index_register_text_analyzer_simple(Index *index_ptr,
                                        const char *tokenizer_name_ptr,
                                        uintptr_t text_limit,
                                        const char *lang_str_ptr,
                                        char **error_buffer);

int index_register_text_analyzer_raw(Index *index_ptr,
                                     const char *tokenizer_name_ptr,
                                     char **error_buffer);

int index_add_and_consume_documents(Index *index_ptr,
                                    struct Document **docs_ptr,
                                    uintptr_t docs_len,
                                    char **error_buffer);

int index_delete_documents(Index *index_ptr,
                           const char *field_name_ptr,
                           const char **delete_ids_ptr,
                           uintptr_t delete_ids_len,
                           char **error_buffer);

uint64_t index_num_docs(Index *index_ptr, char **error_buffer);

struct SearchResult *index_search(Index *index_ptr,
                                  const char **field_names_ptr,
                                  uintptr_t field_names_len,
                                  const char *query_ptr,
                                  char **error_buffer,
                                  uintptr_t docs_limit);

void index_free(Index *index_ptr);

uintptr_t search_result_get_size(struct SearchResult *result_ptr, char **error_buffer);

struct Document *search_result_get_doc(struct SearchResult *result_ptr,
                                       uintptr_t index,
                                       char **error_buffer);

void search_result_free(struct SearchResult *result_ptr);

struct Document *document_create(void);

int document_add_field(struct Document *doc_ptr,
                       const char *field_name_ptr,
                       const char *field_value_ptr,
                       Index *index_ptr,
                       char **error_buffer);

char *document_as_json(struct Document *doc_ptr,
                       const char **include_fields_ptr,
                       uintptr_t include_fields_len,
                       Schema *schema_ptr,
                       char **error_buffer);

void document_free(struct Document *doc_ptr);

void string_free(char *s);

uint8_t init_lib(void);
