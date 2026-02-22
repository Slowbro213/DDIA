#define _GNU_SOURCE
#include <stdio.h>
#include <stdint.h>
#include <stdbool.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <assert.h>

#include "../lib/rbtree.h"
#include "../lib/memtable.h"

#ifndef N_INSERTS
#define N_INSERTS 1000000
#endif

#ifndef PAYLOAD_MAX
#define PAYLOAD_MAX 64
#endif

#ifndef N_LOOKUPS
#define N_LOOKUPS 20000
#endif

#ifndef SEED
#define SEED 0xC0FFEEu
#endif

static inline uint64_t now_ns(void) {
  struct timespec ts;
  clock_gettime(CLOCK_MONOTONIC, &ts);
  return (uint64_t)ts.tv_sec * 1000000000ull + (uint64_t)ts.tv_nsec;
}

static inline uint32_t rng32(uint32_t *s) {
  uint32_t x = *s;
  x ^= x << 13;
  x ^= x >> 17;
  x ^= x << 5;
  *s = x;
  return x;
}

static void make_payload(char *buf, size_t cap, int key, uint32_t r) {
  snprintf(buf, cap, "k=%d r=%08x", key, r);
}

int main(void) {
  const int N = (int)N_INSERTS;

  RBNode *nodes = calloc((size_t)N + 1, sizeof(RBNode));
  Value  *vals  = calloc((size_t)N + 1, sizeof(Value));
  if (!nodes || !vals) {
    fprintf(stderr, "OOM allocating pools for %d inserts\n", N);
    free(nodes); free(vals);
    return 1;
  }

  Memtable m;
  mt_init(&m, nodes, vals, (size_t)N + 1, false);

  uint32_t s = (uint32_t)SEED;
  char payload[PAYLOAD_MAX];

  int *sample_keys = malloc((size_t)N_LOOKUPS * sizeof(int));
  if (!sample_keys) {
    fprintf(stderr, "OOM allocating sample_keys\n");
    free(vals); free(nodes);
    return 1;
  }

  uint64_t t0 = now_ns();

  for (int i = 0; i < N; i++) {
    int key = (int)(rng32(&s) & 0x7fffffff);

    make_payload(payload, sizeof(payload), key, rng32(&s));
    mt_put(&m, key, payload, (int)strlen(payload) + 1);

    if (i < (int)N_LOOKUPS) sample_keys[i] = key;
  }

  uint64_t t1 = now_ns();
  double secs_put = (t1 - t0) / 1e9;

  printf("Inserted %d entries in %.3f s (%.0f ops/s)\n",
         N, secs_put, N / secs_put);

  free(sample_keys);
  free(vals);
  free(nodes);
  return 0;
}
