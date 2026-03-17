#ifndef RB_TREE_TREE_H
#define RB_TREE_TREE_H
#include <stdbool.h>

typedef enum Color {
  RED,
  BLACK
}Color ;

typedef struct {
  long key;
  
  int left_idx;
  int right_idx;
  int parent_idx;
  Color color;
  bool tombstone;
} RBNode;

typedef struct {
  const char *value;
  int length;
} Value;

typedef struct {
  RBNode *nodes;
  Value *values;

  bool owns_values;
  int root_idx;
  int next_free;
  int length;
  int size;
} RBTree;


void rb_tree_init(RBTree* t, RBNode* nodes, Value *values, int size, bool owns_values);
bool rb_tree_put(RBTree* t, long key, const char *value, int length);
Value *rb_tree_get(RBTree* t, long key);
bool rb_tree_delete(RBTree* t, long key);
void rb_tree_reset(RBTree* t);


#endif

