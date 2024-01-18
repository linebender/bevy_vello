use bevy::{prelude::*, utils::HashMap};

#[derive(PartialEq, Component, Default, Clone, Debug, Reflect)]
#[reflect(Component)]
/// Add this component to a `VelloVectorBundle` entity to enable runtime color editing.
/// This interface allows swapping colors in a lottie composition by selecting the desired layer
/// and shape and overriding the original color with a new color.
///
/// Only works for layer shapes with fill or stroke elements.
pub struct ColorPaletteSwap {
    pub(crate) colors: HashMap<String, Color>,
}

impl ColorPaletteSwap {
    pub fn empty() -> Self {
        Self {
            colors: HashMap::default(),
        }
    }

    /// Swap a color with the given layer name.
    pub fn add(mut self, layer_name: &str, color: Color) -> Self {
        self.colors.insert(layer_name.to_string(), color);
        self
    }

    /// Swap a color for the selected layer name. This will overwrite the previous value.
    pub fn edit(&mut self, layer_name: &str, color: Color) -> &mut Self {
        self.colors.insert(layer_name.to_string(), color);
        self
    }
}
