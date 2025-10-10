#pragma once
#include <stdint.h>

void vm_page_init(uint64_t base, uint64_t size);
uint64_t vm_page_alloc(void);
int vm_page_free(uint64_t pa);
