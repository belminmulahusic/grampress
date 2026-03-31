#[macro_export]
macro_rules! qprintln{
    ($qflag:expr, $($args:tt)*) => {
        if !$qflag{
            println!($($args)*);
        }
    };
}
