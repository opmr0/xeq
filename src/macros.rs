#[macro_export]
macro_rules! log {
        ($quiet:expr, $($arg:tt)*) => {
            if !$quiet {
                println!("{} {}", "[xeq]".cyan().bold(), format!($($arg)*));
            }
        };
    }

#[macro_export]
macro_rules! err {
        ($($arg:tt)*) => {
            eprintln!("{} {}", "[xeq]".red().bold(), format!($($arg)*).red());
        };
    }
