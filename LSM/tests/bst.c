// tests/bst.c
#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <string.h>
#include <time.h>
#include <stdint.h>

#include "../lib/rbtree.h"

// Sizes to benchmark: 2^MIN_POW ... 2^MAX_POW (capped at SIZE)
#define MIN_POW 10     // 1 Ki ops
#define MAX_POW 25     // 1 Mi ops

#define SIZE   (1 << MAX_POW)
#define EXTRA  1


#define REPEATS 1

static inline uint64_t ns_now(void) {
  struct timespec ts;
  clock_gettime(CLOCK_MONOTONIC, &ts);
  return (uint64_t)ts.tv_sec * 1000000000ULL + (uint64_t)ts.tv_nsec;
}

static void print_metrics(const char *label, size_t nops, uint64_t ns) {
  double sec = (double)ns / 1e9;
  double ops_sec = sec > 0.0 ? (double)nops / sec : 0.0;
  double ns_op = nops ? (double)ns / (double)nops : 0.0;
  printf("%-12s  ops=%10zu  time=%9.3f ms  %10.2f Mops/s  %9.1f ns/op\n",
         label, nops, (double)ns / 1e6, ops_sec / 1e6, ns_op);
}

// Deterministic pseudo-random-ish key transform (same as your original test)
static inline long make_key(long i) {
  return (i * 37L) ^ 0x5a5aL;
}

// Build fixed-size, NUL-terminated value strings so rb_tree_put gets stable pointers.
// We avoid malloc-per-op and snprintf in timed sections.
static void build_values(char *vals, char *vals_upd, long *keys,
                         size_t N, size_t stride) {
  for (size_t i = 0; i < N; ++i) {
    keys[i] = make_key((long)i);

    char *v  = vals     + i * stride;
    char *vu = vals_upd + i * stride;

    // Keep it deterministic and cheap; formatting here is NOT timed.
    // Ensure NUL-termination and stable length.
    // Example: "val_12345"
    int n1 = snprintf(v,  stride, "val_%zu", i);
    int n2 = snprintf(vu, stride, "val_%zu_updated", i);
    assert(n1 > 0 && (size_t)n1 < stride);
    assert(n2 > 0 && (size_t)n2 < stride);
  }
}

static inline int value_len_at(const char *base, size_t i, size_t stride) {
  const char *p = base + i * stride;
  return (int)strlen(p) + 1;
}

int main(void) {
  RBNode *nodes = (RBNode *)malloc(sizeof(RBNode) * (SIZE + EXTRA));
  Value  *values = (Value *)calloc((SIZE + EXTRA), sizeof(Value));
  RBTree *t = (RBTree *)malloc(sizeof(RBTree));
  assert(nodes && values && t);

  // Preallocate key/value material for the largest size once.
  // stride: room for "val_<up to 7 digits>_updated\0" comfortably.
  const size_t STRIDE = 64;

  long *keys = (long *)malloc(sizeof(long) * SIZE);
  char *vals = (char *)malloc(STRIDE * SIZE);
  char *vals_upd = (char *)malloc(STRIDE * SIZE);
  assert(keys && vals && vals_upd);

  build_values(vals, vals_upd, keys, SIZE, STRIDE);

  puts("RBTree performance benchmark (tree-only; allocation excluded)");
  puts("------------------------------------------------------------------");
  puts("Each size runs: insert N, get N, overwrite N/3 (stride=3)");
  puts("All keys/values are prebuilt outside timed regions (no malloc/snprintf in loops).");
  puts("------------------------------------------------------------------\n");

  printf("%8s  %-12s  %-52s\n", "N", "phase", "metrics");
  printf("--------------------------------------------------------------------------------\n");

  for (int pow = MIN_POW; pow <= MAX_POW; ++pow) {
    size_t N = (size_t)1u << pow;
    if (N > SIZE) break;

    for (int r = 0; r < REPEATS; ++r) {
      rb_tree_init(t, nodes, values, (int)(SIZE + EXTRA), false);

      // ---- Insert unique keys (timed) ----
      uint64_t t0 = ns_now();
      for (size_t i = 0; i < N; i++) {
        const char *v = vals + i * STRIDE;
        int vlen = value_len_at(vals, i, STRIDE);

        bool ok = rb_tree_put(t, keys[i], v, vlen);
        assert(ok && "rb_tree_put failed (capacity? duplicate handling?)");
      }
      uint64_t t1 = ns_now();
      uint64_t ins_ns = t1 - t0;

      // ---- Get all keys (timed) ----
      t0 = ns_now();
      for (size_t i = 0; i < N; i++) {
        Value *vv = rb_tree_get(t, keys[i]);
        assert(vv && "rb_tree_get returned NULL for existing key");
        (void)vv;
      }
      t1 = ns_now();
      uint64_t get_ns = t1 - t0;

      // ---- Overwrite some keys (timed) ----
      size_t overw_ops = 0;
      t0 = ns_now();
      for (size_t i = 0; i < N; i += 3) {
        const char *v2 = vals_upd + i * STRIDE;
        int v2len = value_len_at(vals_upd, i, STRIDE);

        bool ok = rb_tree_put(t, keys[i], v2, v2len);
        assert(ok && "rb_tree_put failed on overwrite");
        overw_ops++;
      }
      t1 = ns_now();
      uint64_t overw_ns = t1 - t0;

      // Print
      printf("%8zu  ", N);
      print_metrics("insert", N, ins_ns);
      printf("%8s  ", "");
      print_metrics("get", N, get_ns);
      printf("%8s  ", "");
      print_metrics("overwrite", overw_ops, overw_ns);

      // Spot-check at largest size only (not timed)
      if ((size_t)1u << MAX_POW == N || N == SIZE) {
        for (size_t i = 0; i < N; i += (N / 16 ? N / 16 : 1)) {
          Value *vv = rb_tree_get(t, keys[i]);
          assert(vv && vv->value);
        }
      }

      rb_tree_reset(t);
    }

    printf("--------------------------------------------------------------------------------\n");
  }

  free(vals_upd);
  free(vals);
  free(keys);

  free(t);
  free(values);
  free(nodes);
  return 0;
}

