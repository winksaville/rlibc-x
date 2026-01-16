fn main() {
    // Only apply these flags to the hw-x2 binary, not tests
    // Link statically, use rlibc-x2 instead of system libc
    println!("cargo:rustc-link-arg-bin=hw-x2=-static");
    println!("cargo:rustc-link-arg-bin=hw-x2=-nostdlib");
    println!("cargo:rustc-link-arg-bin=hw-x2=-nodefaultlibs");

    // Set entry point and force linker to include our symbols
    println!("cargo:rustc-link-arg-bin=hw-x2=-e_start");
    println!("cargo:rustc-link-arg-bin=hw-x2=-Wl,--undefined=_start");
    println!("cargo:rustc-link-arg-bin=hw-x2=-Wl,--undefined=__libc_start_main");
}
