#pragma once
#include <stddef.h>
#include <stdint.h>

/* The only way drivers (including the Rust ones) reach hardware and kernel services. */
uint8_t hw_mmio_read8(uintptr_t addr);
uint32_t hw_mmio_read32(uintptr_t addr);
void hw_mmio_write8(uintptr_t addr, uint8_t v);
void hw_mmio_write32(uintptr_t addr, uint32_t v);

typedef void (*hw_irq_handler)(void *arg);
int hw_irq_attach(unsigned irq, hw_irq_handler fn, void *arg);

/* Zeroed, physically contiguous memory. Returns a kernel virtual address and its physical address. */
void *hw_dma_alloc(size_t size, uintptr_t *phys);

void hw_log(const char *msg, size_t len);

/* Provided by the Rust uart driver. */
void uart_rs_init(uintptr_t base);
void uart_rs_putc(uint8_t c);
int uart_rs_getc(void);   /* -1 if nothing is waiting */
