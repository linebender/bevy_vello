use bevy::{prelude::*, utils::HashMap};
use vellottie::{
    runtime::model::{Brush, Shape},
    Composition,
};

#[derive(PartialEq, Component, Default, Clone, Debug, Reflect)]
#[reflect(Component)]
/// Add this component to a `VelloAssetBundle` entity to enable runtime color editing.
/// This interface allows swapping colors in a lottie composition by selecting the desired layer
/// and shape and overriding the original color with a new color.
///
/// Only works for layer shapes with fill or stroke elements.
pub struct Theme {
    pub(crate) colors: HashMap<String, Color>,
}

impl Theme {
    pub fn new() -> Self {
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

    pub fn get(&self, layer_name: &str) -> Option<&Color> {
        self.colors.get(layer_name)
    }

    pub fn get_mut(&mut self, layer_name: &str) -> Option<&mut Color> {
        self.colors.get_mut(layer_name)
    }
}

impl Theme {
    pub fn recolor(&self, composition: &Composition) -> Composition {
        let mut composition = composition.clone();
        'layers: for layer in composition.layers.iter_mut() {
            // Continue if this layer doesn't have a color swap
            let Some(target_color) = self.colors.get(&layer.name) else {
                continue 'layers;
            };
            let shapes = match &mut layer.content {
                vellottie::runtime::model::Content::Shape(shapes) => shapes,
                vellottie::runtime::model::Content::None
                | vellottie::runtime::model::Content::Instance { .. } => {
                    continue 'layers;
                }
            };
            let target_color = vello::peniko::Color::rgba(
                target_color.r().into(),
                target_color.g().into(),
                target_color.b().into(),
                target_color.a().into(),
            );
            for shape in shapes.iter_mut() {
                recolor_shape(shape, target_color);
            }
        }
        composition
    }
}

/// A helper method to recolor a shape with a target color.
fn recolor_shape(shape: &mut Shape, target_color: vello::peniko::Color) {
    match shape {
        vellottie::runtime::model::Shape::Group(shapes, _) => {
            for shape in shapes.iter_mut() {
                recolor_shape(shape, target_color);
            }
        }
        vellottie::runtime::model::Shape::Draw(draw) => {
            recolor_brush(&mut draw.brush, target_color);
        }
        vellottie::runtime::model::Shape::Repeater(_)
        | vellottie::runtime::model::Shape::Geometry(_) => {}
    }
}

/// A helper method to recolor a brush with a target color.
fn recolor_brush(brush: &mut Brush, target_color: vello::peniko::Color) {
    match brush {
        vellottie::runtime::model::Brush::Fixed(brush) => match brush {
            vello::peniko::Brush::Solid(solid) => {
                *solid = target_color;
            }
            vello::peniko::Brush::Gradient(gradient) => {
                for stop in gradient.stops.iter_mut() {
                    stop.color = target_color;
                }
            }
            vello::peniko::Brush::Image(_) => {}
        },
        vellottie::runtime::model::Brush::Animated(brush) => match brush {
            vellottie::runtime::model::animated::Brush::Solid(brush) => match brush {
                vellottie::runtime::model::Value::Fixed(solid) => {
                    *solid = target_color;
                }
                vellottie::runtime::model::Value::Animated(keyframes) => {
                    for solid in keyframes.values.iter_mut() {
                        *solid = target_color;
                    }
                }
            },
            vellottie::runtime::model::animated::Brush::Gradient(gr) => match &mut gr.stops {
                vellottie::runtime::model::ColorStops::Fixed(stops) => {
                    for stop in stops.iter_mut() {
                        stop.color = target_color;
                    }
                }
                vellottie::runtime::model::ColorStops::Animated(stops) => {
                    for _ in 0..stops.count {
                        for stop in stops.values.iter_mut() {
                            let _offset = stop[0];
                            let r = &mut stop[1];
                            *r = target_color.r as f32;
                            let g = &mut stop[2];
                            *g = target_color.g as f32;
                            let b = &mut stop[3];
                            *b = target_color.b as f32;
                            let a = &mut stop[4];
                            *a = target_color.a as f32;
                        }
                    }
                }
            },
        },
    }
}
