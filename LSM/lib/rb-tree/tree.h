#ifndef RB_TREE_TREE_H
#define RB_TREE_TREE_H
#include <stdbool.h>
#include "./node.h"

typedef struct {
  RB_Node *nodes;

  int length;
  int size;
} RB_Tree;


void rb_tree_init(RB_Tree* rb_tree, RB_Node* nodes,int size);
bool rb_tree_put(RB_Tree* rb_tree,const char key[],const char *value);
const char* rb_tree_get(RB_Tree* rb_tree,const char key[]);
bool rb_tree_delete(RB_Tree* rb_tree,const char key[]);
void rb_tree_reset(RB_Tree* rb_tree);
void print_tree(RB_Tree* rb_tree);


#endif

