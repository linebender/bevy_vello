use crate::{VelloFont, prelude::*};
use bevy::{
    asset::AssetEvent,
    camera::primitives::Aabb,
    prelude::*,
    ui::{ContentSize, NodeMeasure},
};

fn helper_calculate_aabb(
    font: &VelloFont,
    text: &VelloText2d,
    text_anchor: &VelloTextAnchor,
) -> Aabb {
    let layout = font.layout(&text.value, &text.style, text.text_align, text.max_advance);
    let (width, height) = (layout.width(), layout.height());
    let half_size = Vec3::new(width / 2.0, height / 2.0, 0.0);
    let (dx, dy) = {
        match text_anchor {
            VelloTextAnchor::TopLeft => (half_size.x, -half_size.y),
            VelloTextAnchor::Left => (half_size.x, 0.0),
            VelloTextAnchor::BottomLeft => (half_size.x, half_size.y),
            VelloTextAnchor::Top => (0.0, -half_size.y),
            VelloTextAnchor::Center => (0.0, 0.0),
            VelloTextAnchor::Bottom => (0.0, half_size.y),
            VelloTextAnchor::TopRight => (-half_size.x, -half_size.y),
            VelloTextAnchor::Right => (-half_size.x, 0.0),
            VelloTextAnchor::BottomRight => (-half_size.x, half_size.y),
        }
    };
    let adjustment = Vec3::new(dx, dy, 0.0);
    let min = -half_size + adjustment;
    let max = half_size + adjustment;
    Aabb::from_min_max(min, max)
}

pub fn update_text_2d_aabb_on_asset_load(
    mut asset_events: MessageReader<AssetEvent<VelloFont>>,
    mut world_texts: Query<(&mut Aabb, &VelloText2d, &VelloTextAnchor)>,
    fonts: Res<Assets<VelloFont>>,
) {
    for event in asset_events.read() {
        let id = if let AssetEvent::LoadedWithDependencies { id } = event {
            *id
        } else {
            continue;
        };
        let Some((mut aabb, text, text_anchor)) = world_texts
            .iter_mut()
            .find(|(_, text, _)| text.style.font.id() == id)
        else {
            continue;
        };
        let Some(font) = fonts.get(&text.style.font) else {
            // Not yet loaded
            continue;
        };
        let new_aabb = helper_calculate_aabb(font, text, text_anchor);
        *aabb = new_aabb;
    }
}

pub fn update_text_2d_aabb_on_change(
    mut world_texts: Query<
        (&mut Aabb, &VelloText2d, &VelloTextAnchor),
        Or<(Changed<VelloText2d>, Changed<Transform>)>,
    >,
    fonts: Res<Assets<VelloFont>>,
) {
    for (mut aabb, text, text_anchor) in world_texts.iter_mut() {
        let Some(font) = fonts.get(&text.style.font) else {
            // Not yet loaded
            continue;
        };
        let new_aabb = helper_calculate_aabb(font, text, text_anchor);
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
