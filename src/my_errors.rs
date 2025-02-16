use std::fmt;

#[derive(Debug)]
pub enum MyError {
    FileError(String)
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MyError::FileError(msg) => write!(f, "File Error: {}", msg),
        }
    }
}

