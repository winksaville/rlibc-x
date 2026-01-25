use std::process::ExitCode;

// Force rlibc-x2 to be linked
extern crate rlibc_x2;

fn main() -> ExitCode {
    ExitCode::from(42)
}
