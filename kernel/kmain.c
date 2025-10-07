#include <stdint.h>

#include "kernel.h"
void kprintf(const char *fmt, ...);
void boot_drop_identity(void);

void kmain(uint64_t hartid, uint64_t dtb)
{
    (void)dtb;
    boot_drop_identity();
    kprintf("elysium-qnx: hart %d\n", (int)hartid);
    for (;;) __asm__ volatile("wfi");
}
