#[macro_export]
macro_rules! err {
    ($($arg:tt)*) => {{
        use colored::Colorize;
        eprintln!("{} {}", "[xeq]".red().bold(), format!($($arg)*).red());
    }};
}

#[macro_export]
macro_rules! log {
    ($quiet:expr, $($arg:tt)*) => {{
        if !$quiet {
            use colored::Colorize;
            println!("{} {}", "[xeq]".cyan().bold(), format!($($arg)*));
        }
    }};
}