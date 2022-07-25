use derive_more::Display;
use strum_macros::{EnumIter, EnumString};
use yew::prelude::*;
use LogLevel::*;

use crate::Platform::{self, *};

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, PartialOrd, Hash, EnumString, EnumIter)]
#[strum(ascii_case_insensitive)]
pub enum LogLevel {
    /// Desktop
    Trace,
    /// Android, iOS
    // `serialize = "verbose"` is included for parsing of `<select>`'s option.
    #[strum(serialize = "V", serialize = "💙", serialize = "verbose")]
    Verbose,
    /// Android, iOS, Desktop
    #[strum(serialize = "D", serialize = "💚", serialize = "debug")]
    Debug,
    /// Android, iOS, Desktop
    #[strum(serialize = "I", serialize = "💛", serialize = "info")]
    Info,
    /// Android, iOS, Desktop
    #[strum(serialize = "W", serialize = "🧡", serialize = "warn")]
    Warn,
    /// Android, iOS, Desktop
    #[strum(serialize = "E", serialize = "❤️", serialize = "error")]
    Error,
    /// Android, Desktop
    #[strum(serialize = "F", serialize = "fatal")]
    Fatal,
}

impl Default for LogLevel {
    fn default() -> Self {
        Info
    }
}

impl LogLevel {
    pub fn applicable_to_platform(&self, platform: Platform) -> bool {
        matches!(
            (self, platform),
            (Trace, Desktop)
                | (Verbose, Android | Ios)
                | (Debug | Info | Warn | Error, Android | Ios | Desktop)
                | (Fatal, Android | Desktop)
        )
    }

    pub fn color(&self) -> Classes {
        match self {
            Trace | Verbose => classes!("text-green-600", "dark:text-green-400"),
            Debug => classes!("text-blue-600", "dark:text-blue-400"),
            Info => classes!(),
            Warn => classes!("text-yellow-600", "dark:text-yellow-400"),
            Error => classes!("text-rose-600", "dark:text-rose-400"),
            Fatal => classes!("text-fuchsia-600", "dark:text-fuchsia-400"),
        }
    }
}
