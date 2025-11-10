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
