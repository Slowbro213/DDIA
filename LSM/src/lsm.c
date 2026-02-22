#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>
#include <errno.h>
#include <zlib.h>

#include "../lib/lsm.h"
#include "../lib/memtable.h"
#include "../lib/sstable.h.h"
#include "../lib/rbtree.h.h"

#define BLOCK_SIZE ( 1 << 16 )

#define SEGMENT_FILE_FMT "segments/segment_%lld.log"
#define SEGMENT_FILE_INDEX_FMT "segments/segment_index_%lld.ser"
#define SEGMENT_FILE_COUNT "segments/segment_count"

static uint64_t load_segment_count(void) {
  FILE *f = fopen(SEGMENT_FILE_COUNT, "rb");
  if (!f) {
    return 0;
  }

  uint64_t n = 0;
  size_t r = fread(&n, sizeof(n), 1, f);
  fclose(f);

  if (r != 1) {
    return 0;
  }
  return n;
}

static int store_segment_count(uint64_t next_id) {
  FILE *f = fopen(SEGMENT_FILE_COUNT, "wb");
  if (!f) return -1;

  size_t w = fwrite(&next_id, sizeof(next_id), 1, f);
  int ok = (w == 1 && fflush(f) == 0);
  fclose(f);

  return ok ? 0 : -1;
}

#define FRAME_MAGIC 0x4C534D31u  // "LSM1"

static int write_frame_compressed(FILE *f, const uint8_t *src, uint32_t src_len) {
  // Worst-case bound for zlib
  uLongf dst_cap = compressBound((uLong)src_len);
  uint8_t *dst = (uint8_t *)malloc(dst_cap);
  if (!dst) return -1;

  uLongf dst_len = dst_cap;
  int zrc = compress2(dst, &dst_len, src, (uLong)src_len, Z_DEFAULT_COMPRESSION);
  if (zrc != Z_OK) {
    free(dst);
    return -1;
  }

  uint32_t magic = FRAME_MAGIC;
  uint32_t ulen  = src_len;
  uint32_t clen  = (uint32_t)dst_len;

  if (fwrite(&magic, sizeof(magic), 1, f) != 1) { free(dst); return -1; }
  if (fwrite(&ulen,  sizeof(ulen),  1, f) != 1) { free(dst); return -1; }
  if (fwrite(&clen,  sizeof(clen),  1, f) != 1) { free(dst); return -1; }
  if (fwrite(dst, 1, clen, f) != clen)          { free(dst); return -1; }

  free(dst);
  return 0;
}

static int flush_buf_if_nonempty(Memtable *m, FILE *segment) {
  if (m->buf_len == 0) return 0;
  int rc = write_frame_compressed(segment, m->buf, (uint32_t)m->buf_len);
  if (rc == 0) m->buf_len = 0;
  return rc;
}

static int buf_append(LSM *l, unsigned long long id, FILE *segment, const void *data, size_t n) {
  Memtable *m = &l->m;
  const uint8_t *p = (const uint8_t *)data;

  while (n > 0) {
    size_t space = MT_BUF_CAP - m->buf_len;
    if (space == 0) {
      if (flush_buf_if_nonempty(m, segment) != 0) return -1;
      space = MT_BUF_CAP;
    }

    size_t take = (n < space) ? n : space;
    memcpy(m->buf + m->buf_len, p, take);
    m->buf_len += take;
    p += take;
    n -= take;

    if (m->buf_len == MT_BUF_CAP) {
      if (flush_buf_if_nonempty(m, segment) != 0) return -1;
    }
  }
  return 0;
}

void flush(LSM *l) {
  Memtable *m = &l->m;
  RBTree *t = &m->t;
  if (t->root_idx == 0 || t->length == 0) return;

  char path[256];
  uint64_t id = l->next_segment_id++;
  snprintf(path, sizeof(path), SEGMENT_FILE_FMT, (unsigned long long)id);

  FILE *segment = fopen(path, "wb");
  if (!segment) {
    perror("fopen segment");
    return;
  }

  m->buf_len = 0;

  int *stack = (int *)malloc(sizeof(int) * (size_t)(t->length + 1));
  if (!stack) {
    perror("malloc stack");
    fclose(segment);
    return;
  }
  int index_count = (l->m.total_size / BLOCK_SIZE);
  long *keys = malloc(sizeof(long) * index_count);
  long *offsets = malloc(sizeof(long) * index_count);
  SSTable *sst = &l->tables[id];
  sstable_init(sst, keys, offsets);
  size_t nbytes = l->m.t.length * sizeof(long);
  uint8_t *bitmasks = calloc(nbytes, 1);
  Bloom *b = &l->blooms[id];
  bloom_init(b, bitmasks, nbytes, 6);

  int sp = 0;
  int cur = t->root_idx;

  while (cur != 0 || sp > 0) {
    while (cur != 0) {
      stack[sp++] = cur;
      cur = t->nodes[cur].left_idx;
    }
    cur = stack[--sp];

    RBNode *n = &t->nodes[cur];
    Value  *v = &t->values[cur];

    int32_t len = n->tombstone ? -1 : (int32_t)v->length;

    if (buf_append(l, id, segment, &n->key, sizeof(n->key)) != 0) { perror("buf_append key"); break; }
    if (buf_append(l, id, segment, &len, sizeof(len)) != 0)       { perror("buf_append len"); break; }
    if (len > 0) {
      if (!v->value) { fprintf(stderr, "flush: NULL value with len>0\n"); break; }
      if (buf_append(l, id, segment, v->value, (size_t)len) != 0) { perror("buf_append val"); break; }
    }

    cur = n->right_idx;
  }

  free(stack);

  if (flush_buf_if_nonempty(m, segment) != 0) {
    perror("flush_buf_if_nonempty");
  }

  if (fflush(segment) != 0) perror("fflush");
  fclose(segment);

  l->next_segment_id = (id + 1) % (uint64_t)INT64_MAX;
  if (store_segment_count(l->next_segment_id) != 0) {
    perror("store_segment_count");
  }
}

void lsm_init(LSM *l, RBNode *nodes, Value *values, int size, bool owns_values){
  l->next_segment_id = load_segment_count();
  mt_init(&l->m, nodes, values, size, owns_values);
}

