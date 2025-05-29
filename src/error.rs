use std::fmt::Display;

use hex::FromHexError;


#[derive(Debug)]
pub enum DownloadError {
    IoError(std::io::Error),
    RequestError(reqwest::Error),
    /// An invalid hash was provided, which could not be decoded
    InvalidHash(FromHexError),
    /// Hash mismatch error, containing the expected and actual hash.
    HashMismatch(String, String)
}


impl Display for DownloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DownloadError::HashMismatch(expected, actual) => write!(f, "Hash mismatch: expected {}, got {}", expected, actual),
            DownloadError::IoError(error) => write!(f, "IO error: {}", error),
            DownloadError::RequestError(error) => write!(f, "Request error: {}", error),
            DownloadError::InvalidHash(e) => write!(f, "Invalid hash: {}", e),
        }
    }
}

impl std::error::Error for DownloadError {}