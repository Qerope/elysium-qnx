#include <assert.h>
#include <stdio.h>

#include "kernel.h"
#include "pmap.h"
#include "vm_page.h"

#define BASE 0x80000000UL

int main(void)
{
    vm_page_init(BASE, 1UL << 20);
    pmap_t pm;
    assert(pmap_init(&pm) == 0);

    uint64_t pa = vm_page_alloc();
    assert(pmap_enter(&pm, 0x400000, pa, PTE_R | PTE_W | PTE_U) == 0);
    assert(pmap_extract(&pm, 0x400123) == pa + 0x123);
    assert(pmap_lookup(&pm, 0x400000) & PTE_U);
    assert(pmap_enter(&pm, 0x400000, pa, PTE_R) == -1);                       /* already mapped */
    assert(pmap_enter(&pm, 0x401000, pa, PTE_R | PTE_W | PTE_X) == -1);       /* W^X */
    assert(pmap_enter(&pm, 0x0000004000000000UL, pa, PTE_R) == -1);           /* non-canonical */
    assert(pmap_enter(&pm, 0xffffffc000000000UL, pa, PTE_R) == 0);            /* canonical high half */
    assert(pmap_remove(&pm, 0x400000) == 0);
    assert(pmap_lookup(&pm, 0x400000) == 0);
    assert(vm_page_free(pa) == 0);
    assert(vm_page_free(pa) == -1);                                             /* double free */
    printf("pmap ok\n");
    return 0;
}
