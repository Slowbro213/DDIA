#include <stdbool.h>
#include <stdio.h>

#include "../lib/rbtree.h"

static RBNode *get_node(RBTree *t, int idx) { return &t->nodes[idx]; }
static void init_node(RBNode *node, long key) {
  node->left_idx = 0;
  node->right_idx = 0;
  node->parent_idx = 0;
  node->key = key;
}

static const char *get_value(RBTree *t, int idx) { return t->values[idx]; }
static void set_value(RBTree *t, int idx, const char *value) {
  t->values[idx] = value;
}

static int new_node(RBTree *t) {
  if (t->next_free >= t->size)
    return 0;

  t->length++;
  return t->next_free++;
}

static int new_key_value_pair(RBTree *t, long key, const char *value) {
  int idx = new_node(t);
  if (idx == 0)
    return 0;
  RBNode *new_node = get_node(t, idx);
  init_node(new_node, key);
  t->values[idx] = value;
  return idx;
}

void rb_tree_init(RBTree *t, RBNode *nodes, const char **values, int size) {
  t->nodes = nodes;
  t->values = values;
  t->size = size;
  t->length = 0;
  t->next_free = 1;
}

const char *rb_tree_get(RBTree *t, long key) {
  if (t->length == 0)
    return NULL;

  int idx = 1;
  RBNode *node;
  for (;;) {
    node = get_node(t, idx);
    if (key < node->key && node->left_idx != 0)
      idx = node->left_idx;
    else if (key > node->key && node->right_idx != 0)
      idx = node->right_idx;
    else
      break;
    node = get_node(t, idx);
  }

  if (key != node->key)
    return NULL;
  return get_value(t, idx);
}

bool rb_tree_put(RBTree *t, long key, const char *value) {
  if (t->length == 0) {
    return new_key_value_pair(t, key, value)!=0;
  }

  int idx = 1;
  RBNode *node;
  for (;;) {
    node = get_node(t, idx);
    if (key < node->key) {
      if (node->left_idx == 0) {
        idx = new_key_value_pair(t, key, value);
        node->left_idx = idx;
        return true;
      }

      idx = node->left_idx;
    } else if (key > node->key) {
      if (node->right_idx == 0) {
        idx = new_key_value_pair(t, key, value);
        node->right_idx = idx;
        return true;
      }
      idx = node->right_idx;
    } else {
      set_value(t, idx, value);
      return true;
    }
  }

  return false;
}
