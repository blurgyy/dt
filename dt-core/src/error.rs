use std::fmt;

#[derive(Debug)]
pub enum Error {
    ParseError(String),
    IoError(std::io::Error),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::ParseError(ref msg) => {
                write!(f, "Parse Error: {}", msg)
            }
            Error::IoError(ref err) => {
                write!(f, "Io Error: {}", err)
            }
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err)
    }
}

// Author: Blurgy <gy@blurgy.xyz>
// Date:   Oct 29 2021, 23:07 [CST]
