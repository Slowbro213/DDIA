#ifndef LSM_H
#define LSM_H

#include "bloom.h"
#include "memtable.h"
#include "sstable.h"

typedef struct {
  Bloom *blooms;
  SSTable *tables;

  Memtable m;
  unsigned long long next_segment_id;
} LSM;


void lsm_init(LSM* l, RBNode* nodes, Value *values, int size, bool owns_values);


#endif


