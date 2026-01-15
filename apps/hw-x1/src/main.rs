#![no_std]
#![no_main]

#[unsafe(no_mangle)]
fn main() {
    let msg = b"Hello from hw-x1!\n";
    rlibc_x1::write(1, msg.as_ptr(), msg.len());
    rlibc_x1::exit(0)
}
