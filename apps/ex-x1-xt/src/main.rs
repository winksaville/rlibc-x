#![no_std]
#![no_main]

#[unsafe(no_mangle)]
fn main() {
    rlibc_x1::exit(42)
}
