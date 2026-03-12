#include "../lib/sstable.h"
#include <unistd.h>


void sstable_init(SSTable *sst, long *keys, long *offsets, int id){
  sst->keys = keys;
  sst->offsets = offsets;
  sst->id = id;
  sst->length = 0;
}

static const char* segment_get(SSTable *sst, long key, long offset){

  return NULL;
}

const char* sstable_get(SSTable *sst, long key){
  int low = 0;
  int high = sst->length-1;

  while(low<=high){
    int mid = (low + high) / 2;
    long temp_key = sst->keys[mid];

    if(key == temp_key)
      return segment_get(sst, key, sst->offsets[mid]);
    if(key < temp_key)
      high = mid-1;
    if(key > temp_key)
      low = mid +1;
  }

  return segment_get(sst, key, sst->offsets[((low+high)/2)]);
}
