#include <stdint.h>

#include "kernel.h"
void boot_drop_identity(void);

void kmain(uint64_t hartid, uint64_t dtb)
{
    (void)dtb;
    boot_drop_identity();
    for (;;) __asm__ volatile("wfi");
}
