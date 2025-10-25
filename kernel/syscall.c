#include <stdint.h>

#include "kernel.h"
#include "sys.h"
#include "ipc.h"
#include "sched.h"

struct trapframe {
    uint64_t x[32];
    uint64_t sepc, sstatus, scause, stval;
};

/* a7 = number, a0..a5 = arguments. Results go back in a0; negative values are -errno. */
long syscall_dispatch(struct trapframe *tf)
{
    long num = (long)tf->x[17];
    switch (num) {
    case SYS_ChannelCreate: return ChannelCreate_();
    case SYS_ConnectAttach: return ConnectAttach_((int)tf->x[10]);
    case SYS_MsgSend: {
        int r = ipc_send(current, (int)tf->x[10], (const void *)tf->x[11], tf->x[12],
                         (void *)tf->x[13], tf->x[14]);
        if (r < 0) return r;
        current->state = T_REPLY;
        sched_block();
        return current->status;
    }
    case SYS_Write: {
        const char *s = (const char *)tf->x[10];
        for (size_t i = 0; i < tf->x[11]; i++) kprintf("%s", (char[]){s[i], 0});
        return (long)tf->x[11];
    }
    }
    return -38;  /* ENOSYS */
}
