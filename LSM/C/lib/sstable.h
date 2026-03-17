#ifndef SSTABLE_H
#define SSTABLE_H

#include <stdio.h>
#include <stdbool.h>

#define SEGMENT_FILE_FMT "segments/segment_%lld.log"
#define SEGMENT_FILE_INDEX_FMT "segments/segment_index_%lld.ser"
#define SEGMENT_FILE_COUNT "segments/segment_count"

#define FRAME_MAGIC 0x4C534D31u  // "LSM1"

typedef struct {
  long *keys;
  long *offsets;
  int length;
}SSTable;


void sstable_init(SSTable *sst, long *keys, long *offsets);
const char* sstable_get(SSTable *sst, FILE* segment, long key);
bool sstable_add(SSTable *sst, FILE* segment_idx, long key, long offset, int idx);


#endif

