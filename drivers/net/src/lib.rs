//! virtio-net over virtio-mmio (version 2, "modern"). Hardware access goes through the C shim.
#![cfg_attr(not(test), no_std)]

extern "C" {
    fn hw_mmio_read32(addr: usize) -> u32;
    fn hw_mmio_write32(addr: usize, v: u32);
}

mod reg {
    pub const MAGIC: usize = 0x000;
    pub const VERSION: usize = 0x004;
    pub const DEVICE_ID: usize = 0x008;
    pub const DEVICE_FEATURES: usize = 0x010;
    pub const DEVICE_FEATURES_SEL: usize = 0x014;
    pub const DRIVER_FEATURES: usize = 0x020;
    pub const DRIVER_FEATURES_SEL: usize = 0x024;
    pub const STATUS: usize = 0x070;
    pub const CONFIG: usize = 0x100;
}

const MAGIC_VALUE: u32 = 0x7472_6976; // "virt"
const DEVICE_NET: u32 = 1;
const STATUS_ACK: u32 = 1;
const STATUS_DRIVER: u32 = 2;
const STATUS_DRIVER_OK: u32 = 4;
const STATUS_FEATURES_OK: u32 = 8;
const F_VERSION_1: u32 = 1; // bit 32, i.e. bit 0 of feature word 1
const F_MAC: u32 = 1 << 5;

#[derive(Debug)]
pub enum Error {
    NotVirtio,
    WrongDevice,
    Features,
    Queue,
    Full,
    TooBig,
}

pub struct NetDevice {
    base: usize,
    pub mac: [u8; 6],
}

fn rd(base: usize, off: usize) -> u32 {
    unsafe { hw_mmio_read32(base + off) }
}

fn wr(base: usize, off: usize, v: u32) {
    unsafe { hw_mmio_write32(base + off, v) }
}

impl NetDevice {
    pub fn new(base: usize) -> Result<NetDevice, Error> {
        if rd(base, reg::MAGIC) != MAGIC_VALUE || rd(base, reg::VERSION) != 2 {
            return Err(Error::NotVirtio);
        }
        if rd(base, reg::DEVICE_ID) != DEVICE_NET {
            return Err(Error::WrongDevice);
        }
        wr(base, reg::STATUS, 0);
        wr(base, reg::STATUS, STATUS_ACK);
        wr(base, reg::STATUS, STATUS_ACK | STATUS_DRIVER);

        // Only VERSION_1 and MAC are wanted; everything else (offloads, mergeable buffers) stays off.
        wr(base, reg::DEVICE_FEATURES_SEL, 0);
        let lo = rd(base, reg::DEVICE_FEATURES);
        wr(base, reg::DEVICE_FEATURES_SEL, 1);
        let hi = rd(base, reg::DEVICE_FEATURES);
        if hi & F_VERSION_1 == 0 {
            return Err(Error::Features);
        }
        wr(base, reg::DRIVER_FEATURES_SEL, 0);
        wr(base, reg::DRIVER_FEATURES, lo & F_MAC);
        wr(base, reg::DRIVER_FEATURES_SEL, 1);
        wr(base, reg::DRIVER_FEATURES, F_VERSION_1);
        wr(base, reg::STATUS, STATUS_ACK | STATUS_DRIVER | STATUS_FEATURES_OK);
        if rd(base, reg::STATUS) & STATUS_FEATURES_OK == 0 {
            return Err(Error::Features);
        }

        let mut mac = [0u8; 6];
        if lo & F_MAC != 0 {
            for (i, b) in mac.iter_mut().enumerate() {
                *b = unsafe { hw_mmio_read32(base + reg::CONFIG + i) } as u8;
            }
        }
        wr(base, reg::STATUS, STATUS_ACK | STATUS_DRIVER | STATUS_FEATURES_OK | STATUS_DRIVER_OK);
        Ok(NetDevice { base, mac })
    }
}
