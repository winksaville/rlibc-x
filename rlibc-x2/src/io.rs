//! I/O operations (read, write, etc.)

use crate::syscall::{SYS_READ, SYS_WRITE, syscall3};

#[unsafe(no_mangle)]
pub extern "C" fn read(fd: i32, buf: *mut u8, count: usize) -> isize {
    unsafe { syscall3(SYS_READ, fd as u64, buf as u64, count as u64) as isize }
}

#[unsafe(no_mangle)]
pub extern "C" fn write(fd: i32, buf: *const u8, count: usize) -> isize {
    unsafe { syscall3(SYS_WRITE, fd as u64, buf as u64, count as u64) as isize }
}

#[unsafe(no_mangle)]
pub extern "C" fn writev(_fd: i32, _iov: *const u8, _iovcnt: i32) -> isize {
    // TODO: implement properly
    -1
}
