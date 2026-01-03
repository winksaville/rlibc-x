#![no_std]
#![no_main]

#[unsafe(no_mangle)]
fn main() {
    my_libc::exit(2)
}
