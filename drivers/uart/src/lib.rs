//! ns16550 UART. Reaches the device only through the C shim.
#![no_std]

use core::sync::atomic::{AtomicUsize, Ordering};

extern "C" {
    fn hw_mmio_read8(addr: usize) -> u8;
    fn hw_mmio_write8(addr: usize, value: u8);
}

const RBR_THR: usize = 0;
const IER: usize = 1;
const FCR: usize = 2;
const LSR: usize = 5;
const LSR_RX_READY: u8 = 1 << 0;
const LSR_TX_EMPTY: u8 = 1 << 5;

static BASE: AtomicUsize = AtomicUsize::new(0);

fn read(off: usize) -> u8 {
    unsafe { hw_mmio_read8(BASE.load(Ordering::Relaxed) + off) }
}

fn write(off: usize, v: u8) {
    unsafe { hw_mmio_write8(BASE.load(Ordering::Relaxed) + off, v) }
}

pub fn init(base: usize) {
    BASE.store(base, Ordering::Relaxed);
    write(IER, 0);
    write(FCR, 1); // enable the FIFOs
}

pub fn putc(c: u8) {
    if BASE.load(Ordering::Relaxed) == 0 {
        return;
    }
    while read(LSR) & LSR_TX_EMPTY == 0 {}
    write(RBR_THR, c);
}

pub fn try_getc() -> Option<u8> {
    if BASE.load(Ordering::Relaxed) != 0 && read(LSR) & LSR_RX_READY != 0 {
        Some(read(RBR_THR))
    } else {
        None
    }
}
