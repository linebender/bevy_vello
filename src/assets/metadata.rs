use bevy::utils::hashbrown::HashSet;
use std::sync::Arc;
use velato::Composition;

/// Metadata used for introspection and color swapping.
pub struct Metadata {
    pub(crate) composition: Arc<Composition>,
}

impl Metadata {
    pub fn get_layers(&self) -> impl Iterator<Item = &str> {
        self.composition
            .layers
            .iter()
            .fold(HashSet::new(), move |mut acc, l| {
                acc.insert(l.name.as_str());
                acc
            })
            .into_iter()
    }
}
