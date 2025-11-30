#include "uaccess.h"

#include "kernel.h"
#include "pmap.h"
#include "kstring.h"
#include "task.h"

/* Walk the range one page at a time; the physical pages are reached through the kernel mapping. */
static int user_copy(struct task *t, uint64_t uva, void *k, size_t n, int to_user)
{
    uint8_t *kp = k;
    if (!t) {  /* kernel thread: the "user" pointer is a kernel pointer */
        if (to_user) memcpy((void *)(uintptr_t)uva, kp, n);
        else memcpy(kp, (void *)(uintptr_t)uva, n);
        return 0;
    }
    while (n) {
        uint64_t pte = pmap_lookup(&t->pmap, uva);
        uint64_t need = PTE_U | (to_user ? PTE_W : PTE_R);
        if (!pte || (pte & need) != need) return -1;
        size_t off = uva & (PAGE_SIZE - 1);
        size_t chunk = PAGE_SIZE - off < n ? PAGE_SIZE - off : n;
        uint8_t *up = (uint8_t *)phys_to_virt(pmap_extract(&t->pmap, uva));
        if (to_user) memcpy(up, kp, chunk);
        else memcpy(kp, up, chunk);
        uva += chunk;
        kp += chunk;
        n -= chunk;
    }
    return 0;
}

int copy_from_user(struct task *t, void *kdst, const void *usrc, size_t n)
{
    return user_copy(t, (uint64_t)(uintptr_t)usrc, kdst, n, 0);
}

int copy_to_user(struct task *t, void *udst, const void *ksrc, size_t n)
{
    return user_copy(t, (uint64_t)(uintptr_t)udst, (void *)ksrc, n, 1);
}
