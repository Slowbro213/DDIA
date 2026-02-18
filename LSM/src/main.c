#include <stdlib.h>
#include <string.h>

#include "../lib/rbtree.h"
#include "../lib/memtable.h"

#define SIZE 20
#define EXTRA 1

int main() {
  int keys[SIZE] = {1,2,3,4,5,6,7,8,9,10};

  const char *payloads[SIZE] = {
    "Thanas_1","Thanas_2","Thanas_3","Thanas_4","Thanas_5",
    "Thanas_6","Thanas_7","Thanas_8","Thanas_9","Thanas_10"
  };

  RBNode *nodes = calloc(SIZE + EXTRA, sizeof(RBNode));
  Value  *vals  = calloc(SIZE + EXTRA, sizeof(Value));
  Memtable m;

  mt_init(&m, nodes, vals, SIZE + EXTRA, false);

  for (int i = 0; i < SIZE/2; i++) {
    mt_put(&m, keys[i], payloads[i], (int)strlen(payloads[i]) + 1);
  }

  flush(&m);

  free(vals);
  free(nodes);
  return 0;
}

