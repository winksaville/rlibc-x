fn main() {
    println!("cargo:rustc-flag=-C target-feature=+crt-static");
    println!("cargo:rustc-link-arg=-static");
    println!("cargo:rustc-link-arg=-nostdlib");
    println!("cargo:rustc-link-arg=-nodefaultlibs");
    println!("cargo:rustc-link-arg=-e_start");
    println!("cargo:rustc-link-arg=-Wl,--undefined=_start");
    println!("cargo:rustc-link-arg=-Wl,--undefined=__libc_start_main");
}
