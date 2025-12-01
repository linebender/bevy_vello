use bevy::prelude::*;
use vello::kurbo::Affine;

#[derive(Component, Copy, Clone, Deref, DerefMut)]
pub struct PreparedAffine(pub Affine);
