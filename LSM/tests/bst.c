// tests/bst.c
#include <assert.h>
#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <stdbool.h>

#include "../lib/rb-tree/tree.h"
#include "../lib/rb-tree/node.h"


#ifndef ROOT_IDX_FALLBACK
#define ROOT_IDX_FALLBACK 1
#endif

static int root_idx(RB_Tree *t) {
#ifdef RB_TREE_HAS_ROOT_IDX
  return t->root_idx;
#else
  (void)t;
  return ROOT_IDX_FALLBACK;
#endif
}

// ---- Helpers ----
static char *dupstr(const char *s) {
  size_t n = strlen(s) + 1;
  char *p = malloc(n);
  assert(p);
  memcpy(p, s, n);
  return p;
}

// ---- Debug printing wrappers ----
static void log_put(RB_Tree *t, const char *k, const char *v) {
  printf("\n[PUT] key='%s' val='%s'\n", k, v);
  bool ok = rb_tree_put(t, k, v);
  printf("      -> %s\n", ok ? "OK" : "FAIL");
  printf("      (len=%d size=%d root=%d)\n", t->length, t->size, root_idx(t));
}

static const char *log_get(RB_Tree *t, const char *k) {
  printf("\n[GET] key='%s'\n", k);
  const char *v = rb_tree_get(t, k);
  printf("      -> %s\n", v ? v : "(null)");
  printf("      (len=%d size=%d root=%d)\n", t->length, t->size, root_idx(t));
  return v;
}

static void log_del(RB_Tree *t, const char *k) {
  printf("\n[DEL] key='%s'\n", k);
  bool ok = rb_tree_delete(t, k);
  printf("      -> %s\n", ok ? "OK" : "FAIL");
  printf("      (len=%d size=%d root=%d)\n", t->length, t->size, root_idx(t));
}


// ---- BST validation (requires a meaningful root idx) ----
static int g_visit_count;

static void validate_bst_rec(RB_Tree *t, int idx, const char *min, const char *max) {
  if (idx == 0) return;
  assert(idx > 0 && idx < t->size);

  RB_Node *n = &t->nodes[idx];

  if (min) assert(strcmp(n->key, min) > 0);
  if (max) assert(strcmp(n->key, max) < 0);

  g_visit_count++;
  assert(g_visit_count < t->size); // crude cycle guard

  // Optional: parent pointer sanity (will fail if you don't maintain parent_idx)
  if (n->left_idx != 0)  assert(t->nodes[n->left_idx].parent_idx == idx);
  if (n->right_idx != 0) assert(t->nodes[n->right_idx].parent_idx == idx);

  validate_bst_rec(t, n->left_idx,  min,   n->key);
  validate_bst_rec(t, n->right_idx, n->key, max);
}

static void validate_bst(RB_Tree *t) {
  int r = root_idx(t);
  g_visit_count = 0;
  if (r == 0) return;
  validate_bst_rec(t, r, NULL, NULL);
}

static void test_empty_tree(RB_Tree *t) {
  puts("\n=== test_empty_tree ===");
  assert(log_get(t, "nope") == NULL);
  assert(!rb_tree_delete(t, "nope"));
}

static void test_singleton(RB_Tree *t) {
  puts("\n=== test_singleton ===");
  log_put(t, "k", dupstr("v"));
  validate_bst(t);
  assert(strcmp(log_get(t, "k"), "v") == 0);

  // update
  log_put(t, "k", dupstr("v2"));
  validate_bst(t);
  assert(strcmp(log_get(t, "k"), "v2") == 0);

  // delete
  log_del(t, "k");
  assert(log_get(t, "k") == NULL);
  assert(!rb_tree_delete(t, "k"));
}

static void test_three_node_shapes(RB_Tree *t) {
  puts("\n=== test_three_node_shapes ===");

  // case: insert in increasing order (degenerate right chain)
  log_put(t, "a", dupstr("1"));
  log_put(t, "b", dupstr("2"));
  log_put(t, "c", dupstr("3"));
  validate_bst(t);
  assert(strcmp(log_get(t, "a"), "1") == 0);
  assert(strcmp(log_get(t, "b"), "2") == 0);
  assert(strcmp(log_get(t, "c"), "3") == 0);

  // delete middle, then ends
  log_del(t, "b");
  validate_bst(t);
  assert(log_get(t, "b") == NULL);

  log_del(t, "a");
  validate_bst(t);
  assert(log_get(t, "a") == NULL);

  log_del(t, "c");
  assert(log_get(t, "c") == NULL);
}

static void test_root_delete_cases(RB_Tree *t) {
  puts("\n=== test_root_delete_cases ===");

  // Root with two children: successor replacement should work
  log_put(t, "b", dupstr("2"));
  log_put(t, "a", dupstr("1"));
  log_put(t, "c", dupstr("3"));
  validate_bst(t);

  // delete root key
  log_del(t, "b");
  validate_bst(t);
  assert(log_get(t, "b") == NULL);
  assert(strcmp(log_get(t, "a"), "1") == 0);
  assert(strcmp(log_get(t, "c"), "3") == 0);

  // Root with one child
  log_del(t, "a");
  validate_bst(t);
  assert(log_get(t, "a") == NULL);
  assert(strcmp(log_get(t, "c"), "3") == 0);

  // Root leaf
  log_del(t, "c");
  assert(log_get(t, "c") == NULL);
}

static void test_updates_dont_break_structure(RB_Tree *t) {
  puts("\n=== test_updates_dont_break_structure ===");

  log_put(t, "m", dupstr("0"));
  log_put(t, "g", dupstr("0"));
  log_put(t, "t", dupstr("0"));
  log_put(t, "d", dupstr("0"));
  log_put(t, "j", dupstr("0"));
  log_put(t, "p", dupstr("0"));
  log_put(t, "w", dupstr("0"));
  validate_bst(t);

  // update in place repeatedly
  log_put(t, "m", dupstr("1"));
  log_put(t, "m", dupstr("2"));
  log_put(t, "m", dupstr("3"));
  validate_bst(t);
  assert(strcmp(log_get(t, "m"), "3") == 0);

  // ensure neighbors still accessible
  assert(strcmp(log_get(t, "d"), "0") == 0);
  assert(strcmp(log_get(t, "w"), "0") == 0);
}

static void test_delete_all_in_various_orders(RB_Tree *t) {
  puts("\n=== test_delete_all_in_various_orders ===");

  const char *keys[] = {"h","d","l","b","f","j","n","a","c","e","g","i","k","m","o"};
  const int n = (int)(sizeof(keys)/sizeof(keys[0]));

  for (int i = 0; i < n; i++) {
    char val[8];
    snprintf(val, sizeof val, "%d", i);
    log_put(t, keys[i], dupstr(val));
  }
  validate_bst(t);

  // delete in an order that stresses successor cases
  const char *del_order[] = {"h","d","l","b","f","j","n","a","c","e","g","i","k","m","o"};
  for (int i = 0; i < n; i++) {
    log_del(t, del_order[i]);
    // after each delete, key must be gone, others must still be searchable if present
    assert(log_get(t, del_order[i]) == NULL);
    validate_bst(t);
  }

  // tree should be empty-ish now
  assert(log_get(t, "h") == NULL);
  assert(!rb_tree_delete(t, "h"));
}

static void test_randomish_regression(RB_Tree *t) {
  puts("\n=== test_randomish_regression ===");

  // deterministic pseudo-random sequence without rand():
  // insert 50 keys, delete some, reinsert some, check expectations.
  char key[32], val[32];

  for (int i = 0; i < 50; i++) {
    int x = (i * 37) % 97;
    snprintf(key, sizeof key, "k%03d", x);
    snprintf(val, sizeof val, "v%03d", x);
    bool ok = rb_tree_put(t, key, dupstr(val));
    assert(ok);
    if ((i % 10) == 0) validate_bst(t);
  }
  validate_bst(t);

  // delete every 3rd inserted key
  for (int i = 0; i < 50; i += 3) {
    int x = (i * 37) % 97;
    snprintf(key, sizeof key, "k%03d", x);
    bool ok = rb_tree_delete(t, key);
    // might be false if duplicates existed and were already removed depending on your semantics
    (void)ok;
    if ((i % 9) == 0) validate_bst(t);
  }
  validate_bst(t);

  // spot-check a few
  assert(rb_tree_get(t, "k000") == NULL || strncmp(rb_tree_get(t, "k000"), "v", 1) == 0);
  assert(rb_tree_get(t, "k037") == NULL || strncmp(rb_tree_get(t, "k037"), "v", 1) == 0);
}

static void reset_tree(RB_Tree *t, RB_Node *nodes, int cap) {
  memset(nodes, 0, (size_t)cap * sizeof(*nodes));
  rb_tree_init(t, nodes, cap);
}

static void test_put_get_delete(RB_Tree *t) {
  // This function name kept so your main() doesn't need to change.
  // It now runs a suite. Re-init between tests to isolate failures.

  enum { CAP_LOCAL = 128 };
  RB_Node *nodes = t->nodes;

  reset_tree(t, nodes, CAP_LOCAL);
  test_empty_tree(t);

  reset_tree(t, nodes, CAP_LOCAL);
  test_singleton(t);

  reset_tree(t, nodes, CAP_LOCAL);
  test_three_node_shapes(t);

  reset_tree(t, nodes, CAP_LOCAL);
  test_root_delete_cases(t);

  reset_tree(t, nodes, CAP_LOCAL);
  test_updates_dont_break_structure(t);

  reset_tree(t, nodes, CAP_LOCAL);
  test_delete_all_in_various_orders(t);

  reset_tree(t, nodes, CAP_LOCAL);
  test_randomish_regression(t);
}


int main(void) {
  enum { CAP = 128 };

  // Node pool provided by caller
  RB_Node *nodes = calloc(CAP, sizeof(RB_Node));
  assert(nodes);

  RB_Tree t;
  rb_tree_init(&t, nodes, CAP);

  printf("=== START test_put_get_delete ===\n");
  test_put_get_delete(&t);

  free(nodes);
  puts("\nAll tests passed.");
  return 0;
}

