use thiserror::{self, Error};

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum VectorLoaderError {
    #[error("Could not load file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Could not parse utf-8: {0}")]
    FromStrUtf8(#[from] std::str::Utf8Error),
    #[cfg(feature = "svg")]
    #[error("Could not lower svg through imaging: {0}")]
    SvgImaging(#[from] svg_imaging::Error),
    #[cfg(feature = "lottie")]
    #[error("Could not parse lottie: {0}")]
    Velato(#[from] velato::Error),
}
