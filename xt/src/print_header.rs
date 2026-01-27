#[macro_export]
macro_rules! print_header {
    ($title:expr) => {{
        $crate::print_hline!();
        println!("{:^width$}", $title, width = $crate::print_hline::LINE_WIDTH);
        $crate::print_hline!();
    }};
    ($title:expr, $width:expr) => {{
        $crate::print_hline!($width);
        println!("{:^width$}", $title, width = $width);
        $crate::print_hline!($width);
    }};
    ($title:expr, $width:expr, $ch:expr) => {{
        $crate::print_hline!($width, $ch);
        println!("{:^width$}", $title, width = $width);
        $crate::print_hline!($width, $ch);
    }};
}
