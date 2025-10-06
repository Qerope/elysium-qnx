#include <stdint.h>

#define KERNBASE 0xffffffff80000000UL
#define PHYS_BASE 0x80000000UL
#define DEVBASE 0xffffffff40000000UL
#define PTE_V (1UL << 0)
#define PTE_R (1UL << 1)
#define PTE_W (1UL << 2)
#define PTE_X (1UL << 3)
#define PTE_G (1UL << 5)
#define PTE_A (1UL << 6)
#define PTE_D (1UL << 7)
#define SATP_SV39 (8UL << 60)

/* In .data, not .bss: bss is cleared after paging is on and would wipe the live tables. */
__attribute__((section(".data.boot"), aligned(4096))) uint64_t early_root[512];

extern char kernel_entry[];

/* Runs with the MMU off but is linked high. Symbol addresses are pc-relative, so they come out
   physical here, which is what the page table and satp want. */
void boot_enable_paging(uint64_t hartid, uint64_t dtb)
{
    uint64_t ram = ((PHYS_BASE >> 12) << 10) | PTE_V | PTE_R | PTE_W | PTE_X | PTE_G | PTE_A | PTE_D;
    uint64_t dev = PTE_V | PTE_R | PTE_W | PTE_G | PTE_A | PTE_D;  /* first GiB, never executable */

    early_root[(PHYS_BASE >> 30) & 0x1ff] = ram;  /* identity, dropped by kmain */
    early_root[(KERNBASE >> 30) & 0x1ff] = ram;
    early_root[(DEVBASE >> 30) & 0x1ff] = dev;

    uint64_t kentry = (uint64_t)kernel_entry - PHYS_BASE + KERNBASE;
    register uint64_t a0 __asm__("a0") = hartid;
    register uint64_t a1 __asm__("a1") = dtb;
    __asm__ volatile("sfence.vma zero, zero\n csrw satp, %0\n sfence.vma zero, zero\n jr %1\n"
                     :: "r"(SATP_SV39 | ((uint64_t)early_root >> 12)), "r"(kentry), "r"(a0), "r"(a1) : "memory");
    __builtin_unreachable();
}

void boot_drop_identity(void)
{
    early_root[(PHYS_BASE >> 30) & 0x1ff] = 0;
    __asm__ volatile("sfence.vma zero, zero" ::: "memory");
}
