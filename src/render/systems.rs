use bevy::{
    asset::RenderAssetUsages,
    camera::visibility::NoFrustumCulling,
    image::ToExtents,
    mesh::Indices,
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_resource::{
            Extent3d, PrimitiveTopology, TextureDescriptor, TextureDimension, TextureFormat,
            TextureUsages,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::GpuImage,
    },
    sprite_render::MeshMaterial2d,
    window::PrimaryWindow,
};
use vello::{RenderParams, Scene};

use super::{
    VelloCanvasMaterial, VelloCanvasSettings, VelloEntityCountData, VelloFrameProfileData,
    VelloRenderQueue, VelloRenderSettings, VelloRenderer, VelloWorldRenderItem,
    extract::VelloRenderTarget, prepare::PreparedAffine,
};
#[cfg(feature = "lottie")]
use crate::integrations::lottie::render::{ExtractedUiVelloLottie, ExtractedVelloLottie2d};
#[cfg(feature = "svg")]
use crate::integrations::svg::render::{ExtractedUiVelloSvg, ExtractedVelloSvg2d};
#[cfg(feature = "text")]
use crate::integrations::text::{
    VelloFont,
    render::{ExtractedUiVelloText, ExtractedVelloText2d},
};
use crate::{
    integrations::scene::render::{ExtractedUiVelloScene, ExtractedVelloScene2d},
    render::{VelloUiRenderItem, VelloView},
};

pub fn setup_image(images: &mut Assets<Image>, width: u32, height: u32) -> Handle<Image> {
    let size = Extent3d {
        width,
        height,
        ..default()
    };

    // This is the texture that will be rendered to.
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        },
        ..default()
    };

    // fill image.data with zeroes
    image.resize(size);
    images.add(image)
}

#[allow(clippy::too_many_arguments, reason = "Many features gates")]
pub fn sort_render_items(
    view_world_scenes: Query<(&PreparedAffine, &ExtractedVelloScene2d)>,
    view_ui_scenes: Query<(&PreparedAffine, &ExtractedUiVelloScene)>,
    #[cfg(feature = "text")] view_world_text: Query<(&PreparedAffine, &ExtractedVelloText2d)>,
    #[cfg(feature = "text")] view_ui_text: Query<(&PreparedAffine, &ExtractedUiVelloText)>,
    #[cfg(feature = "svg")] view_world_svgs: Query<(&PreparedAffine, &ExtractedVelloSvg2d)>,
    #[cfg(feature = "svg")] view_ui_svgs: Query<(&PreparedAffine, &ExtractedUiVelloSvg)>,
    #[cfg(feature = "lottie")] view_world_lotties: Query<(
        &PreparedAffine,
        &ExtractedVelloLottie2d,
    )>,
    #[cfg(feature = "lottie")] view_ui_lotties: Query<(&PreparedAffine, &ExtractedUiVelloLottie)>,
    mut final_render_queue: ResMut<VelloRenderQueue>,
    frame_data: ResMut<VelloEntityCountData>,
) {
    let mut n_world_items: usize = 0;
    let mut n_ui_items: usize = 0;

    // Scenes
    n_world_items += frame_data.n_world_scenes as usize;
    n_ui_items += frame_data.n_ui_scenes as usize;
    // Text
    #[cfg(feature = "text")]
    {
        n_world_items += frame_data.n_world_texts as usize;
        n_ui_items += frame_data.n_ui_texts as usize;
    }
    // Svg
    #[cfg(feature = "svg")]
    {
        n_world_items += frame_data.n_world_svgs as usize;
        n_ui_items += frame_data.n_ui_svgs as usize;
    }
    // Lottie
    #[cfg(feature = "lottie")]
    {
        n_world_items += frame_data.n_world_lotties as usize;
        n_ui_items += frame_data.n_ui_lotties as usize;
    }

    // Reserve space for the render queues to avoid reallocations
    let mut world_render_queue: Vec<(f32, VelloWorldRenderItem)> =
        Vec::with_capacity(n_world_items);
    let mut ui_render_queue: Vec<(u32, VelloUiRenderItem)> = Vec::with_capacity(n_world_items);

    // Scenes
    for (&affine, scene) in view_world_scenes.iter() {
        world_render_queue.push((
            scene.transform.translation().z,
            VelloWorldRenderItem::Scene {
                affine: *affine,
                item: scene.clone(),
            },
        ));
    }
    for (&affine, scene) in view_ui_scenes.iter() {
        ui_render_queue.push((
            scene.ui_node.stack_index,
            VelloUiRenderItem::Scene {
                affine: *affine,
                item: scene.clone(),
            },
        ));
    }

    #[cfg(feature = "svg")]
    {
        for (&affine, svg) in view_world_svgs.iter() {
            world_render_queue.push((
                svg.transform.translation().z,
                VelloWorldRenderItem::Svg {
                    affine: *affine,
                    item: svg.clone(),
                },
            ));
        }
        for (&affine, svg) in view_ui_svgs.iter() {
            ui_render_queue.push((
                svg.ui_node.stack_index,
                VelloUiRenderItem::Svg {
                    affine: *affine,
                    item: svg.clone(),
                },
            ));
        }
    }

    #[cfg(feature = "lottie")]
    {
        for (&affine, lottie) in view_world_lotties.iter() {
            world_render_queue.push((
                lottie.transform.translation().z,
                VelloWorldRenderItem::Lottie {
                    affine: *affine,
                    item: lottie.clone(),
                },
            ));
        }
        for (&affine, lottie) in view_ui_lotties.iter() {
            ui_render_queue.push((
                lottie.ui_node.stack_index,
                VelloUiRenderItem::Lottie {
                    affine: *affine,
                    item: lottie.clone(),
                },
            ));
        }
    }

    #[cfg(feature = "text")]
    {
        for (&affine, text) in view_world_text.iter() {
            world_render_queue.push((
                text.transform.translation().z,
                VelloWorldRenderItem::Text {
                    affine: *affine,
                    item: text.clone(),
                },
            ));
        }
        for (&affine, text) in view_ui_text.iter() {
            ui_render_queue.push((
                text.ui_node.stack_index,
                VelloUiRenderItem::Text {
                    affine: *affine,
                    item: text.clone(),
                },
            ));
        }
    }

    // Sort by render mode with screen space on top, then by z-index
    world_render_queue.sort_unstable_by(|(a_z_index, _), (b_z_index, _)| {
        a_z_index
            .partial_cmp(b_z_index)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    ui_render_queue.sort_unstable_by(|(a_stack_index, _), (b_stack_index, _)| {
        a_stack_index
            .partial_cmp(b_stack_index)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Render queue is drained on render
    final_render_queue.world.clear();
    // Reserve space for the final render queue to avoid reallocations
    final_render_queue.world.reserve(n_world_items);
    final_render_queue
        .world
        .extend(world_render_queue.into_iter().map(|(_, r)| r));

    // Same thing for UI
    final_render_queue.ui.clear();
    final_render_queue.ui.reserve(n_ui_items);
    final_render_queue
        .ui
        .extend(ui_render_queue.into_iter().map(|(_, r)| r));
}

/// Transforms all the vectors extracted from the game world and places them in
/// a scene, and renders the scene to a texture with WGPU
#[allow(clippy::complexity)]
pub fn render_frame(
    ss_render_target: Single<&VelloRenderTarget>,
    #[cfg(feature = "text")] font_render_assets: Res<RenderAssets<VelloFont>>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    renderer: Res<VelloRenderer>,
    #[cfg(feature = "lottie")] mut velato_renderer: ResMut<super::VelatoRenderer>,
    render_settings: Res<VelloRenderSettings>,
    render_queue: Res<VelloRenderQueue>,
    mut frame_profile: ResMut<VelloFrameProfileData>,
) {
    let VelloRenderTarget(render_target_image) = *ss_render_target;
    let gpu_image = gpu_images.get(render_target_image).unwrap();

    let mut scene_buffer = Scene::new();

    // World Renderables
    for render_item in render_queue.world.iter() {
        match render_item {
            VelloWorldRenderItem::Scene {
                affine,
                item: ExtractedVelloScene2d { scene, .. },
            } => {
                scene_buffer.append(scene, Some(*affine));
            }
            #[cfg(feature = "lottie")]
            VelloWorldRenderItem::Lottie {
                affine,
                item:
                    ExtractedVelloLottie2d {
                        asset,
                        alpha,
                        theme,
                        playhead,
                        ..
                    },
            } => {
                if *alpha <= 0.0 {
                    continue;
                }
                if *alpha < 1.0 {
                    scene_buffer.push_layer(
                        vello::peniko::Mix::Normal,
                        *alpha,
                        *affine,
                        &vello::kurbo::Rect::new(
                            0.0,
                            0.0,
                            asset.composition.width as f64,
                            asset.composition.height as f64,
                        ),
                    );
                }
                let recolored = theme.as_ref().map(|cs| cs.recolor(&asset.composition));
                let animation = recolored.as_ref().unwrap_or(&asset.composition);
                velato_renderer.append(
                    animation,
                    *playhead as f64,
                    *affine,
                    1.0,
                    &mut scene_buffer,
                );
                if *alpha < 1.0 {
                    scene_buffer.pop_layer();
                }
            }
            #[cfg(feature = "svg")]
            VelloWorldRenderItem::Svg {
                affine,
                item: ExtractedVelloSvg2d { asset, alpha, .. },
            } => {
                if *alpha <= 0.0 {
                    continue;
                }
                if *alpha < 1.0 {
                    scene_buffer.push_layer(
                        vello::peniko::Mix::Normal,
                        *alpha,
                        *affine,
                        &vello::kurbo::Rect::new(0.0, 0.0, asset.width as f64, asset.height as f64),
                    );
                }
                scene_buffer.append(&asset.scene, Some(*affine));
                if *alpha < 1.0 {
                    scene_buffer.pop_layer();
                }
            }
            #[cfg(feature = "text")]
            VelloWorldRenderItem::Text {
                affine,
                item:
                    ExtractedVelloText2d {
                        text, text_anchor, ..
                    },
            } => {
                if let Some(font) = font_render_assets.get(text.style.font.id()) {
                    font.render(
                        &mut scene_buffer,
                        *affine,
                        &text.value,
                        &text.style,
                        text.text_align,
                        text.max_advance,
                        *text_anchor,
                    );
                }
            }
        }
    }

    // Ui Renderables
    for render_item in render_queue.ui.iter() {
        match render_item {
            VelloUiRenderItem::Scene {
                affine,
                item: ExtractedUiVelloScene { scene, .. },
            } => {
                scene_buffer.append(scene, Some(*affine));
            }
            #[cfg(feature = "lottie")]
            VelloUiRenderItem::Lottie {
                affine,
                item:
                    ExtractedUiVelloLottie {
                        asset,
                        alpha,
                        theme,
                        playhead,
                        ..
                    },
            } => {
                if *alpha <= 0.0 {
                    continue;
                }
                if *alpha < 1.0 {
                    scene_buffer.push_layer(
                        vello::peniko::Mix::Normal,
                        *alpha,
                        *affine,
                        &vello::kurbo::Rect::new(
                            0.0,
                            0.0,
                            asset.composition.width as f64,
                            asset.composition.height as f64,
                        ),
                    );
                }
                let recolored = theme.as_ref().map(|cs| cs.recolor(&asset.composition));
                let animation = recolored.as_ref().unwrap_or(&asset.composition);
                velato_renderer.append(
                    animation,
                    *playhead as f64,
                    *affine,
                    1.0,
                    &mut scene_buffer,
                );
                if *alpha < 1.0 {
                    scene_buffer.pop_layer();
                }
            }
            #[cfg(feature = "svg")]
            VelloUiRenderItem::Svg {
                affine,
                item: ExtractedUiVelloSvg { asset, alpha, .. },
            } => {
                if *alpha <= 0.0 {
                    continue;
                }
                if *alpha < 1.0 {
                    scene_buffer.push_layer(
                        vello::peniko::Mix::Normal,
                        *alpha,
                        *affine,
                        &vello::kurbo::Rect::new(0.0, 0.0, asset.width as f64, asset.height as f64),
                    );
                }
                scene_buffer.append(&asset.scene, Some(*affine));
                if *alpha < 1.0 {
                    scene_buffer.pop_layer();
                }
            }
            #[cfg(feature = "text")]
            VelloUiRenderItem::Text {
                affine,
                item:
                    ExtractedUiVelloText {
                        text, text_anchor, ..
                    },
            } => {
                if let Some(font) = font_render_assets.get(text.style.font.id()) {
                    font.render(
                        &mut scene_buffer,
                        *affine,
                        &text.value,
                        &text.style,
                        text.text_align,
                        text.max_advance,
                        *text_anchor,
                    );
                }
            }
        }
    }

    frame_profile.n_paths = scene_buffer.encoding().n_paths;
    frame_profile.n_path_segs = scene_buffer.encoding().n_path_segments;
    frame_profile.n_clips = scene_buffer.encoding().n_clips;
    frame_profile.n_open_clips = scene_buffer.encoding().n_open_clips;

    renderer
        .lock()
        .unwrap()
        .render_to_texture(
            device.wgpu_device(),
            &queue,
            &scene_buffer,
            &gpu_image.texture_view,
            &RenderParams {
                base_color: vello::peniko::Color::TRANSPARENT,
                width: gpu_image.size.width,
                height: gpu_image.size.height,
                antialiasing_method: render_settings.antialiasing,
            },
        )
        .unwrap();
}

// Returns the width and height of the available viewport space;
// camera viewport size if present, otherwise default to window size
pub fn get_viewport_size(
    camera_query: Query<&Camera, With<VelloView>>,
    window: Option<Single<&Window, With<PrimaryWindow>>>,
) -> (u32, u32) {
    if let Ok(camera) = camera_query.single()
        && let Some(viewport) = &camera.viewport
    {
        return (viewport.physical_size.x, viewport.physical_size.y);
    }

    if let Some(window) = window.as_deref() {
        (
            window.resolution.physical_width(),
            window.resolution.physical_height(),
        )
    } else {
        tracing::error!(
            "bevy_vello does not see a window, and thus, cannot resize the render target"
        );
        (0, 0)
    }
}

pub fn resize_rendertargets(
    mut query: Query<(&mut VelloRenderTarget, &MeshMaterial2d<VelloCanvasMaterial>)>,
    mut images: ResMut<Assets<Image>>,
    mut target_materials: ResMut<Assets<VelloCanvasMaterial>>,
    window: Option<Single<&Window, With<PrimaryWindow>>>,
    camera_query: Query<&Camera, With<VelloView>>,
) {
    let (width, height) = get_viewport_size(camera_query, window);

    let size = Extent3d {
        width,
        height,
        ..default()
    };
    if size.width == 0 || size.height == 0 {
        return;
    }
    for (mut target, target_mat_handle) in query.iter_mut() {
        if let Some(image) = images.get(target.0.id())
            && image.size().to_extents() == size
        {
            continue;
        }

        let image = setup_image(&mut images, width, height);
        if let Some(mat) = target_materials.get_mut(target_mat_handle.id()) {
            target.0 = image.clone();
            mat.texture = image;
        }
        tracing::debug!(
            size = format!(
                "Resized Vello render image to {:?}",
                (size.width, size.height)
            )
        );
    }
}

#[allow(clippy::complexity)]
pub fn setup_rendertarget(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut custom_materials: ResMut<Assets<VelloCanvasMaterial>>,
    window: Option<Single<&Window, With<PrimaryWindow>>>,
    mut render_target_mesh_handle: Local<Option<Handle<Mesh>>>,
    settings: Res<VelloCanvasSettings>,
    camera_query: Query<&Camera, With<VelloView>>,
) {
    let (width, height) = get_viewport_size(camera_query, window);

    let mesh_handle = render_target_mesh_handle.get_or_insert_with(|| {
        let mut rendertarget_quad = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );

        // Rectangle of the screen
        let verts = vec![
            [-1.0, -1.0, 0.0],
            [1.0, -1.0, 0.0],
            [1.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0],
        ];
        rendertarget_quad.insert_attribute(Mesh::ATTRIBUTE_POSITION, verts);

        let uv_pos = vec![[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [1.0, 1.0]];
        rendertarget_quad.insert_attribute(Mesh::ATTRIBUTE_UV_0, uv_pos);

        let indices = vec![0, 1, 2, 0, 2, 3];
        rendertarget_quad.insert_indices(Indices::U32(indices));

        meshes.add(rendertarget_quad)
    });
    let texture_image = setup_image(&mut images, width, height);

    commands
        .spawn((
            Name::new("Vello Canvas"),
            VelloRenderTarget(texture_image.clone()),
            Mesh2d(mesh_handle.clone()),
            MeshMaterial2d(custom_materials.add(VelloCanvasMaterial {
                texture: texture_image,
            })),
        ))
        .insert(NoFrustumCulling)
        .insert(settings.render_layers.clone());
}

/// Reinitialize for renderer settings changes.
pub fn render_settings_change_detection(
    mut commands: Commands,
    render_settings: Res<VelloRenderSettings>,
) {
    if render_settings.is_changed() && !render_settings.is_added() {
        // Replace renderer
        tracing::info!("Render settings changed, re-initializing vello...");
        commands.remove_resource::<VelloRenderer>();
        commands.init_resource::<VelloRenderer>();
    }
}
