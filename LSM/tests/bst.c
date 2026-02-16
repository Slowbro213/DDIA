// tests/bst.c
#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <string.h>

#include "../lib/rbtree.h"

#define SIZE 128
#define EXTRA 1

static char *dup_str(const char *s) {
  size_t n = strlen(s) + 1;
  char *p = (char *)malloc(n);
  assert(p && "malloc failed");
  memcpy(p, s, n);
  return p;
}

int main(void) {
  RBNode *nodes = (RBNode *)malloc(sizeof(RBNode) * (SIZE+EXTRA));
  const char **values = (const char **)malloc(sizeof(const char *) * (SIZE+EXTRA));
  RBTree *t = (RBTree *)malloc(sizeof(RBTree));

  assert(nodes && values && t);

  rb_tree_init(t, nodes, values, SIZE+EXTRA);

  // ---- Insert unique keys ----
  for (long i = 0; i < SIZE; i++) {
    char buf[64];
    // Make keys non-trivial (not just 0..SIZE-1) to exercise ordering/rotations.
    long key = (i * 37L) ^ 0x5a5aL;

    snprintf(buf, sizeof(buf), "val_%ld", i);
    char *v = dup_str(buf);

    bool ok = rb_tree_put(t, key, v);
    assert(ok && "rb_tree_put failed (capacity? duplicate handling?)");
  }

  // ---- Verify all keys ----
  for (long i = 0; i < SIZE; i++) {
    long key = (i * 37L) ^ 0x5a5aL;

    const char *got = rb_tree_get(t, key);
    assert(got && "rb_tree_get returned NULL for existing key");

    char expected[64];
    snprintf(expected, sizeof(expected), "val_%ld", i);
    assert(strcmp(got, expected) == 0 && "wrong value returned");
  }

  // ---- Overwrite some keys (update semantics) ----
  for (long i = 0; i < SIZE; i += 3) {
    long key = (i * 37L) ^ 0x5a5aL;

    char buf[64];
    snprintf(buf, sizeof(buf), "val_%ld_updated", i);
    char *v2 = dup_str(buf);

    bool ok = rb_tree_put(t, key, v2);
    assert(ok && "rb_tree_put failed on overwrite");

    const char *got = rb_tree_get(t, key);
    assert(got);
    assert(strcmp(got, buf) == 0 && "overwrite did not take effect");
  }

  // ---- Missing keys should return NULL (or adjust to your API) ----
  for (long j = 0; j < 50; j++) {
    long missing_key = 1000000L + j * 17L;
    const char *got = rb_tree_get(t, missing_key);
    assert(got == NULL && "expected NULL for missing key");
  }

  puts("RBTree basic put/get tests passed.");

  // ---- Cleanup ----
  // If your tree stores pointers directly (no internal copy),
  // we need to free the allocated strings. We don't know the internal layout,
  // so we use rb_tree_get for keys we know, and free unique pointers.
  //
  // NOTE: If your rb_tree_put internally copies strings, remove this block.
  // NOTE: If overwrite discards old pointers, those old pointers are leaked here
  // unless rb_tree_put returns the old value or frees it internally.
  //
  // Best-effort: free current values for all keys we inserted.
  for (long i = 0; i < SIZE; i++) {
    long key = (i * 37L) ^ 0x5a5aL;
    const char *p = rb_tree_get(t, key);
    free((void *)p);
  }

  free(t);
  free(values);
  free(nodes);

  return 0;
}

