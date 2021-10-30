use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Error {
    ConfigError(String),
    IoError(String),
    ParseError(String),
    PathError(String),
    SyncingError(String),
}

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
