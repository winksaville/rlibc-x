//! Test that ex-x1 binary does not use libc.
//!
//! In Cargo.toml harness=false for test and it invokes the is-libc-used binary.
//! otherwise there are panic_handler conflicts with rlibc-x1 and std.

use std::path::Path;
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_root = Path::new(manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .expect("should find workspace root");

    // Find is-libc-used binary (required)
    let is_libc_used = ["target/release/is-libc-used", "target/debug/is-libc-used"]
        .iter()
        .map(|p| workspace_root.join(p))
        .find(|p| p.exists())
        .expect("is-libc-used binary not found (run 'cargo build -p is-libc-used' first)");

    // Find ex-x1 binary (required)
    let ex_x1 = ["target/release/ex-x1", "target/debug/ex-x1"]
        .iter()
        .map(|p| workspace_root.join(p))
        .find(|p| p.exists())
        .expect("ex-x1 binary not found (run 'cargo build -p ex-x1' first)");

    println!("test no_libc ... ");

    let output = Command::new(&is_libc_used)
        .arg("-v")
        .arg(&ex_x1)
        .output()
        .expect("failed to run is-libc-used");

    if output.status.success() {
        println!("ok");
        print!("{}", String::from_utf8_lossy(&output.stdout));
        ExitCode::SUCCESS
    } else {
        println!("FAILED");
        eprintln!("ex-x1 should NOT use libc:");
        eprint!("{}", String::from_utf8_lossy(&output.stdout));
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
        ExitCode::FAILURE
    }
}
