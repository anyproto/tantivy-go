#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct Index Index;
typedef struct SchemaBuilder SchemaBuilder;

SchemaBuilder *schema_builder_new(void);

void schema_builder_add_text_field(SchemaBuilder *builder, const char *name, bool stored);

Index *create_index_with_schema_builder(const char *path, SchemaBuilder *builder);

Index *create_index(const char *path);

bool add_document(Index *index_ptr, const char *title, const char *body);

char *search_index(Index *index_ptr, const char *query);

void free_index(Index *index_ptr);

void free_string(char *s);

void free_schema_builder(SchemaBuilder *builder_ptr);

uint8_t init(void);
