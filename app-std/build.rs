fn main() {
    // Link statically and don't use system libc - use rlibc-x1 instead
    println!("cargo:rustc-link-arg=-static");
    println!("cargo:rustc-link-arg=-nostdlib");
    println!("cargo:rustc-link-arg=-nodefaultlibs");
    // Set entry point explicitly
    println!("cargo:rustc-link-arg=-e_start");
    // Force linker to pull in _start symbol (use -Wl to pass to linker)
    println!("cargo:rustc-link-arg=-Wl,--undefined=_start");
    println!("cargo:rustc-link-arg=-Wl,--undefined=__libc_start_main");
}
