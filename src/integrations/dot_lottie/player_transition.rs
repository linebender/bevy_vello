#[derive(Debug, Clone)]
#[allow(clippy::enum_variant_names)]
pub enum PlayerTransition {
    /// Transitions to the given state after a period of seconds.
    OnAfter { state: &'static str, secs: f32 },
    /// Transition to the given state after the animation finishes.
    OnComplete { state: &'static str },
    /// Transition to the given state when the mouse enters the image bounding box.
    OnMouseEnter { state: &'static str },
    /// Transition to the given state when the mouse clicks inside the image bounding box.
    OnMouseClick { state: &'static str },
    /// Transition to the given state when the mouse exits the image bounding box.
    OnMouseLeave { state: &'static str },
    /// Transition to the given state on first render of this state.
    OnShow { state: &'static str },
}
