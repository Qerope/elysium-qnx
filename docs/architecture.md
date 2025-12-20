# Architecture

## Kernel (C)

Boot: OpenSBI enters in S-mode; the image is linked in the top 2GiB and
`boot_enable_paging` turns on Sv39 from gigapage mappings before `kmain`.
Everything after that is ordinary kernel code. Memory is a page allocator plus a
Sv39 `pmap` that enforces W^X.

Message passing follows the QNX model (this is an independent implementation of
the *idea*, not QNX code):

- `MsgSend` blocks the client until the server replies (send/receive/reply).
- A server blocks in `MsgReceive` on a channel; clients reach it via a
  connection.
- While a server handles a message it runs at the client's priority if that is
  higher (priority inheritance), and drops back on `MsgReply`.
- Receive ids carry a generation counter, so a reply to a client that already
  moved on is refused instead of waking the wrong message.

Messages are copied once through a small kernel bounce buffer (`MSG_MAX` bytes)
using `copy_from_user`/`copy_to_user`, which check page permissions.

## Drivers (Rust)

All Rust code is `no_std` and reaches hardware only through `hw_shim.h`:
MMIO, DMA memory, logging, and (not yet implemented) IRQ attach. The drivers
are linked as one staticlib so there is a single panic handler.

- `uart`: ns16550, polled.
- `net`: virtio-net over virtio-mmio v2 with split virtqueues, one packet per
  descriptor. Served over messages by `net_server`.
- `wlan`: 802.11 management frames and a scan/auth/assoc state machine. Radio
  is a trait; the only backends are a null radio and the software AP in the
  tests. No real wireless hardware is supported.

## Not done

- Drivers still run inside the kernel image; making them separate user
  processes needs a process loader and per-process connection tables.
- No interrupts: drivers poll. The PLIC driver is not written.
- Connection ids are global.
- DMA is limited to a single page per allocation, so the virtio packet buffers
  are smaller than the queue layout would allow.
- The Rust crates have not been compiled in CI yet; only the C code and the
  pure-logic parts have been exercised.
