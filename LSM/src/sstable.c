#include "../lib/sstable.h"
#include <stdio.h>
#include <unistd.h>
#include <stdint.h>
#include <string.h>
#include <zlib.h>


void sstable_init(SSTable *sst, long *keys, long *offsets){
  sst->keys = keys;
  sst->offsets = offsets;
  sst->length = 0;
}

static const char* segment_get(SSTable *sst, FILE *segment, int idx){
  long offset = sst->offsets[idx];

  if(idx == sst->length-1){

  } else {
    long next_offset = sst->offsets[idx+1];
    long size = next_offset - offset;

    fseek(segment, offset, SEEK_CUR);
    uint8_t buf[size];
    if(fread(buf,1,size,segment) != size) return NULL;

    uint32_t frame_magic;
    memcpy(&frame_magic, &buf[0], sizeof(uint32_t));

    if(frame_magic != FRAME_MAGIC) perror("frame_magic_mismatch");


    uint32_t src_len;
    memcpy(&src_len, &buf[4], sizeof(uint32_t));

    uint8_t src_buf[src_len];
    uncompress(src_buf,(uLongf *)&src_len,buf,size);

  }

  return NULL;
}

const char* sstable_get(SSTable *sst, FILE *segment, long key){
  int low = 0;
  int high = sst->length-1;

  while(low<=high){
    int mid = (low + high) / 2;
    long temp_key = sst->keys[mid];

    if(key == temp_key)
      return segment_get(sst, segment, mid);
    if(key < temp_key)
      high = mid-1;
    if(key > temp_key)
      low = mid +1;
  }

  return segment_get(sst, segment, ((low+high)/2));
}

bool sstable_add(SSTable *sst, FILE *segment_idx, long key, long offset, int idx){
  if(fwrite(&key,sizeof(key),1,segment_idx) != 1) return false;
  if(fwrite(&offset,sizeof(offset),1,segment_idx) != 1) return false;

  sst->keys[idx] = key;
  sst->offsets[idx] = offset;
  sst->length++;

  return true;
}
