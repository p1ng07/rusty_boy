use std::{error, fmt::{self, Display}};

#[derive(Debug)]
pub struct UnableToOpenSelectedFileError {
    v: String,
}

impl UnableToOpenSelectedFileError {
    fn new() -> UnableToOpenSelectedFileError {
        UnableToOpenSelectedFileError {
            v: "Selected file could not be opened.".to_string()
        }
    }

    fn change_message(&mut self, new_message: &str) {
        self.v = new_message.to_string();
    }
}

impl error::Error for UnableToOpenSelectedFileError {}

impl Display for UnableToOpenSelectedFileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MyError: {}", &self.v)
    }
}
