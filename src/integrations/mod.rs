//! Integrations for supported file types. These are included by cargo feature.
//!
//! # Features
//! - `svg` - Enables SVG loading and rendering
//! - `lottie` - Enable Lottie (JSON) loading and rendering

#[cfg(feature = "svg")]
pub mod svg;

#[cfg(feature = "lottie")]
pub mod lottie;

mod error;
pub use error::VectorLoaderError;
