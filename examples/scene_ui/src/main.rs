use std::{
    f64::consts::{FRAC_PI_4, SQRT_2},
    ops::DerefMut,
};

use bevy::{color::palettes::css, prelude::*};
use bevy_vello::{VelloPlugin, prelude::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(VelloPlugin::default())
        .add_systems(Startup, setup_ui)
        .add_systems(Update, update_ui)
        .run();
}

fn setup_ui(mut commands: Commands) {
    commands.spawn((Camera2d, VelloView));

    let one_third = Val::Percent(100.0 / 3.0);
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: one_third,
            top: one_third,
            width: one_third,
            height: one_third,
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BorderColor(css::FUCHSIA.with_alpha(0.5).into()),
        Interaction::default(),
        VelloScene::new(),
    ));
}

fn update_ui(mut query: Single<(&ComputedNode, &Interaction, &mut VelloScene)>) {
    let (node, interaction, scene) = query.deref_mut();

    let size = node.size();
    let dmin = f32::min(size.x, size.y);
    let radius = (dmin / 2.0) as f64;

    let center = size / 2.0;
    let center = kurbo::Point::from((center.x as f64, center.y as f64));

    scene.reset();

    match *interaction {
        Interaction::Hovered | Interaction::Pressed => {
            let color = match *interaction {
                Interaction::Hovered => peniko::Color::from_rgba8(0, 255, 0, 255),
                Interaction::Pressed => peniko::Color::from_rgba8(0, 110, 0, 255),
                _ => unreachable!(),
            };

            scene.fill(
                peniko::Fill::NonZero,
                kurbo::Affine::default(),
                color,
                None,
                &kurbo::Circle::new(center, radius),
            );
            scene.fill(
                peniko::Fill::NonZero,
                kurbo::Affine::rotate_about(FRAC_PI_4, center)
                    .then_translate(kurbo::Vec2::new(SQRT_2 / 2.0, SQRT_2 / 2.0) * radius * 0.2)
                    .then_translate(kurbo::Vec2::new(0.0, radius * -0.1)),
                peniko::Color::WHITE,
                None,
                &kurbo::Rect::from_center_size(center, (radius * 0.2, radius)),
            );
            scene.fill(
                peniko::Fill::NonZero,
                kurbo::Affine::rotate_about(-FRAC_PI_4, center)
                    .then_translate(kurbo::Vec2::new(SQRT_2 / -2.0, SQRT_2 / -2.0) * radius * 0.1)
                    .then_translate(kurbo::Vec2::new(SQRT_2 / -2.0, SQRT_2 / 2.0) * radius * 0.4)
                    .then_translate(kurbo::Vec2::new(0.0, radius * -0.1)),
                peniko::Color::WHITE,
                None,
                &kurbo::Rect::from_center_size(center, (radius * 0.2, radius * 0.5)),
            );
        }
        Interaction::None => {
            scene.fill(
                peniko::Fill::NonZero,
                kurbo::Affine::default(),
                peniko::Color::from_rgba8(255, 0, 0, 255),
                None,
                &kurbo::Circle::new(center, radius),
            );
            scene.fill(
                peniko::Fill::NonZero,
                kurbo::Affine::rotate_about(FRAC_PI_4, center),
                peniko::Color::WHITE,
                None,
                &kurbo::Rect::from_center_size(center, (radius * 0.2, radius * 1.25)),
            );
            scene.fill(
                peniko::Fill::NonZero,
                kurbo::Affine::rotate_about(-FRAC_PI_4, center),
                peniko::Color::WHITE,
                None,
                &kurbo::Rect::from_center_size(center, (radius * 0.2, radius * 1.25)),
            );
        }
    }
}
