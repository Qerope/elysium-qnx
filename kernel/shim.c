#include "hw_shim.h"

#include <stdint.h>

#include "kernel.h"
#include "kstring.h"
#include "ipc.h"
#include "sched.h"
#include "vm_page.h"

uint8_t hw_mmio_read8(uintptr_t a) { return *(volatile uint8_t *)a; }
uint32_t hw_mmio_read32(uintptr_t a) { return *(volatile uint32_t *)a; }
void hw_mmio_write8(uintptr_t a, uint8_t v) { *(volatile uint8_t *)a = v; }
void hw_mmio_write32(uintptr_t a, uint32_t v) { *(volatile uint32_t *)a = v; }

int hw_irq_attach(unsigned irq, hw_irq_handler fn, void *arg)
{
    (void)irq; (void)fn; (void)arg;
    return -1;  /* PLIC support is not written yet; drivers poll */
}

void *hw_dma_alloc(size_t size, uintptr_t *phys)
{
    if (size > PAGE_SIZE) return NULL;  /* one page is enough for the current drivers */
    uint64_t pa = vm_page_alloc();
    if (!pa) return NULL;
    void *va = phys_to_virt(pa);
    memset(va, 0, PAGE_SIZE);
    *phys = pa;
    return va;
}

void hw_log(const char *msg, size_t len)
{
    for (size_t i = 0; i < len; i++) kprintf("%s", (char[]){msg[i], 0});
}

/* Entry points for drivers that run as kernel threads (all of them, for now). Same semantics as the
   user-visible calls, without the ecall. */
long MsgReceive(int chid, void *buf, size_t len, int *rcvid)
{
    long n = ipc_receive(current, chid, buf, len, rcvid);
    if (n != 0) return n;
    current->state = T_RECV;
    sched_block();
    return current->status;
}

int MsgReply(int rcvid, long status, const void *msg, size_t len)
{
    return ipc_reply(current, rcvid, status, msg, len);
}
