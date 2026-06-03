use chrono::Local;
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
}

impl LogLevel {
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "DEBUG" => Self::Debug,
            "WARN" => Self::Warn,
            "ERROR" => Self::Error,
            _ => Self::Info,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
        }
    }
}

static CURRENT_LEVEL: Mutex<LogLevel> = Mutex::new(LogLevel::Info);

pub fn set_log_level(level: LogLevel) {
    if let Ok(mut l) = CURRENT_LEVEL.lock() {
        *l = level;
    }
}

pub fn log(level: LogLevel, message: String) {
    let current = if let Ok(l) = CURRENT_LEVEL.lock() {
        *l
    } else {
        LogLevel::Info
    };

    if level >= current {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        let level_str = level.as_str();

        let color_code = match level {
            LogLevel::Debug => "\x1b[36m", // Cyan
            LogLevel::Info => "\x1b[32m",  // Green
            LogLevel::Warn => "\x1b[33m",  // Yellow
            LogLevel::Error => "\x1b[31m", // Red
        };
        let reset_code = "\x1b[0m";

        println!(
            "[{}]{}{:<5}{} - {}",
            timestamp, color_code, level_str, reset_code, message
        );
    }
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        $crate::logger::log($crate::logger::LogLevel::Debug, format!($($arg)*));
    }
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::logger::log($crate::logger::LogLevel::Info, format!($($arg)*));
    }
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        $crate::logger::log($crate::logger::LogLevel::Warn, format!($($arg)*));
    }
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::logger::log($crate::logger::LogLevel::Error, format!($($arg)*));
    }
}
