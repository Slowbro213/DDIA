#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#include "../lib/bloom.h"

static inline uint64_t mix64(uint64_t x) {
    x += 0x9e3779b97f4a7c15ULL;
    x = (x ^ (x >> 30)) * 0xbf58476d1ce4e5b9ULL;
    x = (x ^ (x >> 27)) * 0x94d049bb133111ebULL;
    return x ^ (x >> 31);
}

static inline void set_bit(uint8_t *bits, size_t pos) {
    bits[pos >> 3] |= (uint8_t)(1u << (pos & 7));
}

static inline bool get_bit(const uint8_t *bits, size_t pos) {
    return (bits[pos >> 3] & (uint8_t)(1u << (pos & 7))) != 0;
}

void bloom_put(Bloom *b, long key) {
    size_t m = b->nbytes * 8u;
    uint64_t x = (uint64_t)key;

    uint64_t h1 = mix64(x);
    uint64_t h2 = mix64(x ^ 0xD6E8FEB86659FD93ULL);

    for (uint32_t i = 0; i < b->k; i++) {
        size_t pos = (size_t)((h1 + (uint64_t)i * h2) % m);
        set_bit(b->bitmasks, pos);
    }
}

bool bloom_has(Bloom *b, long key) {
    size_t m = b->nbytes * 8u;
    uint64_t x = (uint64_t)key;

    uint64_t h1 = mix64(x);
    uint64_t h2 = mix64(x ^ 0xD6E8FEB86659FD93ULL);

    for (uint32_t i = 0; i < b->k; i++) {
        size_t pos = (size_t)((h1 + (uint64_t)i * h2) % m);
        if (!get_bit(b->bitmasks, pos)) return false;
    }
    return true;
}
