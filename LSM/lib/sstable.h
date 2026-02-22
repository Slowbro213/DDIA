#ifndef SSTABLE_H
#define SSTABLE_H

typedef struct {
  long *keys;
  long *offsets;
  int length;
}SSTable;


void sstable_init(SSTable *sst, long *keys, long *offsets);

#endif

