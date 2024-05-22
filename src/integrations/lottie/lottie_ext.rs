use bevy::utils::hashbrown::HashSet;
use velato::Composition;

/// Extension methods used for debugging.
pub trait LottieExt {
    fn get_layers(&self) -> impl Iterator<Item = &str>;
}

impl LottieExt for &Composition {
    fn get_layers(&self) -> impl Iterator<Item = &str> {
        self.layers
            .iter()
            .fold(HashSet::new(), move |mut acc, l| {
                acc.insert(l.name.as_str());
                acc
            })
            .into_iter()
    }
}
