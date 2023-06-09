use velato::Composition;

pub struct Metadata {
    pub(crate) composition: Composition,
}

impl Metadata {
    pub fn get_layers(&self) -> Vec<String> {
        self.composition
            .layers
            .iter()
            .map(|l| l.name.clone())
            .collect()
    }

    pub fn get_layer_shapes(&self, layer: &str) -> Option<usize> {
        match self.composition.layers.iter().find(|l| l.name.eq(&layer)) {
            Some(layer) => {
                let velato::model::Content::Shape(ref shapes) = layer.content else {
                    return None;
                };
                Some(shapes.iter().len())
            }
            None => None,
        }
    }
}
