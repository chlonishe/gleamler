use std::fmt;
use std::sync::OnceLock;

/// Log levels for Gleamler NIF messages.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Debug,
    Info,
    Warn,
    Error,
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Level::Debug => write!(f, "DEBUG"),
            Level::Info => write!(f, "INFO"),
            Level::Warn => write!(f, "WARN"),
            Level::Error => write!(f, "ERROR"),
        }
    }
}

type LogHandler = fn(Level, &str, fmt::Arguments);

static CUSTOM_LOGGER: OnceLock<LogHandler> = OnceLock::new();

/// Sets a custom log handler (e.g. for forwarding logs to an Erlang logger or test sink).
pub fn set_logger(handler: LogHandler) -> bool {
    CUSTOM_LOGGER.set(handler).is_ok()
}

#[doc(hidden)]
pub fn __log_message(level: Level, module: &str, args: fmt::Arguments) {
    if let Some(handler) = CUSTOM_LOGGER.get() {
        handler(level, module, args);
    } else {
        eprintln!("[{level}] [{module}] {args}");
    }
}

/// Logs a message with an explicit level.
#[macro_export]
macro_rules! log {
    ($level:expr, $($arg:tt)*) => {
        $crate::log::__log_message($level, module_path!(), format_args!($($arg)*))
    };
}

/// Logs an info message.
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        $crate::log!($crate::log::Level::Info, $($arg)*)
    };
}

/// Logs a warning message.
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        $crate::log!($crate::log::Level::Warn, $($arg)*)
    };
}

/// Logs an error message.
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::log!($crate::log::Level::Error, $($arg)*)
    };
}

/// Logs a debug message.
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        $crate::log!($crate::log::Level::Debug, $($arg)*)
    };
}
