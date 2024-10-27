// See also:
// https://os.phil-opp.com/freestanding-rust-binary/
// https://siliconislandblog.wordpress.com/2022/04/24/writing-a-no_std-compatible-crate-in-rust/
#![no_std]
#![no_main]

#[allow(unused_imports)]
use core::panic::PanicInfo;

use errore::prelude::*;
use talc::*;

static mut ARENA: [u8; 10000] = [0; 10000];

#[global_allocator]
static ALLOCATOR: Talck<spin::Mutex<()>, ClaimOnOom> = Talc::new(unsafe {
    // if we're in a hosted environment, the Rust runtime may allocate before
    // main() is called, so we need to initialize the arena automatically
    ClaimOnOom::new(Span::from_const_array(core::ptr::addr_of!(ARENA)))
})
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
