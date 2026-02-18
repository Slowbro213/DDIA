#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>

#include "../lib/rbtree.h"
#include "string.h"

static inline RBNode *get_node(RBTree *t, int idx) { return &t->nodes[idx]; }
static inline void init_node(RBNode *node, long key) {
  node->left_idx = 0;
  node->right_idx = 0;
  node->parent_idx = 0;
  node->key = key;
  node->tombstone = false;
  node->color = RED;
}

static inline Value *get_value(RBTree *t, int idx) { return &t->values[idx]; }
static inline void set_value(RBTree *t, int idx, const char *value, int length) {
  if(t->owns_values && t->values[idx].value != NULL) free((void *)t->values[idx].value);
  t->values[idx].value = value;
  t->values[idx].length = length;
}
static inline void unset_value(RBTree *t, int idx) {
  if(t->owns_values && t->values[idx].value != NULL) free((void *)t->values[idx].value);
  t->values[idx].value = NULL;
  t->values[idx].length = -1;
}

static inline int new_node(RBTree *t) {
  if (t->next_free >= t->size)
    return 0;

  t->length++;
  return t->next_free++;
}

static inline int new_key_value_pair(RBTree *t, int parent_idx, long key, const char *value, int length) {
  int idx = new_node(t);
  if (idx == 0)
    return 0;
  RBNode *new_node = get_node(t, idx);
  init_node(new_node, key);
  set_value(t,idx,value,length);
  new_node->parent_idx = parent_idx;
  return idx;
}

static inline int is_red(RBTree *t, int idx) {
  return (idx != 0) && (get_node(t, idx)->color == RED);
}

static inline void set_color(RBTree *t, int idx, Color c) {
  if (idx != 0) get_node(t, idx)->color = c;
}

static void leftRotate(RBTree *t, int x_idx) {
  RBNode *x = get_node(t, x_idx);
  int y_idx = x->right_idx;
  RBNode *y = get_node(t, y_idx);

  x->right_idx = y->left_idx;
  if (y->left_idx != 0) {
    get_node(t, y->left_idx)->parent_idx = x_idx;
  }

  y->parent_idx = x->parent_idx;
  if (x->parent_idx == 0) {
    t->root_idx = y_idx;
  } else {
    RBNode *xp = get_node(t, x->parent_idx);
    if (xp->left_idx == x_idx) xp->left_idx = y_idx;
    else xp->right_idx = y_idx;
  }

  y->left_idx = x_idx;
  x->parent_idx = y_idx;
}

static void rightRotate(RBTree *t, int y_idx) {
  RBNode *y = get_node(t, y_idx);
  int x_idx = y->left_idx;
  RBNode *x = get_node(t, x_idx);

  y->left_idx = x->right_idx;
  if (x->right_idx != 0) {
    get_node(t, x->right_idx)->parent_idx = y_idx;
  }

  x->parent_idx = y->parent_idx;
  if (y->parent_idx == 0) {
    t->root_idx = x_idx;
  } else {
    RBNode *yp = get_node(t, y->parent_idx);
    if (yp->left_idx == y_idx) yp->left_idx = x_idx;
    else yp->right_idx = x_idx;
  }

  x->right_idx = y_idx;
  y->parent_idx = x_idx;
}

static void fixInsert(RBTree *t, int z_idx) {
  while (z_idx != t->root_idx && is_red(t, get_node(t, z_idx)->parent_idx)) {
    int p_idx = get_node(t, z_idx)->parent_idx;
    int g_idx = get_node(t, p_idx)->parent_idx;

    if (g_idx == 0) break;

    RBNode *g = get_node(t, g_idx);

    if (g->left_idx == p_idx) {
      int u_idx = g->right_idx;

      if (is_red(t, u_idx)) {
        set_color(t, p_idx, BLACK);
        set_color(t, u_idx, BLACK);
        set_color(t, g_idx, RED);
        z_idx = g_idx;
      } else {
        if (get_node(t, p_idx)->right_idx == z_idx) {
          z_idx = p_idx;
          leftRotate(t, z_idx);
          p_idx = get_node(t, z_idx)->parent_idx;
          g_idx = get_node(t, p_idx)->parent_idx;
        }
        set_color(t, p_idx, BLACK);
        set_color(t, g_idx, RED);
        rightRotate(t, g_idx);
      }
    } else {
      int u_idx = g->left_idx;

      if (is_red(t, u_idx)) {
        set_color(t, p_idx, BLACK);
        set_color(t, u_idx, BLACK);
        set_color(t, g_idx, RED);
        z_idx = g_idx;
      } else {
        if (get_node(t, p_idx)->left_idx == z_idx) {
          z_idx = p_idx;
          rightRotate(t, z_idx);
          p_idx = get_node(t, z_idx)->parent_idx;
          g_idx = get_node(t, p_idx)->parent_idx;
        }
        set_color(t, p_idx, BLACK);
        set_color(t, g_idx, RED);
        leftRotate(t, g_idx);
      }
    }
  }

  set_color(t, t->root_idx, BLACK);
}


void rb_tree_init(RBTree *t, RBNode *nodes, Value *values, int size, bool owns_values) {
  t->nodes = nodes;
  t->values = values;
  t->size = size;
  t->length = 0;
  t->next_free = 1;
  t->root_idx = 1;
  t->owns_values = owns_values;
}

Value* rb_tree_get(RBTree *t, long key) {
  if (t->length == 0)
    return NULL;

  int idx = t->root_idx;
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

  if (key != node->key || node->tombstone)
    return NULL;
  return get_value(t, idx);
}

bool rb_tree_put(RBTree *t, long key, const char *value, int length) {
  if (t->length == 0) {
    int idx = new_key_value_pair(t, 0, key, value, length);
    if(idx == 0) return false;
    set_color(t, idx, BLACK);
    return true;
  }

  int idx = t->root_idx;
  RBNode *node;
  for (;;) {
    node = get_node(t, idx);
    if (key < node->key) {
      if (node->left_idx == 0) {
        idx = new_key_value_pair(t, idx, key, value, length);
        node->left_idx = idx;
        fixInsert(t, idx);
        return true;
      }

      idx = node->left_idx;
    } else if (key > node->key) {
      if (node->right_idx == 0) {
        idx = new_key_value_pair(t, idx, key, value, length);
        node->right_idx = idx;
        fixInsert(t, idx);
        return true;
      }
      idx = node->right_idx;
    } else {
      set_value(t, idx, value, length);
      return true;
    }
  }

  return false;
}

bool rb_tree_delete(RBTree *t, long key){
  if(t->length==0) return false;

  int idx = t->root_idx;
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

  if (key != node->key || node->tombstone)
    return false;

  node->tombstone = true;
  unset_value(t, idx);
  return true;
}


void rb_tree_reset(RBTree *t){
  t->root_idx = 1;
  t->length = 0;
  t->next_free = 1;
  if(t->owns_values) {
    int size = t->size;
    Value *values = t->values;

    for(int i=0;i < size;++i){
      if(values[i].value != NULL && values[i].length != -1){ 
        free((void *)values[i].value);
      }
    }
  }
}
