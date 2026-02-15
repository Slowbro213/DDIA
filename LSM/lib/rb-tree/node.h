#ifndef RB_TREE_NODE_H
#define RB_TREE_NODE_H

#define KEY_SIZE ( 1 << 8 )

typedef struct {
  char key[KEY_SIZE];
  const char* value;

  int parent_idx;
  int left_idx;
  int right_idx;
} RB_Node;


#endif
