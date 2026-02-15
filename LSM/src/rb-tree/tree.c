#include <stdbool.h>
#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include "../../lib/rb-tree/node.h"
#include "../../lib/rb-tree/tree.h"

typedef struct {
  int idx;
  int prev_idx;
  bool left_idx;
  bool right_idx;
  bool is_same_node;
} Find_Result;

static int new_node(RB_Tree *rb_tree, Find_Result result) {
  RB_Node* nodes = rb_tree->nodes;
  RB_Node *head = &nodes[result.prev_idx];
  int next_free = rb_tree->length++ + 1;
  RB_Node *node = &nodes[next_free];


  if(result.left_idx) head->left_idx = next_free;
  if(result.right_idx) head->right_idx = next_free;

  node->parent_idx = result.prev_idx;
  node->left_idx = 0;
  node->right_idx = 0;
  return next_free;
}

static RB_Node* get_node(RB_Tree *rb_tree, Find_Result result){
  if(result.idx != 0)
    return &rb_tree->nodes[result.idx];
  int idx = new_node(rb_tree,result);
  return &rb_tree->nodes[idx];
}

static Find_Result find(RB_Tree *rb_tree, const char *key){
  RB_Node* nodes = rb_tree->nodes;
  int idx = 1, prev_idx;
  RB_Node *head = &nodes[idx];
  Find_Result find_result;
  bool is_same_node = false;

  do {
    prev_idx = idx;
    int comparison = strcmp(key, head->key);
    if(comparison < 0) {
      idx = head->left_idx; find_result.left_idx = true; find_result.right_idx = false;
    } else if(comparison == 0) {
      is_same_node = true;
    } else {
      idx = head->right_idx; find_result.left_idx = false; find_result.right_idx = true;
    }
    if(is_same_node) break;
    head = &nodes[idx];
  }while(head->parent_idx != 0);

  find_result.idx = idx;
  find_result.prev_idx = prev_idx;
  find_result.is_same_node = is_same_node;
  return find_result;
}

void rb_tree_init(RB_Tree* rb_tree, RB_Node* nodes, int size){
  rb_tree->size=size;
  rb_tree->nodes=nodes;
  rb_tree->length=0;
}

bool rb_tree_put(RB_Tree *rb_tree, const char *key, const char *value){
  if(rb_tree->length >= rb_tree->size - 1)
    return false;

  Find_Result result = find(rb_tree,key);
  RB_Node *node = get_node(rb_tree,result);

  memcpy(node->key, key, KEY_SIZE);
  node->value = value;

  return true;
}

const char* rb_tree_get(RB_Tree* rb_tree,const char key[]){
  Find_Result result = find(rb_tree,key);
  if(result.idx == 0) return NULL;

  RB_Node *node = get_node(rb_tree,result);
  return node->value;
}

static int min_in_subtree(RB_Tree *t, int idx) {
  while (idx != 0 && t->nodes[idx].left_idx != 0) {
    idx = t->nodes[idx].left_idx;
  }
  return idx;
}

static void replace_child(RB_Tree *t, int parent_idx, int old_child_idx, int new_child_idx) {
  if (parent_idx == 0) return;
  RB_Node *p = &t->nodes[parent_idx];
  if (p->left_idx == old_child_idx) p->left_idx = new_child_idx;
  else if (p->right_idx == old_child_idx) p->right_idx = new_child_idx;

  if (new_child_idx != 0) {
    t->nodes[new_child_idx].parent_idx = parent_idx;
  }
}

static void delete_at_idx(RB_Tree *t, int z) {
  RB_Node *Z = &t->nodes[z];

  if (Z->left_idx == 0 && Z->right_idx == 0) {
    if (z != 1) {
      replace_child(t, Z->parent_idx, z, 0);
      Z->parent_idx = 0;
      Z->left_idx = 0;
      Z->right_idx = 0;
      Z->key[0] = '\0';
      Z->value = NULL;
    } else {
      Z->left_idx = 0;
      Z->right_idx = 0;
      Z->key[0] = '\0';
      Z->value = NULL;
      t->length = 0;
    }
    return;
  }

  if (Z->left_idx == 0 || Z->right_idx == 0) {
    int child = (Z->left_idx != 0) ? Z->left_idx : Z->right_idx;

    if (z != 1) {
      replace_child(t, Z->parent_idx, z, child);

      Z->parent_idx = 0;
      Z->left_idx = 0;
      Z->right_idx = 0;
      Z->key[0] = '\0';
      Z->value = NULL;
    } else {
      RB_Node *C = &t->nodes[child];

      memcpy(Z->key, C->key, KEY_SIZE);
      Z->value = C->value;
      Z->left_idx = C->left_idx;
      Z->right_idx = C->right_idx;

      if (Z->left_idx != 0)  t->nodes[Z->left_idx].parent_idx = 1;
      if (Z->right_idx != 0) t->nodes[Z->right_idx].parent_idx = 1;

      C->parent_idx = 0;
      C->left_idx = 0;
      C->right_idx = 0;
      C->key[0] = '\0';
      C->value = NULL;
    }
    return;
  }

  int y = min_in_subtree(t, Z->right_idx);
  RB_Node *Y = &t->nodes[y];

  memcpy(Z->key, Y->key, KEY_SIZE);
  Z->value = Y->value;

  int y_child = Y->right_idx;
  if (y_child != 0) t->nodes[y_child].parent_idx = Y->parent_idx;

  replace_child(t, Y->parent_idx, y, y_child);

  Y->parent_idx = 0;
  Y->left_idx = 0;
  Y->right_idx = 0;
  Y->key[0] = '\0';
  Y->value = NULL;
}

bool rb_tree_delete(RB_Tree *rb_tree, const char key[]) {
  Find_Result result = find(rb_tree, key);
  if (result.idx == 0 || !result.is_same_node) return false;

  delete_at_idx(rb_tree, result.idx);
  return true;
}

void rb_tree_reset(RB_Tree *rb_tree){
  rb_tree->length = 0;
}

static void print_node(RB_Node *node){
  printf("{ key: %s, value: %s, left_idx: %d, right_idx: %d, parent_idx: %d }\n", node->key, node->value, node->left_idx, node->right_idx, node->parent_idx);
}
void print_tree(RB_Tree *rb_tree){
  RB_Node* nodes = rb_tree->nodes;
  int idx = 1;
  RB_Node *head = &nodes[idx];

  do {
    print_node(head);
    head = &nodes[++idx];
  } while( head->parent_idx != 0);

}


