fn main() {
    // Only apply these flags to the ex-x1 binary, not tests
    println!("cargo:rustc-link-arg-bin=ex-x1=-nostartfiles");
    println!("cargo:rustc-link-arg-bin=ex-x1=-static");
}
