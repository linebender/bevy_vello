//! Custom events for lottie player transitions.

use bevy::prelude::*;

/// Emitted when a lottie is shown. Used in player transitions.
#[derive(EntityEvent)]
pub struct LottieOnShowEvent {
    /// The entity this lottie event happened for.
    pub entity: Entity,
    /// The next state to transition to.
    pub next_state: &'static str,
}

/// Emitted when a lottie has played for some required time. Used in player transitions.
#[derive(EntityEvent)]
pub struct LottieOnAfterEvent {
    /// The entity this lottie event happened for.
    pub entity: Entity,
    /// The next state to transition to.
    pub next_state: &'static str,
}

/// Emitted when a lottie fully plays to the end. Used in player transitions.
#[derive(EntityEvent)]
pub struct LottieOnCompletedEvent {
    /// The entity this lottie event happened for.
    pub entity: Entity,
    /// The next state to transition to.
    pub next_state: &'static str,
}
