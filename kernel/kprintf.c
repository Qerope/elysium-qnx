#include <stdarg.h>
#include <stdint.h>

#include "kernel.h"
#include "sbi.h"
#include "hw_shim.h"

static int uart_ready;

static void putc_(char c)
{
    if (uart_ready) {
        uart_rs_putc((uint8_t)c);
        return;
    }
    sbi_putchar(c);
}

void console_use_uart(uintptr_t base)
{
    uart_rs_init(base);
    uart_ready = 1;
}

static void put_num(uint64_t v, unsigned base)
{
    char buf[24];
    int n = 0;
    do { buf[n++] = "0123456789abcdef"[v % base]; v /= base; } while (v);
    while (n) putc_(buf[--n]);
}

void kprintf(const char *fmt, ...)
{
    va_list ap;
    va_start(ap, fmt);
    for (; *fmt; fmt++) {
        if (*fmt != '%') { if (*fmt == '\n') putc_('\r'); putc_(*fmt); continue; }
        switch (*++fmt) {
        case 's': { const char *s = va_arg(ap, const char *); while (s && *s) putc_(*s++); break; }
        case 'd': { int v = va_arg(ap, int); if (v < 0) { putc_('-'); v = -v; } put_num(v, 10); break; }
        case 'x': put_num(va_arg(ap, unsigned), 16); break;
        case 'p': putc_('0'); putc_('x'); put_num((uint64_t)va_arg(ap, void *), 16); break;
        case '%': putc_('%'); break;
        }
    }
    va_end(ap);
}

void panic(const char *msg)
{
    kprintf("panic: %s\n", msg);
    for (;;) __asm__ volatile("wfi");
}
