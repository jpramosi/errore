// See also:
// https://os.phil-opp.com/freestanding-rust-binary/
// https://siliconislandblog.wordpress.com/2022/04/24/writing-a-no_std-compatible-crate-in-rust/
#![no_std]
#![no_main]

#[allow(unused_imports)]
use core::panic::PanicInfo;
use core::ptr::addr_of;

use errore::prelude::*;
use talc::*;

static mut ARENA: [u8; 10000] = [0; 10000];

#[global_allocator]
static ALLOCATOR: Talck<spin::Mutex<()>, ClaimOnOom> =
    Talc::new(unsafe { ClaimOnOom::new(Span::from_array(addr_of!(ARENA) as *mut [u8; 10000])) })
        .lock();

/// -
#[derive(Error, Debug)]
pub enum Error {
    #[error("...")]
    Field,
}

pub fn func() -> Result<(), Ec> {
    err!(Error::Field)
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    loop {}
}
