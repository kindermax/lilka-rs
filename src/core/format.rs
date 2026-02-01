// use core::fmt::{self, write, write_fmt, Write};
// // use core::fmt::Write;
// use heapless::String;

#[macro_export]
macro_rules! format {
    ($capacity:expr, $($arg:tt)*) => {{
        let mut s = heapless::String::<$capacity>::new();
        core::write!(&mut s, $($arg)*).unwrap();
        s
    }};
}
