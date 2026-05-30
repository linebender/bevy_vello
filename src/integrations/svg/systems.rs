use crate::prelude::*;
use bevy::{
    camera::primitives::Aabb,
    prelude::*,
    ui::{ContentSize, NodeMeasure},
};

pub fn update_svg_2d_aabb_on_asset_load(
    mut asset_events: MessageReader<AssetEvent<VelloSvg>>,
    mut world_svgs: Query<(&mut Aabb, &VelloSvg2d, &VelloAnchor)>,
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
            let new_aabb = anchor.to_aabb_from_dimensions(svg.width, svg.height);
            *aabb = new_aabb;
        }
    }
}

pub fn update_svg_2d_aabb_on_change(
    mut world_svgs: Query<(&mut Aabb, &mut VelloSvg2d, &VelloAnchor), Changed<VelloSvg2d>>,
    svgs: Res<Assets<VelloSvg>>,
) {
    for (mut aabb, svg, anchor) in world_svgs.iter_mut() {
        let Some(svg) = svgs.get(&svg.0) else {
            // Not yet loaded
            continue;
        };
        let new_aabb = anchor.to_aabb_from_dimensions(svg.width, svg.height);
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
