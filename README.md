# rlibc-x

Rust based libc experiment

I started this with this prompt for the claude-code 4.5:

"I want to create a rust app that simply returns a number, something like `fn main -> i32 { 2 }`. I want to
supply all code including the "standard libraries". I don't want the complication of code unwinding so panic="abort".
So the minimum set of functions in lib.rs is _start, panic, exit, free, malloc, realloc, exit and probably a few others.
And main.rs is something like `fn main() { exit(2) }`."

## Created by working with Claude Code 4.5

Thanks for the help Claude :)

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
