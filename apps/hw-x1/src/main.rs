#![no_std]
#![no_main]

#[unsafe(no_mangle)]
fn main() {
    let msg = b"Hello from hw-x1!\n";
    // SAFETY: msg.as_ptr() points to msg.len() valid bytes
    unsafe { rlibc_x1::write(1, msg.as_ptr(), msg.len()) };
    rlibc_x1::exit(0)
}
