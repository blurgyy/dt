use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Error {
    /// Errors that occur when a config is deemed as invalid.
    ConfigError(String),
    /// Errors that may occur during I/O operations.
    IoError(String),
    /// Errors that occur when parsing of structures failes.
    ParseError(String),
    /// Errors that may occur while manipulating paths.
    PathError(String),
    /// Errors that may occur during syncing.
    SyncingError(String),
}

/// Result type used across the application.
pub type Result<T> = std::result::Result<T, Error>;

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::ConfigError(ref msg) => {
                write!(f, "Config Error: {}", msg)
            }
            Error::IoError(ref msg) => {
                write!(f, "IO Error: {}", msg)
            }
            Error::ParseError(ref msg) => {
                write!(f, "Parse Error: {}", msg)
            }
            Error::PathError(ref msg) => {
                write!(f, "Path Error: {}", msg)
            }
            Error::SyncingError(ref msg) => {
                write!(f, "Syncing Error: {}", msg)
            }
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err.to_string())
    }
}
impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self {
        Self::ParseError(err.to_string())
    }
}
impl From<std::path::StripPrefixError> for Error {
    fn from(err: std::path::StripPrefixError) -> Self {
        Self::PathError(err.to_string())
    }
}
impl From<glob::PatternError> for Error {
    fn from(err: glob::PatternError) -> Self {
        Self::PathError(err.to_string())
    }
}

// Author: Blurgy <gy@blurgy.xyz>
// Date:   Oct 29 2021, 23:07 [CST]
