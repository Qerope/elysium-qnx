#pragma once

static inline void sbi_putchar(int c)
{
    register long a0 __asm__("a0") = c;
    register long a7 __asm__("a7") = 1;  /* legacy console putchar */
    __asm__ volatile("ecall" : "+r"(a0) : "r"(a7) : "memory");
}
