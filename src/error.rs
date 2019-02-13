use std::fmt;
use std::error;

#[derive(Debug, Clone)]
pub struct Error {
    message: String
}

pub fn error(message: &str) -> Error {
    Error { message: message.to_owned() }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.message)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        &self.message
    }
}
