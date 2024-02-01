mod asset_loader;
mod parser;

pub(crate) mod vector;

pub use asset_loader::VelloAssetLoader;
pub use vector::VelloAsset;
pub use vector::VelloAssetData;

pub(crate) use asset_loader::VectorLoaderError;
pub use parser::*;
