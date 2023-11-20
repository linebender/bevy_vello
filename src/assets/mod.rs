mod asset_loader;
mod parser;

pub(crate) mod vector;

pub use asset_loader::VelloVectorLoader;
pub use vector::Vector;
pub use vector::VelloVector;

pub(crate) use asset_loader::VectorLoaderError;
pub use parser::*;
