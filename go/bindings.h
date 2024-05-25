#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct Example Example;

struct Example *create_example(const char *name);

void example_set_name(struct Example *example_ptr, const char *name_ptr);

const char *example_get_name(const struct Example *example_ptr);

void delete_example(struct Example *ptr);
