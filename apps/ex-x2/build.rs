fn main() {
    // Link statically, use rlibc-x2 instead of system libc
    println!("cargo:rustc-link-arg=-static");
    println!("cargo:rustc-link-arg=-nostdlib");
    println!("cargo:rustc-link-arg=-nodefaultlibs");

    // Set entry point and force linker to include our symbols
    println!("cargo:rustc-link-arg=-e_start");
    println!("cargo:rustc-link-arg=-Wl,--undefined=_start");
    println!("cargo:rustc-link-arg=-Wl,--undefined=__libc_start_main");
}
