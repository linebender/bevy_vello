//! Integrations for supported file types. Some are included by cargo feature.
//!
//! # Cargo features
//! - `svg` - Enables SVG loading and rendering
//! - `lottie` - Enable Lottie (JSON) loading and rendering
//! - `text` - Enable text loading and rendering

pub mod scene;

#[cfg(feature = "svg")]
pub mod svg;

#[cfg(feature = "lottie")]
pub mod lottie;

#[cfg(feature = "text")]
pub mod text;

mod error;
pub use error::VectorLoaderError;
