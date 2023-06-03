use anyhow::{anyhow, Result};
use chrono::prelude::*;
use colored::*;
use std::env;
use termsize;

#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub enum LogEntryLevel {
    DEBUG = 0x0,
    INFO = 0x1,
    WARN = 0x2,
    ERROR = 0x3,
}

const DEFAULT_LOG_AT_LEVEL: LogEntryLevel = LogEntryLevel::INFO;
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
            Ok(DEFAULT_LOG_AT_LEVEL)
        }
    }
}

static mut IS_VERBOSE: bool = false;

type FnPrint = dyn Fn(&String) + Send + Sync + 'static;
static mut PRINT: Option<Box<FnPrint>> = None;

/// Provides an alternative print method for integration with other command line
/// options such as `informif`.
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
pub fn set_verbose(v: bool) {
    unsafe {
        IS_VERBOSE = v;
    }
}

/// Indicates whether the verbose flag is set
pub fn is_verbose() -> bool {
    unsafe { IS_VERBOSE }
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
            $crate::print::do_println(&format!("{} {}:{} {}", $crate::format_datetime(), file!(), line!(), format!($($arg)*)));
        }
    };
}

/// Print to stderr if user specified increased output verbosity
#[macro_export]
macro_rules! veprintln {
    () => (if $crate::is_verbose() { std::eprint!("\n"); });
    ($($arg:tt)*) => {
        if $crate::print::is_verbose() {
            eprintln!("{} {}:{} {}", $crate::format_datetime(), file!(), line!(), format!($($arg)*));
        }
    };
}

#[macro_export]
macro_rules! status {
    ($level:expr, $($arg:tt)*) => {
        println!("{} {:?} {}:{} {}", $crate::format_datetime(), $level, file!(), line!(), format!($($arg)*));
    };
}

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

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        status_at_or_above!($crate::LogEntryLevel::DEBUG, $($arg)*);
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        status_at_or_above!($crate::LogEntryLevel::INFO, $($arg)*);
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        status_at_or_above!($crate::LogEntryLevel::WARN, $($arg)*);
    };
}

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

pub fn print_done(file_base_name: &String) {
    do_println(&format_complete(file_base_name, CompleteStatus::OK));
}

pub fn format_done(file_base_name: &String) -> String {
    format_complete(file_base_name, CompleteStatus::OK)
}

pub fn print_warn(file_base_name: &String) {
    do_println(&format_complete(file_base_name, CompleteStatus::WARN));
}

pub fn format_warn(file_base_name: &String) -> String {
    format_complete(file_base_name, CompleteStatus::WARN)
}

pub fn print_fail(file_base_name: &String) {
    do_println(&format_complete(file_base_name, CompleteStatus::FAIL));
}

pub fn format_fail(file_base_name: &String) -> String {
    format_complete(file_base_name, CompleteStatus::FAIL)
}

pub fn print_complete(file_base_name: &String, status: CompleteStatus) {
    do_println(&format_complete(file_base_name, status));
}

pub fn format_complete(file_base_name: &String, status: CompleteStatus) -> String {
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

pub fn print_experimental() {
    do_println(&format!("{} - Results may vary, bugs will be present, and not all functionality has been implemented", "Experimental Code!".red()));
}
