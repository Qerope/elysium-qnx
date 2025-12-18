CROSS ?= riscv64-unknown-elf-
CC := $(CROSS)gcc
CFLAGS := -std=gnu11 -O2 -g -Wall -Wextra -ffreestanding -fno-pic -mcmodel=medany \
          -march=rv64imac_zicsr -mabi=lp64 -mno-relax -Iinclude
LDFLAGS := -nostdlib -T linker.ld

KSRCS := kernel/asm.S kernel/boot.c kernel/kmain.c
KLIBS :=
KSRCS += kernel/kprintf.c
KSRCS += kernel/vm_page.c
KSRCS += kernel/pmap.c
KSRCS += kernel/sched.c
KSRCS += kernel/trap.c
KSRCS += kernel/syscall.c
KSRCS += kernel/uaccess.c
KSRCS += kernel/ipc.c
KSRCS += kernel/shim.c
RUSTTARGET := riscv64imac-unknown-none-elf
RUSTLIB := target/$(RUSTTARGET)/release/libelysium_drivers.a
KLIBS += $(RUSTLIB)

$(RUSTLIB): $(shell find drivers -name '*.rs')
	cargo build --release -p elysium_drivers --target $(RUSTTARGET)

build/kernel.elf: $(KSRCS) linker.ld $(KLIBS)
	mkdir -p build
	$(CC) $(CFLAGS) $(LDFLAGS) -o $@ $(KSRCS) $(KLIBS)

run: build/kernel.elf
	qemu-system-riscv64 -machine virt -nographic -m 256M -bios default -kernel $<

HOSTCC ?= cc
HOSTFLAGS := -std=gnu11 -O1 -g -Wall -Wextra -DELY_HOST -Iinclude -Itests
TESTS := build/test_pmap
TESTS := build/test_pmap build/test_ipc
ifeq ($(SAN),1)
HOSTFLAGS += -fsanitize=address,undefined -fno-omit-frame-pointer
endif

build/test_pmap: tests/test_pmap.c kernel/pmap.c kernel/vm_page.c tests/host.c
	mkdir -p build
	$(HOSTCC) $(HOSTFLAGS) -o $@ $^

build/test_ipc: tests/test_ipc.c kernel/ipc.c tests/host.c
	mkdir -p build
	$(HOSTCC) $(HOSTFLAGS) -o $@ $^
smoke: build/kernel.elf
	sh scripts/qemu-smoke.sh build/kernel.elf

lint:
	cppcheck --error-exitcode=1 --enable=warning,performance,portability --inline-suppr -Iinclude -DELY_HOST kernel/ipc.c kernel/pmap.c kernel/vm_page.c kernel/uaccess.c
	cargo clippy -p elysium_drivers --release --target $(RUSTTARGET)

fmt:
	cargo fmt --all

test: $(TESTS)
	for t in $(TESTS); do $$t || exit 1; done

clean:
	rm -rf build target
