mod asset;
pub use asset::{VectorFile, VelloAsset};

mod parser;
pub use parser::{
    load_lottie_from_bytes, load_lottie_from_str, load_svg_from_bytes, load_svg_from_str,
};

mod asset_loader;
pub(crate) use asset_loader::{VectorLoaderError, VelloAssetLoader};

mod metadata;
pub use metadata::Metadata;
