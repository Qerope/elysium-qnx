#include <stdint.h>

#include "kernel.h"
#include "sys.h"

struct trapframe {
    uint64_t x[32];
    uint64_t sepc, sstatus, scause, stval;
};

/* a7 = number, a0..a5 = arguments. Results go back in a0; negative values are -errno. */
long syscall_dispatch(struct trapframe *tf)
{
    long num = (long)tf->x[17];
    switch (num) {
    case SYS_Write: {
        const char *s = (const char *)tf->x[10];
        for (size_t i = 0; i < tf->x[11]; i++) kprintf("%s", (char[]){s[i], 0});
        return (long)tf->x[11];
    }
    }
    return -38;  /* ENOSYS */
}
