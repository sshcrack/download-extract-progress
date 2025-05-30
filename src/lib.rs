mod download;
#[cfg(feature = "7z")]
mod extract_7z;
#[cfg(feature = "zip")]
mod extract_zip;

mod error;
mod github_releases;

pub use error::*;
pub use download::*;
#[cfg(feature = "7z")]
pub use extract_7z::*;

#[cfg(feature = "zip")]
pub use extract_zip::*;