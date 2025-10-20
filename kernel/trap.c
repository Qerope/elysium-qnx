#include <stdint.h>

#include "kernel.h"
#include "sys.h"

struct trapframe {
    uint64_t x[32];
    uint64_t sepc, sstatus, scause, stval;
};

extern void trap_entry(void);
long syscall_dispatch(struct trapframe *tf);

void trap_init(void)
{
    csr_write(stvec, trap_entry);
}

void trap_handler(struct trapframe *tf)
{
    uint64_t cause = tf->scause & ~(1UL << 63);
    if (tf->scause >> 63) {
        return;  /* interrupts are not used yet */
    }
    if (cause == 8) {  /* ecall from U-mode */
        tf->sepc += 4;
        tf->x[10] = syscall_dispatch(tf);
        return;
    }
    kprintf("fault: scause=%d sepc=%p stval=%p\n", (int)cause, (void *)tf->sepc, (void *)tf->stval);
    panic("unhandled exception");
}
