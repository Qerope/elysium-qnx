#pragma once
#include <stddef.h>
#include <stdint.h>

#define PAGE_SIZE 4096UL
#ifndef ELY_HOST
#define KERNBASE 0xffffffff80000000UL
#define PHYS_BASE 0x80000000UL
#define phys_to_virt(pa) ((void *)((uintptr_t)(pa) - PHYS_BASE + KERNBASE))
#define virt_to_phys(va) ((uint64_t)(uintptr_t)(va) - KERNBASE + PHYS_BASE)
#define mmio(pa) ((volatile void *)(0xffffffff40000000UL + (pa)))

#define SSTATUS_SIE (1UL << 1)
#define SATP_SV39 (8UL << 60)
#define csr_read(csr) ({ uint64_t __v; __asm__ volatile("csrr %0, " #csr : "=r"(__v)); __v; })
#define csr_write(csr, v) __asm__ volatile("csrw " #csr ", %0" :: "r"((uint64_t)(v)))
#define csr_set(csr, v) __asm__ volatile("csrs " #csr ", %0" :: "r"((uint64_t)(v)))

static inline void tlb_flush_va(uint64_t va) { __asm__ volatile("sfence.vma %0, zero" :: "r"(va) : "memory"); }
static inline void tlb_flush_all(void) { __asm__ volatile("sfence.vma zero, zero" ::: "memory"); }
static inline unsigned long irq_save(void)
{
    unsigned long f;
    __asm__ volatile("csrrc %0, sstatus, %1" : "=r"(f) : "r"(SSTATUS_SIE) : "memory");
    return f & SSTATUS_SIE;
}
static inline void irq_restore(unsigned long f) { if (f) csr_set(sstatus, SSTATUS_SIE); }
#else
void *phys_to_virt(uint64_t pa);
uint64_t virt_to_phys(const void *va);
#define tlb_flush_va(va) ((void)(va))
#define tlb_flush_all() ((void)0)
static inline unsigned long irq_save(void) { return 0; }
static inline void irq_restore(unsigned long f) { (void)f; }
#endif
