#!/bin/sh
# Boots the kernel under qemu for a few seconds and checks the boot banner.
set -eu
elf=${1:-build/kernel.elf}
out=$(mktemp)
trap 'rm -f "$out"' EXIT

timeout 15 qemu-system-riscv64 -machine virt -m 256M -bios default -kernel "$elf" \
    -display none -serial "file:$out" -monitor none -no-reboot || true

if ! grep -q "elysium-qnx: hart" "$out"; then
    echo "boot banner not found, console output was:" >&2
    cat "$out" >&2
    exit 1
fi
echo "boot ok"
