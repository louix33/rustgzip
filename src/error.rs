use std::fmt;

#[derive(Debug)]
pub struct DecodeError {
    message: String
}

#[derive(Debug)]
pub struct EncodeError {
    message: String
}

impl std::error::Error for DecodeError {}
impl std::error::Error for EncodeError {}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DecodeError: {}",
            self.message
        )
    }
}

impl fmt::Display for EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "EncodeError: {}",
            self.message
        )
    }
}

impl From<&str> for DecodeError {
    fn from(msg: &str) -> Self {
        DecodeError { message: String::from(msg) }
    }
}

impl From<&str> for EncodeError {
    fn from(msg: &str) -> Self {
        EncodeError { message: String::from(msg) }
    }
}
