use bevy::{
    camera::primitives::Aabb,
    prelude::*,
    ui::{ContentSize, NodeMeasure},
};
use tracing::warn;

use crate::{VelloFont, prelude::*};

pub fn update_text_2d_aabb_on_change(
    mut text_q: Query<
        (&mut Aabb, &mut VelloText2d, &VelloTextAnchor),
        Or<(Changed<VelloText2d>, Changed<Transform>)>,
    >,
    fonts: Res<Assets<VelloFont>>,
) {
    for (mut aabb, text, text_anchor) in text_q.iter_mut() {
        let Some(font) = fonts.get(&text.style.font) else {
            warn!("VelloText2d: font {:?} not found", text.style.font);
            continue;
        };

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
        *aabb = Aabb::from_min_max(min, max);
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
            warn!("UiVelloText: font {:?} not found", text.style.font);
            continue;
        };

        let layout = font.layout(&text.value, &text.style, text.text_align, text.max_advance);
        let size = Vec2::new(layout.width(), layout.height()) / node.inverse_scale_factor();
        let measure = NodeMeasure::Fixed(bevy::ui::FixedMeasure { size });
        content_size.set(measure);
    }
}
