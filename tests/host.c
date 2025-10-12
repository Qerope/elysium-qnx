#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "kernel.h"

static uint8_t *arena;
#define FAKE_BASE 0x80000000UL
#define ARENA_SIZE (4UL << 20)

void *phys_to_virt(uint64_t pa)
{
    if (!arena) arena = aligned_alloc(PAGE_SIZE, ARENA_SIZE);
    return arena + (pa - FAKE_BASE);
}

uint64_t virt_to_phys(const void *va) { return (const uint8_t *)va - arena + FAKE_BASE; }

void panic(const char *msg)
{
    fprintf(stderr, "panic: %s\n", msg);
    abort();
}

void kprintf(const char *fmt, ...) { (void)fmt; }
