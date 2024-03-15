//! Augmenting and controls for lottie playback behavior.

mod alpha_override;
pub use alpha_override::PlaybackAlphaOverride;

mod playback_options;
pub use playback_options::{
    PlaybackDirection, PlaybackLoopBehavior, PlaybackOptions, PlaybackPlayMode,
};

mod playhead;
pub use playhead::Playhead;
