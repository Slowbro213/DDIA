#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <stdbool.h>
#include <time.h>

#include "../lib/bloom.h"

#define NBYTES   1000000u
#define N_INSERT 1000000u
#define N_TEST   1000000u

static inline long rand_long(void) {
    return ((long)rand() << 32) ^ (long)rand();
}

int run(void) {
    srand((unsigned)time(NULL));

    uint8_t *bitmasks = calloc(NBYTES, 1);
    long *values = malloc((size_t)N_INSERT * sizeof(long));
    Bloom b = {
        .bitmasks = bitmasks,
        .nbytes   = NBYTES,
        .k        = 6,
    };

    for (uint32_t i = 0; i < N_INSERT; ++i) {
        values[i] = rand_long();
        bloom_put(&b, values[i]);
    }

    uint32_t false_negatives = 0;
    for (uint32_t i = 0; i < N_INSERT; ++i) {
        if (!bloom_has(&b, values[i])) false_negatives++;
    }

    uint32_t false_positives = 0;
    for (uint32_t i = 0; i < N_TEST; ++i) {
        long x = rand_long();
        if (bloom_has(&b, x)) false_positives++;
    }

    printf("Filter bytes:      %u (bits=%u)\n", NBYTES, NBYTES * 8u);
    printf("k probes:          %u\n", b.k);
    printf("Inserted elements: %u\n", N_INSERT);
    printf("False negatives:   %u\n", false_negatives);
    printf("False positives:   %u (%.2f%%)\n",
           false_positives,
           100.0 * (double)false_positives / (double)N_TEST);

    free(values);
    free(bitmasks);
    return 0;
}
