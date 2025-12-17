## What and why

## How it was tested

- [ ] `make test` (host tests)
- [ ] `make smoke` (boots under qemu)
- [ ] Rust changes: `cargo test -p wlan`, `make lint`

## Notes for review

Anything touching `kernel/ipc.c` or the pmap should say what happens to a
blocked sender or a live mapping.
