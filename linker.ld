ENTRY(_start)

KERNBASE = 0xffffffff80000000;
PHYS = 0x80200000;
OFFSET = KERNBASE - 0x80000000;

SECTIONS
{
    . = KERNBASE + (PHYS - 0x80000000);
    .text : AT(ADDR(.text) - OFFSET) { *(.text.entry) *(.text*) }
    .rodata : AT(ADDR(.rodata) - OFFSET) { *(.rodata*) }
    . = ALIGN(4096);
    .data : AT(ADDR(.data) - OFFSET) { *(.data.boot) *(.data*) }
    .bss : AT(ADDR(.bss) - OFFSET) { _bss = .; *(.bss*) *(COMMON) _ebss = .; }
    . = ALIGN(4096);
    _kend = .;
}
