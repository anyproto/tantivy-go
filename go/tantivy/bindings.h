#include <binding_typedefs.h>
#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct SearchResult SearchResult;

SchemaBuilder *schema_builder_new(char **error_buffer);

int schema_builder_add_text_field(SchemaBuilder *builder,
                                  const char *name,
                                  bool stored,
                                  char **error_buffer);

Schema *build_schema(SchemaBuilder *builder, char **error_buffer);

Index *create_index_with_schema(const char *path, Schema *schema, char **error_buffer);

TantivyDocument *create_document(void);

int add_field(TantivyDocument *doc_ptr,
              const char *field_name,
              const char *field_value,
              Index *index_ptr,
              char **error_buffer);

int add_document(Index *index_ptr, TantivyDocument *doc_ptr, char **error_buffer);

struct SearchResult *search_index(Index *index_ptr, const char *query, char **error_buffer);

TantivyDocument *get_next_result(struct SearchResult *result_ptr, char **error_buffer);

char *get_document_json(TantivyDocument *doc_ptr, Schema *schema, char **error_buffer);

void free_search_result(struct SearchResult *result_ptr);

void free_index(Index *index_ptr);

void free_string(char *s);

void free_schema_builder(SchemaBuilder *builder_ptr);

void free_schema(Schema *schema_ptr);

void free_document(TantivyDocument *doc_ptr);

uint8_t init(void);
