//! One staticlib for all Rust drivers: a single panic handler and the C-visible entry points.
#![no_std]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn uart_rs_init(base: usize) {
    uart::init(base);
}

#[no_mangle]
pub extern "C" fn uart_rs_putc(c: u8) {
    uart::putc(c);
}

#[no_mangle]
pub extern "C" fn uart_rs_getc() -> i32 {
    uart::try_getc().map_or(-1, |c| c as i32)
}

extern "C" {
    fn MsgReceive(chid: i32, buf: *mut u8, len: usize, rcvid: *mut i32) -> isize;
    fn MsgReply(rcvid: i32, status: isize, msg: *const u8, len: usize) -> i32;
}

/// Request: byte 0 = op (1 = send frame, 2 = receive frame), rest = frame. Reply status is the
/// frame length for op 2, 0 for op 1, negative on error.
#[no_mangle]
pub extern "C" fn net_server(chid: i32, mmio_base: usize) -> ! {
    let mut dev = match net::NetDevice::new(mmio_base) {
        Ok(d) => d,
        Err(_) => loop {},
    };
    let mut req = [0u8; 256];
    let mut frame = [0u8; 256];
    loop {
        let mut rcvid = 0i32;
        let n = unsafe { MsgReceive(chid, req.as_mut_ptr(), req.len(), &mut rcvid) };
        if n < 1 {
            continue;
        }
        match req[0] {
            1 => {
                let r = dev.tx(&req[1..n as usize]).map_or(-1, |_| 0);
                unsafe { MsgReply(rcvid, r, core::ptr::null(), 0) };
            }
            2 => match dev.rx(&mut frame) {
                Some(len) => unsafe { MsgReply(rcvid, len as isize, frame.as_ptr(), len); },
                None => unsafe { MsgReply(rcvid, 0, core::ptr::null(), 0); },
            },
            _ => unsafe { MsgReply(rcvid, -22, core::ptr::null(), 0); },
        }
    }
}

/// Request: byte 0 = op (1 = connect, 2 = status), rest = SSID for connect.
/// Status reply: 0 idle, 1 connecting, 2 associated.
#[no_mangle]
pub extern "C" fn wlan_server(chid: i32) -> ! {
    let mut sta = wlan::Station::new(wlan::NullRadio, [0x02, 0, 0, 0, 0, 1]);
    let mut req = [0u8; 64];
    loop {
        let mut rcvid = 0i32;
        let n = unsafe { MsgReceive(chid, req.as_mut_ptr(), req.len(), &mut rcvid) };
        if n < 1 {
            continue;
        }
        let status = match req[0] {
            1 => {
                sta.connect(&req[1..n as usize]);
                0
            }
            2 => sta.state_code(),
            _ => -22,
        };
        unsafe { MsgReply(rcvid, status, core::ptr::null(), 0) };
    }
}
