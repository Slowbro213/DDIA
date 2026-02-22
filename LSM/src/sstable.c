#include "../lib/sstable.h"


void sstable_init(SSTable *sst, long *keys, long *offsets){
  sst->keys = keys;
  sst->offsets = offsets;
  sst->length = 0;
}
