use anyhow::{anyhow, Result};
use chrono::prelude::*;
use colored::*;
use std::env;

/// Defines the four main logging output levels
#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub enum LogEntryLevel {
    ERROR = 0x0,
    WARN = 0x1,
    INFO = 0x2,
    DEBUG = 0x3,
}

const DEFAULT_DATETIME_PRINT_FORMAT: &str = "%Y-%m-%d %H:%M:%S%.3f";

impl LogEntryLevel {
    pub fn from_string(s: &str) -> Result<LogEntryLevel> {
        match s.to_uppercase().as_str() {
            "DEBUG" => Ok(LogEntryLevel::DEBUG),
            "INFO" => Ok(LogEntryLevel::INFO),
            "WARN" => Ok(LogEntryLevel::WARN),
            "ERROR" => Ok(LogEntryLevel::ERROR),
            _ => Err(anyhow!("Invalid log level: {}", s)),
        }
    }

    /// Returns a minimum log level from the environment variable `STUMP_LOG_AT_LEVEL`
    /// or, if lacking that, the default log level INFO.
    pub fn from_env() -> Result<LogEntryLevel> {
        // If you don't like the default "STUMP_LOG_AT_LEVEL", set what you'd like in a build var
        // LOG_LEVEL_VAR_NAME
        let env_var_name = if let Some(v) = option_env!("LOG_LEVEL_VAR_NAME") {
            v.to_string()
        } else {
            "STUMP_LOG_AT_LEVEL".to_string()
        };

        if let Ok(e) = env::var(env_var_name) {
            LogEntryLevel::from_string(&e)
        } else {
            Ok(unsafe { MIN_LOG_LEVEL })
        }
    }
}

/// Global indicator for verbose output when printing via `vprintln` and/or `veprintln`. Not meant to be set directly,
/// instead via `set_verbose`
static mut IS_VERBOSE: bool = false;

/// Program-controlled minimum log level. Controls the printing of messages via `debug!()`, `info!()`, `warn!()`, and `error!()`
static mut MIN_LOG_LEVEL: LogEntryLevel = LogEntryLevel::WARN;

/// Type definition for optional callbacks to capture log output.
type FnPrint = dyn Fn(&String) + Send + Sync + 'static;

/// Global storage point for optional output callback.
static mut PRINT: Option<Box<FnPrint>> = None;

/// Provides an alternative print method for integration with other command line
/// options such as `informif`.
///
/// # Example
/// ```
/// use indicatif::ProgressBar;
/// use stump;
///
/// lazy_static! {
///     static ref PB: ProgressBar = ProgressBar::new(1);
/// }
///
/// fn main() {
///     stump::set_print(|s| {
///         PB.println(s);
///     });
/// }
/// ```
pub fn set_print<F: Fn(&String) + Send + Sync + 'static>(f: F) {
    unsafe {
        PRINT = Some(Box::new(f));
    }
}

/// Print the string to stdout, or if present, a user-provided print closure.
pub fn do_println(s: &String) {
    unsafe {
        if let Some(p) = &PRINT {
            p(s);
        } else {
            println!("{}", s);
        }
    }
}

/// Sets whether the verbose standard print macro prints to stdout or stays silent
///
/// # Example
/// ```
/// vprintln!("This won't print as the default is false");
///
/// stump::set_verbose(true);
///
/// // print to stdout
/// vprintln!("Print something {}", "Here");
///
/// // print to stderr
/// veprintln!("Print something {}", "Here");
///
/// stump::set_verbose(false);
///
/// // Don't print to stdout
/// vprintln("Again nothing will print");
/// ```
pub fn set_verbose(v: bool) {
    unsafe {
        IS_VERBOSE = v;
    }
}

/// Indicates whether the verbose flag is set
pub fn is_verbose() -> bool {
    unsafe { IS_VERBOSE }
}

/// Sets the global minimum logging level. Can be user-overridden with `STUMPLOG_AT_LEVEL`
pub fn set_min_log_level(min_log_level: LogEntryLevel) {
    unsafe {
        MIN_LOG_LEVEL = min_log_level;
    }
}

/// Retrieves the global minimum logging level. Does not check the user env var.
pub fn get_min_log_level() -> LogEntryLevel {
    unsafe { MIN_LOG_LEVEL }
}

/// Returns a data time format string that should be used for logging. Will be either the default
/// string, or a custom one if it exists in the environment variable `STUMP_LOG_DATETIME_FORMAT`.
fn get_log_datetime_format_string() -> String {
    // If you don't like the default "STUMP_LOG_AT_LEVEL", set what you'd like in a build var
    // LOG_LEVEL_VAR_NAME
    let env_var_name = if let Some(v) = option_env!("LOG_DATETIME_FORMAT_VAR_NAME") {
        v.to_string()
    } else {
        "STUMP_LOG_DATETIME_FORMAT".to_string()
    };

    if let Ok(e) = env::var(env_var_name) {
        e
    } else {
        DEFAULT_DATETIME_PRINT_FORMAT.to_string()
    }
}

/// Formats the current date and time (now) using either the default data format or one specified in the
/// environment variable `STUMP_LOG_DATETIME_FORMAT`
pub fn format_datetime() -> String {
    format!(
        "{} ",
        Local::now().format(&get_log_datetime_format_string())
    )
}

/// Print to stdout if user specified increased output verbosity
#[macro_export]
macro_rules! vprintln {
    () => (if $crate::print::is_verbose() { std::print!("\n"); });
    ($($arg:tt)*) => {
        if $crate::is_verbose() {
            $crate::do_println(&format!("{} {}:{} {}", $crate::format_datetime(), file!(), line!(), format!($($arg)*)));
        }
    };
}

/// Print to stderr if user specified increased output verbosity
#[macro_export]
macro_rules! veprintln {
    () => (if $crate::is_verbose() { std::eprint!("\n"); });
    ($($arg:tt)*) => {
        if $crate::is_verbose() {
            eprintln!("{} {}:{} {}", $crate::format_datetime(), file!(), line!(), format!($($arg)*));
        }
    };
}

/// Generic status print, expects a LogEntryLevel as the first parameter.
#[macro_export]
macro_rules! status {
    ($level:expr, $($arg:tt)*) => {
        println!("{} {:?} {}:{} {}", $crate::format_datetime(), $level, file!(), line!(), format!($($arg)*));
    };
}

/// Prints a status message if the minimum status level is at or above the supplied LogEntryLevel enum (first parameter)
#[macro_export]
macro_rules! status_at_or_above {
    ($level:expr, $($arg:tt)*) => {
        if let Ok(min_log_level) = $crate::LogEntryLevel::from_env() {
            if min_log_level >= $level {
                status!($level, $($arg)*);
            }
        }
    };
}

/// Prints messages at the DEBUG (lowest) level. Accepts standard `println` formatting.
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        status_at_or_above!($crate::LogEntryLevel::DEBUG, $($arg)*);
    };
}

/// Prints messages at the INFO level. Accepts standard `println` formatting.
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        status_at_or_above!($crate::LogEntryLevel::INFO, $($arg)*);
    };
}

/// Prints messages at the WARN level. Accepts standard `println` formatting.
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        status_at_or_above!($crate::LogEntryLevel::WARN, $($arg)*);
    };
}

/// Prints messages at the ERROR level. Accepts standard `println` formatting.
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        status_at_or_above!($crate::LogEntryLevel::ERROR, $($arg)*);
    };
}

/// Outputs for task completion
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CompleteStatus {
    OK,
    WARN,
    FAIL,
}

/// Prints a message and an OK similar to older Linux boot displays
///
/// # Example:
/// ```
/// print_done("Completed task #1");
/// // Completed task #1                                                               [ DONE ]
/// ```
pub fn print_done(file_base_name: &str) {
    do_println(&format_complete(file_base_name, CompleteStatus::OK));
}

/// Formats a message and an OK similar to older Linux boot displays
///
/// # Example:
/// ```
/// let m = stump::format_done("Completed task #1");
/// assert_eq!(m, "Completed task #1                                                               [ DONE ]");
/// ```
pub fn format_done(file_base_name: &str) -> String {
    format_complete(file_base_name, CompleteStatus::OK)
}

/// Prints a message and a WARN similar to older Linux boot displays
///
/// # Example:
/// ```
/// format_done("Completed task #1 with warnings");
/// // Completed task #1 with warnings   [ WARN ]
/// ```
pub fn print_warn(file_base_name: &str) {
    do_println(&format_complete(file_base_name, CompleteStatus::WARN));
}

/// Formats a message and a WARN similar to older Linux boot displays
///
/// # Example:
/// ```
/// let m = stump::format_warn("Completed task #1 with warnings");
/// assert_eq!(m, "Completed task #1 with warnings                                                 [ WARN ]");
/// ```
pub fn format_warn(file_base_name: &str) -> String {
    format_complete(file_base_name, CompleteStatus::WARN)
}

/// Prints a message and a FAIL similar to older Linux boot displays
///
/// # Example:
/// ```
/// print_done("Completed task #1 with errors");
/// // Completed task #1 with errors                                                 [ DONE ]
/// ```
pub fn print_fail(file_base_name: &str) {
    do_println(&format_complete(file_base_name, CompleteStatus::FAIL));
}

/// Formats a message and a FAIL similar to older Linux boot displays
///
/// # Example:
/// ```
/// let m = stump::format_fail("Completed task #1 with errors");
/// assert_eq!(m, "Completed task #1 with errors                                                   [ FAIL ]");
/// ```
pub fn format_fail(file_base_name: &str) -> String {
    format_complete(file_base_name, CompleteStatus::FAIL)
}

/// Directly prints a completion message, taking in the CompleteStatus enum
pub fn print_complete(file_base_name: &String, status: CompleteStatus) {
    do_println(&format_complete(file_base_name, status));
}

/// Formats the completion message
fn format_complete(file_base_name: &str, status: CompleteStatus) -> String {
    let mut width = 88;

    if let Some(size) = termsize::get() {
        if size.cols < width {
            width = size.cols;
        }
    };

    let mut formatted = format!("{: <80}", file_base_name);
    if formatted.len() > width as usize - 8 {
        formatted = String::from(&formatted[0..(width as usize - 8)]);
    }

    format!(
        "{}[ {} ]",
        formatted,
        match status {
            CompleteStatus::OK => "DONE".green(),
            CompleteStatus::WARN => "WARN".yellow(),
            CompleteStatus::FAIL => "FAIL".red(),
        }
    )
}

/// Prints a simple message indicating experimental status of a function.
///
/// # Example
/// ```
/// print_experimental()
/// // Experimental Code! - Results may vary, bugs will be present, and not all functionality has been implemented
/// ```
pub fn print_experimental() {
    do_println(&format!("{} - Results may vary, bugs will be present, and not all functionality has been implemented", "Experimental Code!".red()));
}
