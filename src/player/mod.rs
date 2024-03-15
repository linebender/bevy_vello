//! Contains logic for the [`LottiePlayer`], a component used to control Lottie playback settings idiomatically with code.

mod lottie_player;
pub use lottie_player::LottiePlayer;

mod player_state;
pub use player_state::PlayerState;

mod player_transition;
pub use player_transition::PlayerTransition;

mod plugin;
pub use plugin::LottiePlayerPlugin;

mod systems;
