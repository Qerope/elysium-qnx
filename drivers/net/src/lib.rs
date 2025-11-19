//! virtio-net over virtio-mmio (version 2, "modern"). Hardware access goes through the C shim.
#![cfg_attr(not(test), no_std)]

extern "C" {
    fn hw_mmio_read32(addr: usize) -> u32;
    fn hw_mmio_write32(addr: usize, v: u32);
    fn hw_dma_alloc(size: usize, phys: *mut usize) -> *mut u8;
}

mod reg {
    pub const MAGIC: usize = 0x000;
    pub const VERSION: usize = 0x004;
    pub const DEVICE_ID: usize = 0x008;
    pub const DEVICE_FEATURES: usize = 0x010;
    pub const DEVICE_FEATURES_SEL: usize = 0x014;
    pub const DRIVER_FEATURES: usize = 0x020;
    pub const DRIVER_FEATURES_SEL: usize = 0x024;
    pub const QUEUE_SEL: usize = 0x030;
    pub const QUEUE_NUM_MAX: usize = 0x034;
    pub const QUEUE_NUM: usize = 0x038;
    pub const QUEUE_READY: usize = 0x044;
    pub const QUEUE_NOTIFY: usize = 0x050;
    pub const STATUS: usize = 0x070;
    pub const QUEUE_DESC_LOW: usize = 0x080;
    pub const QUEUE_DESC_HIGH: usize = 0x084;
    pub const QUEUE_DRIVER_LOW: usize = 0x090;
    pub const QUEUE_DRIVER_HIGH: usize = 0x094;
    pub const QUEUE_DEVICE_LOW: usize = 0x0a0;
    pub const QUEUE_DEVICE_HIGH: usize = 0x0a4;
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

const QUEUE_SIZE: usize = 8;
const BUF_SIZE: usize = 2048;
const HDR_LEN: usize = 12; // virtio_net_hdr with num_buffers (VERSION_1)

#[repr(C)]
#[derive(Clone, Copy)]
struct Desc {
    addr: u64,
    len: u32,
    flags: u16,
    next: u16,
}

const DESC_F_WRITE: u16 = 2;

#[repr(C)]
#[derive(Clone, Copy)]
struct UsedElem {
    id: u32,
    len: u32,
}

/// One split virtqueue in a single DMA page: descriptors, then the available ring, then the used ring.
/// Every packet uses exactly one descriptor, so descriptor index == buffer slot.
struct Virtqueue {
    desc: *mut Desc,
    avail_idx: *mut u16,
    avail_ring: *mut u16,
    used_idx: *const u16,
    used_ring: *const UsedElem,
    bufs: *mut u8,       // QUEUE_SIZE * BUF_SIZE bytes, CPU address
    bufs_phys: usize,
    last_used: u16,
    in_flight: [bool; QUEUE_SIZE],
}

impl Virtqueue {
    unsafe fn new(mem: *mut u8, phys: usize, bufs: *mut u8, bufs_phys: usize) -> (Virtqueue, [usize; 3]) {
        let desc_off = 0;
        let avail_off = QUEUE_SIZE * core::mem::size_of::<Desc>();
        let used_off = (avail_off + 4 + 2 * QUEUE_SIZE + 3) & !3;
        let q = Virtqueue {
            desc: mem.add(desc_off) as *mut Desc,
            avail_idx: mem.add(avail_off + 2) as *mut u16,
            avail_ring: mem.add(avail_off + 4) as *mut u16,
            used_idx: mem.add(used_off + 2) as *const u16,
            used_ring: mem.add(used_off + 4) as *const UsedElem,
            bufs,
            bufs_phys,
            last_used: 0,
            in_flight: [false; QUEUE_SIZE],
        };
        (q, [phys + desc_off, phys + avail_off, phys + used_off])
    }

    fn buf(&self, slot: usize) -> *mut u8 {
        unsafe { self.bufs.add(slot * BUF_SIZE) }
    }

    /// Hands buffer `slot` to the device. `write` = device writes into it (rx).
    fn submit(&mut self, slot: usize, len: usize, write: bool) {
        unsafe {
            *self.desc.add(slot) = Desc {
                addr: (self.bufs_phys + slot * BUF_SIZE) as u64,
                len: len as u32,
                flags: if write { DESC_F_WRITE } else { 0 },
                next: 0,
            };
            let idx = core::ptr::read_volatile(self.avail_idx);
            core::ptr::write_volatile(self.avail_ring.add(idx as usize % QUEUE_SIZE), slot as u16);
            core::sync::atomic::fence(core::sync::atomic::Ordering::Release);
            core::ptr::write_volatile(self.avail_idx, idx.wrapping_add(1));
        }
        self.in_flight[slot] = true;
    }

    fn free_slot(&self) -> Option<usize> {
        self.in_flight.iter().position(|&b| !b)
    }
    /// Next completed buffer as (slot, length written by the device).
    fn pop_used(&mut self) -> Option<(usize, usize)> {
        let used = unsafe { core::ptr::read_volatile(self.used_idx) };
        if self.last_used >= used {
            return None;
        }
        let e = unsafe { core::ptr::read_volatile(self.used_ring.add(self.last_used as usize % QUEUE_SIZE)) };
        self.last_used += 1;
        self.in_flight[e.id as usize] = false;
        Some((e.id as usize, e.len as usize))
    }
}

pub struct NetDevice {
    base: usize,
    pub mac: [u8; 6],
    rx: Virtqueue,
    tx: Virtqueue,
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
        let rx = Self::setup_queue(base, 0)?;
        let tx = Self::setup_queue(base, 1)?;
        let mut dev = NetDevice { base, mac, rx, tx };
        dev.fill_rx();
        wr(base, reg::STATUS, STATUS_ACK | STATUS_DRIVER | STATUS_FEATURES_OK | STATUS_DRIVER_OK);
        Ok(dev)
    }

    fn setup_queue(base: usize, index: u32) -> Result<Virtqueue, Error> {
        wr(base, reg::QUEUE_SEL, index);
        if rd(base, reg::QUEUE_NUM_MAX) < QUEUE_SIZE as u32 {
            return Err(Error::Queue);
        }
        let mut ring_phys = 0usize;
        let mut buf_phys = 0usize;
        // The rings fit in one page; the packet buffers need QUEUE_SIZE * 2K = 16K, so allocate them
        // as consecutive pages through repeated calls only if the shim hands out contiguous memory.
        let ring = unsafe { hw_dma_alloc(4096, &mut ring_phys) };
        let bufs = unsafe { hw_dma_alloc(BUF_SIZE * QUEUE_SIZE / 4, &mut buf_phys) };
        if ring.is_null() || bufs.is_null() {
            return Err(Error::Queue);
        }
        let (q, addrs) = unsafe { Virtqueue::new(ring, ring_phys, bufs, buf_phys) };
        wr(base, reg::QUEUE_NUM, QUEUE_SIZE as u32);
        wr(base, reg::QUEUE_DESC_LOW, addrs[0] as u32);
        wr(base, reg::QUEUE_DESC_HIGH, (addrs[0] as u64 >> 32) as u32);
        wr(base, reg::QUEUE_DRIVER_LOW, addrs[1] as u32);
        wr(base, reg::QUEUE_DRIVER_HIGH, (addrs[1] as u64 >> 32) as u32);
        wr(base, reg::QUEUE_DEVICE_LOW, addrs[2] as u32);
        wr(base, reg::QUEUE_DEVICE_HIGH, (addrs[2] as u64 >> 32) as u32);
        wr(base, reg::QUEUE_READY, 1);
        Ok(q)
    }

    fn fill_rx(&mut self) {
        while let Some(slot) = self.rx.free_slot() {
            self.rx.submit(slot, BUF_SIZE, true);
        }
        wr(self.base, reg::QUEUE_NOTIFY, 0);
    }
}
