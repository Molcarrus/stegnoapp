#[derive(Debug)]
pub enum Error {
    SecretReadError,
    SecretTooLarge,
    InvalidNumberOfBits,
    ImageReadWriteError
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Error::SecretReadError => write!(f, "Something when while reading secret file"),
            Error::SecretTooLarge => write!(f, "Secret is too large to fit in image"),
            Error::InvalidNumberOfBits => write!(f, "Only 1 to 8 LSB bits are allowed"),
            Error::ImageReadWriteError => write!(f, "Something went wrong while processing the image")
        }   
    } 
}

impl From<std::io::Error> for Error {
    fn from(_value: std::io::Error) -> Self {
        Error::SecretReadError
    }
}

impl From<image::ImageError> for Error {
    fn from(_value: image::ImageError) -> Self {
        Error::ImageReadWriteError
    }
}

