#ifndef MEMTABLE_H
#define MEMTABLE_H

#include <stddef.h>
#include <stdint.h>

#include "rbtree.h"

#define MT_BUF_CAP (1u << 16)

typedef struct {
  RBTree t;

  uint8_t  buf[MT_BUF_CAP];
  size_t   buf_len;
  unsigned long long next_segment_id;
} Memtable;


void mt_init(Memtable* m, RBNode* nodes, Value *values, int size, bool owns_values);
Value* mt_get(Memtable *m,long key);
bool mt_put(Memtable *m, long key, const char *value, int length);
bool mt_delete(Memtable *m, long key);
void flush(Memtable *m);



#endif




