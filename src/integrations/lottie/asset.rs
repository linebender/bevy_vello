use std::sync::Arc;

use bevy::{prelude::*, reflect::TypePath};

#[derive(Asset, TypePath, Clone)]
pub struct VelloLottie {
    pub composition: Arc<velato::Composition>,
    pub alpha: f32,
}
