#include <stdint.h>

#include "kernel.h"
void kprintf(const char *fmt, ...);
#include "vm_page.h"
#include "sched.h"
extern char _kend[];
void boot_drop_identity(void);
void trap_init(void);

void kmain(uint64_t hartid, uint64_t dtb)
{
    (void)dtb;
    boot_drop_identity();
    kprintf("elysium-qnx: hart %d\n", (int)hartid);
    vm_page_init(virt_to_phys(_kend), 64UL << 20);
    sched_init();
    trap_init();
    for (;;) __asm__ volatile("wfi");
}
