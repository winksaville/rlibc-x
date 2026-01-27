// Standalone test binary for func-analysis tests.
// Compiled directly with rustc - no workspace dependencies.
// This provides a simple ELF binary for function analysis testing.

fn helper_function(x: i32) -> i32 {
    x * 2
}

fn another_helper(a: i32, b: i32) -> i32 {
    a + b + helper_function(a)
}

fn main() {
    let result = another_helper(21, 21);
    std::process::exit(result as i32);
}
