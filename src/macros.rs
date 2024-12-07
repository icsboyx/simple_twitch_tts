#[macro_export]
macro_rules! ErrorPrint {
    ($($arg:tt)*) => {
      eprintln!("{} {}", crate::colors::Colorize::red(&String::from("[ERROR]")), format!($($arg)*))
    };
    () => {
      eprintln!();
    };
}

#[macro_export]
macro_rules! WarningPrint {
    ($($arg:tt)*) => {
      eprintln!("{} {}", crate::colors::Colorize::yellow(&String::from("[WARNING]")), format!($($arg)*))
    };
    () => {
      eprintln!();
    };
}
