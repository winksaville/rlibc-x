fn main() {
    // Only apply these flags to the hw-x1 binary, not tests
    println!("cargo:rustc-link-arg-bin=hw-x1=-nostartfiles");
    println!("cargo:rustc-link-arg-bin=hw-x1=-static");
}
