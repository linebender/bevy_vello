use bevy::{
    prelude::*,
    mesh::Indices,
    asset::RenderAssetUsages,
    camera::visibility::NoFrustumCulling,
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
    window::{PrimaryWindow, WindowResized},
};
use vello::{RenderParams, Scene};

use super::{
    VelloCanvasMaterial, VelloCanvasSettings, VelloEntityCountData, VelloFrameProfileData,
    VelloRenderItem, VelloRenderQueue, VelloRenderSettings, VelloRenderer, extract::SSRenderTarget,
    prepare::PreparedAffine,
};
#[cfg(feature = "lottie")]
use crate::integrations::lottie::render::ExtractedLottieAsset;
#[cfg(feature = "svg")]
use crate::integrations::svg::render::ExtractedVelloSvg;
#[cfg(feature = "text")]
use crate::integrations::text::{VelloFont, render::ExtractedVelloText};
use crate::render::{VelloView, extract::ExtractedVelloScene};

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

pub fn sort_render_items(
    view_scenes: Query<(&PreparedAffine, &ExtractedVelloScene)>,
    #[cfg(feature = "text")] view_text: Query<(&PreparedAffine, &ExtractedVelloText)>,
    #[cfg(feature = "svg")] view_svgs: Query<(&PreparedAffine, &ExtractedVelloSvg)>,
    #[cfg(feature = "lottie")] view_lotties: Query<(&PreparedAffine, &ExtractedLottieAsset)>,
    mut final_render_queue: ResMut<VelloRenderQueue>,
    frame_data: ResMut<VelloEntityCountData>,
) {
    let mut n_render_items: usize = 0;

    n_render_items += frame_data.n_scenes as usize;

    #[cfg(feature = "text")]
    {
        n_render_items += frame_data.n_texts as usize;
    }

    #[cfg(feature = "svg")]
    {
        n_render_items += frame_data.n_svgs as usize;
    }

    #[cfg(feature = "lottie")]
    {
        n_render_items += frame_data.n_lotties as usize;
    }

    // Reserve space for the render queues to avoid reallocations
    let mut world_render_queue: Vec<(f32, VelloRenderItem)> = Vec::with_capacity(n_render_items);
    let mut screen_render_queue: Vec<(f32, VelloRenderItem)> = Vec::with_capacity(n_render_items);

    #[cfg(feature = "svg")]
    for (&affine, asset) in view_svgs.iter() {
        if asset.ui_node.is_some() || asset.screen_space.is_some() {
            screen_render_queue.push((
                asset
                    .z_index
                    .map_or_else(|| asset.transform.translation().z, |z| z.0 as f32),
                VelloRenderItem::Svg {
                    affine: *affine,
                    item: asset.clone(),
                },
            ));
        } else {
            world_render_queue.push((
                asset.transform.translation().z,
                VelloRenderItem::Svg {
                    affine: *affine,
                    item: asset.clone(),
                },
            ));
        }
    }

    #[cfg(feature = "lottie")]
    for (&affine, asset) in view_lotties.iter() {
        if asset.ui_node.is_some() || asset.screen_space.is_some() {
            screen_render_queue.push((
                asset
                    .z_index
                    .map_or_else(|| asset.transform.translation().z, |z| z.0 as f32),
                VelloRenderItem::Lottie {
                    affine: *affine,
                    item: asset.clone(),
                },
            ));
        } else {
            world_render_queue.push((
                asset.transform.translation().z,
                VelloRenderItem::Lottie {
                    affine: *affine,
                    item: asset.clone(),
                },
            ));
        }
    }

    for (&affine, scene) in view_scenes.iter() {
        if scene.ui_node.is_some() || scene.screen_space.is_some() {
            screen_render_queue.push((
                scene
                    .z_index
                    .map_or_else(|| scene.transform.translation().z, |z| z.0 as f32),
                VelloRenderItem::Scene {
                    affine: *affine,
                    item: scene.clone(),
                },
            ));
        } else {
            world_render_queue.push((
                scene.transform.translation().z,
                VelloRenderItem::Scene {
                    affine: *affine,
                    item: scene.clone(),
                },
            ));
        }
    }

    #[cfg(feature = "text")]
    for (&affine, text) in view_text.iter() {
        if text.ui_node.is_some() || text.screen_space.is_some() {
            screen_render_queue.push((
                text.z_index
                    .map_or_else(|| text.transform.translation().z, |z| z.0 as f32),
                VelloRenderItem::Text {
                    affine: *affine,
                    item: text.clone(),
                },
            ));
        } else {
            world_render_queue.push((
                text.transform.translation().z,
                VelloRenderItem::Text {
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
    screen_render_queue.sort_unstable_by(|(a_z_index, _), (b_z_index, _)| {
        a_z_index
            .partial_cmp(b_z_index)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Render queue is drained on render
    final_render_queue.clear();

    // Reserve space for the final render queue to avoid reallocations
    final_render_queue.reserve(n_render_items);
    final_render_queue.extend(
        world_render_queue
            .into_iter()
            .chain(screen_render_queue)
            .map(|(_, r)| r),
    );
}

/// Transforms all the vectors extracted from the game world and places them in
/// a scene, and renders the scene to a texture with WGPU
#[allow(clippy::complexity)]
pub fn render_frame(
    ss_render_target: Single<&SSRenderTarget>,
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
    let SSRenderTarget(render_target_image) = *ss_render_target;
    let gpu_image = gpu_images.get(render_target_image).unwrap();

    let mut scene_buffer = Scene::new();

    for render_item in render_queue.iter() {
        match render_item {
            #[cfg(feature = "lottie")]
            VelloRenderItem::Lottie {
                affine,
                item:
                    ExtractedLottieAsset {
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
                        &vello::kurbo::Rect::new(0.0, 0.0, asset.width as f64, asset.height as f64),
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
            VelloRenderItem::Svg {
                affine,
                item: ExtractedVelloSvg { asset, alpha, .. },
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
            VelloRenderItem::Scene {
                affine,
                item: ExtractedVelloScene { scene, .. },
            } => {
                scene_buffer.append(scene, Some(*affine));
            }
            #[cfg(feature = "text")]
            VelloRenderItem::Text {
                affine,
                item:
                    ExtractedVelloText {
                        text, text_anchor, ..
                    },
            } => {
                if let Some(font) = font_render_assets.get(text.style.font.id()) {
                    font.render(&mut scene_buffer, *affine, text, *text_anchor);
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
    if let Ok(camera) = camera_query.single() {
        if let Some(viewport) = &camera.viewport {
            return (viewport.physical_size.x, viewport.physical_size.y);
        }
    }

    let Some(window) = window.as_deref() else {
        panic!("We only support rendering to the primary window right now.");
    };
    (
        window.resolution.physical_width(),
        window.resolution.physical_height(),
    )
}

pub fn resize_rendertargets(
    window_resize_events: MessageReader<WindowResized>,
    mut query: Query<(&mut SSRenderTarget, &MeshMaterial2d<VelloCanvasMaterial>)>,
    mut images: ResMut<Assets<Image>>,
    mut target_materials: ResMut<Assets<VelloCanvasMaterial>>,
    window: Option<Single<&Window, With<PrimaryWindow>>>,
    camera_query: Query<&Camera, With<VelloView>>,
) {
    if window_resize_events.is_empty() {
        return;
    }

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
pub fn setup_ss_rendertarget(
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
            SSRenderTarget(texture_image.clone()),
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

/// Hide the render target canvas if there is nothing to render
pub fn hide_when_empty(
    mut query_render_target: Option<Single<&mut Visibility, With<SSRenderTarget>>>,
    entity_count: Res<VelloEntityCountData>,
) {
    let is_empty = entity_count.n_scenes == 0;
    #[cfg(feature = "text")]
    let is_empty = is_empty && entity_count.n_texts == 0;
    #[cfg(feature = "svg")]
    let is_empty = is_empty && entity_count.n_svgs == 0;
    #[cfg(feature = "lottie")]
    let is_empty = is_empty && entity_count.n_lotties == 0;
    if let Some(visibility) = query_render_target.as_deref_mut() {
        if is_empty {
            **visibility = Visibility::Hidden;
        } else {
            **visibility = Visibility::Inherited;
        }
    }
}
