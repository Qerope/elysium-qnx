CROSS ?= riscv64-unknown-elf-
CC := $(CROSS)gcc
CFLAGS := -std=gnu11 -O2 -g -Wall -Wextra -ffreestanding -fno-pic -mcmodel=medany \
          -march=rv64imac -mabi=lp64 -mno-relax -Iinclude
LDFLAGS := -nostdlib -T linker.ld

KSRCS := kernel/asm.S kernel/boot.c kernel/kmain.c
KLIBS :=
KSRCS += kernel/kprintf.c
KSRCS += kernel/vm_page.c

build/kernel.elf: $(KSRCS) linker.ld $(KLIBS)
	mkdir -p build
	$(CC) $(CFLAGS) $(LDFLAGS) -o $@ $(KSRCS) $(KLIBS)

run: build/kernel.elf
	qemu-system-riscv64 -machine virt -nographic -m 256M -bios default -kernel $<
