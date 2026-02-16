#ifndef RB_TREE_TREE_H
#define RB_TREE_TREE_H
#include <stdbool.h>

typedef struct {
  long key;
  
  int left_idx;
  int right_idx;
  int parent_idx;
} RBNode;

typedef struct {
  RBNode *nodes;
  const char **values;

  int next_free;
  int length;
  int size;
} RBTree;


void rb_tree_init(RBTree* t, RBNode* nodes, const char **values, int size);
bool rb_tree_put(RBTree* t, long key, const char *value);
const char* rb_tree_get(RBTree* t, long key);
bool rb_tree_delete(RBTree* t, long key);
void rb_tree_reset(RBTree* t);


#endif

