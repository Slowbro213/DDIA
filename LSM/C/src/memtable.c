#include <sys/stat.h>
#include <sys/types.h>
 
#include "../lib/rbtree.h"
#include "../lib/memtable.h"

void mt_init(Memtable *m,RBNode *nodes, Value *values, int size, bool owns_values) {
  m->total_size = 0;
  RBTree *t = &m->t;

  t->nodes = nodes;
  t->values = values;
  t->size = size;
  t->length = 0;
  t->next_free = 1;
  t->root_idx = 1;
  t->owns_values = owns_values;
}


inline Value *mt_get(Memtable *m, long key){
  return rb_tree_get(&m->t, key);
}

bool mt_put(Memtable *m, long key, const char *value, int length){
  if(m->t.next_free >= m->t.size-1)
    return false;

  bool res = rb_tree_put(&m->t,key,value,length);
  if(res){
    m->total_size += sizeof(key) + length;
  }
}

bool mt_delete(Memtable *m, long key){
  return rb_tree_delete(&m->t, key);
}
