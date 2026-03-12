#ifndef SSTABLE_H
#define SSTABLE_H

typedef struct {
  long *keys;
  long *offsets;
  int length;
  int id;
}SSTable;


void sstable_init(SSTable *sst, long *keys, long *offsets, int id);
const char* sstable_get(SSTable *sst, long key);


#endif

