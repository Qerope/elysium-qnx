# CI

| Job | What it checks |
|---|---|
| host tests | `make test` with gcc and clang, each with and without asan+ubsan |
| kernel build | cross build with `riscv64-unknown-elf-gcc` and the Rust staticlib; uploads `kernel.elf` |
| boot under qemu | boots the artifact on `qemu-system-riscv64 -machine virt` and looks for the boot banner |
| rust drivers | `cargo test -p wlan`, clippy for the riscv target; rustfmt is advisory |
| cppcheck | static analysis of the C files that have host tests |
| codeql | weekly and on every push |

Releases: pushing a `v*` tag builds the kernel and attaches the ELF, a flat
binary and SHA256SUMS to a GitHub release.

Known gaps: the qemu job only checks that the kernel reaches `kmain` and prints;
there is no test of interrupts, user mode or the drivers under qemu yet.
