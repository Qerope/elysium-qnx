#pragma once
#include <stdint.h>

#define PTE_V (1UL << 0)
#define PTE_R (1UL << 1)
#define PTE_W (1UL << 2)
#define PTE_X (1UL << 3)
#define PTE_U (1UL << 4)

typedef struct pmap { uint64_t *root; } pmap_t;

int pmap_init(pmap_t *pm);
int pmap_enter(pmap_t *pm, uint64_t va, uint64_t pa, uint64_t prot);
int pmap_remove(pmap_t *pm, uint64_t va);
/* Returns the leaf PTE for va or 0. */
uint64_t pmap_lookup(pmap_t *pm, uint64_t va);
uint64_t pmap_extract(pmap_t *pm, uint64_t va);
