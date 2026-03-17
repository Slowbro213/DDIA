#ifndef BLOOM_H
#define BLOOM_H

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

typedef struct {
  uint8_t *bitmasks;
  size_t nbytes;
  uint32_t k;
}Bloom;

void bloom_init(Bloom *b, uint8_t *bitmasks, size_t nbytes, uint32_t k);
void bloom_put(Bloom *b,long key);
bool bloom_has(Bloom *b,long key);

#endif
