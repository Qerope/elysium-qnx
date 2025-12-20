# elysium-qnx

[![ci](https://github.com/Qerope/elysium-qnx/actions/workflows/ci.yml/badge.svg)](https://github.com/Qerope/elysium-qnx/actions/workflows/ci.yml)
[![codeql](https://github.com/Qerope/elysium-qnx/actions/workflows/codeql.yml/badge.svg)](https://github.com/Qerope/elysium-qnx/actions/workflows/codeql.yml)

A RISC-V microkernel in the style of QNX Neutrino: the kernel does memory,
scheduling and synchronous message passing; everything else is a server.
Original code. The message API is QNX-*style* (`ChannelCreate`,
`ConnectAttach`, `MsgSend`, `MsgReceive`, `MsgReply`), not QNX source or
binary compatible.

## Layout

    kernel/    boot, Sv39 pmap, scheduler, traps, syscalls, message passing (C)
    drivers/   uart, net (virtio-net) and wlan, in Rust, no_std
    user/      C syscall stubs
    tests/     host tests for the pmap and message passing

Drivers talk to hardware only through the C shim in `include/hw_shim.h`.

## Build

    make test          # host tests, no cross toolchain
    make run           # riscv64-unknown-elf-gcc, rustup target riscv64imac-unknown-none-elf, qemu

## Status

The C parts that have host tests (pmap, message passing, user copy) are
tested. The Rust drivers have not been run on a device, and the wlan driver has
only been exercised against a software access point. There is no filesystem,
no TCP/IP, and no process manager yet. See docs/architecture.md.
