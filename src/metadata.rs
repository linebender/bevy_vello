use std::sync::Arc;
use vellottie::Composition;

/// Metadata used for introspection and color swapping.
pub struct Metadata {
    pub(crate) composition: Arc<Composition>,
}

impl Metadata {
    pub fn get_layers(&self) -> Vec<String> {
        self.composition
            .layers
            .iter()
            .map(|l| l.name.clone())
            .collect()
    }
}
