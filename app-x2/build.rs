fn main() {
    // Use static C runtime (this tells Rust to link statically)
    println!("cargo:rustc-flag=-C target-feature=+crt-static");

    // Link statically and don't use system libc - use rlibc-x2 instead
    println!("cargo:rustc-link-arg=-static");
    println!("cargo:rustc-link-arg=-nostdlib");
    println!("cargo:rustc-link-arg=-nodefaultlibs");

    // Set entry point explicitly
    println!("cargo:rustc-link-arg=-e_start");

    // Force linker to pull in our entry point symbols
    println!("cargo:rustc-link-arg=-Wl,--undefined=_start");
    println!("cargo:rustc-link-arg=-Wl,--undefined=__libc_start_main");
}
