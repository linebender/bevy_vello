use crate::prelude::*;
use bevy::{
    asset::AssetEvent,
    camera::primitives::Aabb,
    prelude::*,
    ui::{ContentSize, NodeMeasure},
};

pub fn update_text_2d_aabb_on_asset_load(
    mut asset_events: MessageReader<AssetEvent<VelloFont>>,
    mut world_texts: Query<(&mut Aabb, &VelloText2d, &VelloAnchor)>,
    fonts: Res<Assets<VelloFont>>,
) {
    for event in asset_events.read() {
        let id = if let AssetEvent::LoadedWithDependencies { id } = event {
            *id
        } else {
            continue;
        };
        let Some(font) = fonts.get(id) else {
            // Not yet loaded
            continue;
        };
        for (mut aabb, text, text_anchor) in world_texts
            .iter_mut()
            .filter(|(_, text, _)| text.style.font.id() == id)
        {
            let new_aabb = text_anchor.to_aabb_from_dimensions(
                font.layout(&text.value, &text.style, text.text_align, text.max_advance)
                    .width(),
                font.layout(&text.value, &text.style, text.text_align, text.max_advance)
                    .height(),
            );
            *aabb = new_aabb;
        }
    }
}

pub fn update_text_2d_aabb_on_change(
    mut world_texts: Query<
        (&mut Aabb, &VelloText2d, &VelloAnchor),
        Or<(Changed<VelloText2d>, Changed<Transform>)>,
    >,
    fonts: Res<Assets<VelloFont>>,
) {
    for (mut aabb, text, text_anchor) in world_texts.iter_mut() {
        let Some(font) = fonts.get(&text.style.font) else {
            // Not yet loaded
            continue;
        };
        let new_aabb = text_anchor.to_aabb_from_dimensions(
            font.layout(&text.value, &text.style, text.text_align, text.max_advance)
                .width(),
            font.layout(&text.value, &text.style, text.text_align, text.max_advance)
                .height(),
        );
        *aabb = new_aabb;
    }
}

pub fn update_ui_text_content_size_on_change(
    mut text_q: Query<
        (&mut ContentSize, &ComputedNode, &mut UiVelloText),
        Or<(Changed<UiVelloText>, Changed<ComputedNode>)>,
    >,
    fonts: Res<Assets<VelloFont>>,
) {
    for (mut content_size, node, text) in text_q.iter_mut() {
        let Some(font) = fonts.get(&text.style.font) else {
            // Not yet loaded
            continue;
        };

        let layout = font.layout(&text.value, &text.style, text.text_align, text.max_advance);
        let size = Vec2::new(layout.width(), layout.height()) / node.inverse_scale_factor();
        let measure = NodeMeasure::Fixed(bevy::ui::FixedMeasure { size });
        content_size.set(measure);
    }
}
