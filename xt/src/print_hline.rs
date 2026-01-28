pub const LINE_WIDTH: usize = 44;
pub const LINE_CHAR: char = '=';

#[macro_export]
macro_rules! print_hline {
    () => {
        $crate::print_hline::print_hline_impl(
            $crate::print_hline::LINE_WIDTH,
            $crate::print_hline::LINE_CHAR,
        )
    };
    ($width:expr) => {
        $crate::print_hline::print_hline_impl($width, $crate::print_hline::LINE_CHAR)
    };
    ($width:expr, $ch:expr) => {
        $crate::print_hline::print_hline_impl($width, $ch)
    };
}

pub fn print_hline_impl(width: usize, ch: char) {
    let line: String = std::iter::repeat(ch).take(width).collect();
    println!("{line}");
}
