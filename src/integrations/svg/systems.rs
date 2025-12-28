use crate::prelude::*;
use bevy::{
    camera::primitives::Aabb,
    prelude::*,
    ui::{ContentSize, NodeMeasure},
};

fn helper_calculate_aabb(svg: &VelloSvg, anchor: &VelloSvgAnchor) -> Aabb {
    let (width, height) = (svg.width, svg.height);
    let half_size = Vec3::new(width / 2.0, height / 2.0, 0.0);
    let (dx, dy) = {
        match anchor {
            VelloSvgAnchor::TopLeft => (half_size.x, -half_size.y),
            VelloSvgAnchor::Left => (half_size.x, 0.0),
            VelloSvgAnchor::BottomLeft => (half_size.x, half_size.y),
            VelloSvgAnchor::Top => (0.0, -half_size.y),
            VelloSvgAnchor::Center => (0.0, 0.0),
            VelloSvgAnchor::Bottom => (0.0, half_size.y),
            VelloSvgAnchor::TopRight => (-half_size.x, -half_size.y),
            VelloSvgAnchor::Right => (-half_size.x, 0.0),
            VelloSvgAnchor::BottomRight => (-half_size.x, half_size.y),
        }
    };
    let adjustment = Vec3::new(dx, dy, 0.0);
    let min = -half_size + adjustment;
    let max = half_size + adjustment;
    Aabb::from_min_max(min, max)
}

pub fn update_svg_2d_aabb_on_asset_load(
    mut asset_events: MessageReader<AssetEvent<VelloSvg>>,
    mut world_svgs: Query<(&mut Aabb, &VelloSvg2d, &VelloSvgAnchor)>,
    svgs: Res<Assets<VelloSvg>>,
) {
    for event in asset_events.read() {
        let id = if let AssetEvent::LoadedWithDependencies { id } = event {
            *id
        } else {
            continue;
        };
        let Some(svg) = svgs.get(id) else {
            // Not yet loaded
            continue;
        };
        for (mut aabb, _, anchor) in world_svgs.iter_mut().filter(|(_, svg, _)| svg.id() == id) {
            let new_aabb = helper_calculate_aabb(svg, anchor);
            *aabb = new_aabb;
        }
    }
}

pub fn update_svg_2d_aabb_on_change(
    mut world_svgs: Query<(&mut Aabb, &mut VelloSvg2d, &VelloSvgAnchor), Changed<VelloSvg2d>>,
    svgs: Res<Assets<VelloSvg>>,
) {
    for (mut aabb, svg, anchor) in world_svgs.iter_mut() {
        let Some(svg) = svgs.get(&svg.0) else {
            // Not yet loaded
            continue;
        };
        let new_aabb = helper_calculate_aabb(svg, anchor);
        *aabb = new_aabb;
    }
}

pub fn update_ui_svg_content_size_on_change(
    mut text_q: Query<
        (&mut ContentSize, &ComputedNode, &mut UiVelloSvg),
        Or<(Changed<UiVelloSvg>, Changed<ComputedNode>)>,
    >,
    svgs: Res<Assets<VelloSvg>>,
) {
    for (mut content_size, node, svg) in text_q.iter_mut() {
        let Some(svg) = svgs.get(&svg.0) else {
            // Not yet loaded
            continue;
        };

        let size = Vec2::new(svg.width, svg.height) / node.inverse_scale_factor();
        let measure = NodeMeasure::Fixed(bevy::ui::FixedMeasure { size });
        content_size.set(measure);
    }
}
