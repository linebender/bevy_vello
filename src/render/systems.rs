use bevy::{
    camera::{RenderTarget as BevyRenderTarget, visibility::RenderLayers},
    image::ToExtents,
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_resource::{Extent3d, TextureFormat, TextureViewDescriptor},
        renderer::{RenderDevice, RenderQueue},
        texture::GpuImage,
    },
    window::PrimaryWindow,
};
use vello::{RenderParams, Scene, kurbo::Affine};

use super::{
    VelloEntityCountData, VelloFrameProfileData, VelloRenderQueues, VelloRenderSettings,
    VelloRenderer, VelloWorldRenderItem,
    extract::{VelloClearColor, VelloRenderTarget, VelloTargetSize, setup_image},
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
    render::{PerCameraRenderQueue, VelloUiRenderItem, VelloView},
};

// ---------------------------------------------------------------------------
// Main-world system: manage render targets for VelloView cameras
// ---------------------------------------------------------------------------

pub fn manage_render_targets(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut query: Query<
        (
            Entity,
            &Camera,
            Option<&mut VelloRenderTarget>,
            Option<&VelloTargetSize>,
            Has<super::VelloAutoSpawned>,
            Option<&BevyRenderTarget>,
        ),
        With<VelloView>,
    >,
    window: Option<Single<&Window, With<PrimaryWindow>>>,
) {
    for (entity, camera, render_target, target_size, is_auto, bevy_target) in query.iter_mut()
    {
        // Determine target size (in physical pixels)
        let (width, height) = if let Some(size) = target_size {
            // VelloTargetSize is in logical pixels; scale by the display's scale factor
            let scale = window
                .as_deref()
                .map(|w| w.resolution.scale_factor())
                .unwrap_or(1.0);
            (
                (size.0.x as f32 * scale) as u32,
                (size.0.y as f32 * scale) as u32,
            )
        } else if let Some(viewport) = &camera.viewport {
            (viewport.physical_size.x, viewport.physical_size.y)
        } else if let Some(window) = window.as_deref() {
            (
                window.resolution.physical_width(),
                window.resolution.physical_height(),
            )
        } else {
            continue;
        };

        if width == 0 || height == 0 {
            continue;
        }

        let expected_size = Extent3d {
            width,
            height,
            ..default()
        };

        if let Some(target) = render_target {
            // VelloRenderTarget already present (auto-spawned or previously
            // extracted from RenderTarget::Image).

            // For non-auto cameras that still have a Window render target,
            // silence Camera2d by switching to RenderTarget::None. This
            // prevents it from writing to the primary window and interfering
            // with other cameras (e.g. a Camera3d clearing the scene).
            if !is_auto && matches!(bevy_target, Some(BevyRenderTarget::Window(_))) {
                commands
                    .entity(entity)
                    .insert(BevyRenderTarget::None {
                        size: UVec2::new(width, height),
                    });
            }

            // Only resize for auto-spawned targets (always track window size)
            // or when VelloTargetSize is explicitly present (DPI-aware resize).
            // User-provided targets without VelloTargetSize keep their original size.
            if is_auto || target_size.is_some() {
                // Use immutable access first to avoid triggering change detection
                // when no resize is actually needed.
                let needs_resize = images
                    .get(target.0.id())
                    .is_some_and(|image| image.size().to_extents() != expected_size);
                if needs_resize {
                    if let Some(image) = images.get_mut(target.0.id()) {
                        image.texture_descriptor.size = expected_size;
                        image.resize(expected_size);
                        // Re-apply the sRGB view descriptor so the GPU recreates
                        // the correct Rgba8UnormSrgb default view after resize.
                        image.texture_view_descriptor = Some(TextureViewDescriptor {
                            format: Some(TextureFormat::Rgba8UnormSrgb),
                            ..default()
                        });
                        tracing::debug!(
                            "Resized Vello render target to {:?}",
                            (width, height)
                        );
                    }
                }
            }
        } else if let Some(BevyRenderTarget::Image(img_target)) = bevy_target {
            // User provided a RenderTarget::Image on first frame. Extract the
            // handle for Vello rendering and silence Camera2d with
            // RenderTarget::None so its render graph doesn't overwrite the
            // Vello content.
            let user_handle = img_target.handle.clone();
            let img_size = images
                .get(user_handle.id())
                .map(|i| i.size())
                .unwrap_or(UVec2::new(width, height));
            commands
                .entity(entity)
                .insert(VelloRenderTarget(user_handle));
            commands
                .entity(entity)
                .insert(BevyRenderTarget::None { size: img_size });

            tracing::debug!(
                "Extracted Vello render target from RenderTarget::Image {:?}",
                img_size
            );
        } else {
            // No target at all — auto-spawn a fullscreen canvas.
            let handle = setup_image(&mut images, width, height);
            commands
                .entity(entity)
                .insert(VelloRenderTarget(handle.clone()))
                .insert(super::VelloAutoSpawned);
            commands.spawn((
                Sprite {
                    image: handle,
                    ..default()
                },
                super::VelloCanvas,
            ));
            tracing::debug!(
                "Created Vello render target {:?}",
                (width, height)
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Render-world system: build per-camera render queues
// ---------------------------------------------------------------------------

/// Builds per-camera render queues by filtering entities by RenderLayers,
/// computing per-camera affines, and sorting by z-index / stack_index.
#[allow(clippy::too_many_arguments, clippy::complexity)]
pub fn build_render_queues(
    views: Query<
        (
            Entity,
            &bevy::render::camera::ExtractedCamera,
            &bevy::render::view::ExtractedView,
            Option<&RenderLayers>,
            Option<&VelloTargetSize>,
        ),
        (With<Camera2d>, With<VelloView>),
    >,
    view_world_scenes: Query<&ExtractedVelloScene2d>,
    view_ui_scenes: Query<&ExtractedUiVelloScene>,
    #[cfg(feature = "text")] view_world_text: Query<&ExtractedVelloText2d>,
    #[cfg(feature = "text")] view_ui_text: Query<&ExtractedUiVelloText>,
    #[cfg(feature = "svg")] view_world_svgs: Query<&ExtractedVelloSvg2d>,
    #[cfg(feature = "svg")] view_ui_svgs: Query<&ExtractedUiVelloSvg>,
    #[cfg(feature = "lottie")] view_world_lotties: Query<&ExtractedVelloLottie2d>,
    #[cfg(feature = "lottie")] view_ui_lotties: Query<&ExtractedUiVelloLottie>,
    render_targets: Query<&VelloRenderTarget, With<VelloView>>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    mut render_queues: ResMut<VelloRenderQueues>,
    frame_data: Res<VelloEntityCountData>,
) {
    render_queues.cameras.clear();

    for (camera_entity, camera, view, camera_render_layers, target_size) in views.iter() {
        let camera_layers = camera_render_layers.cloned().unwrap_or_default();
        let mut queue = PerCameraRenderQueue::default();

        // Look up the actual physical size of the GPU render target image
        let physical_target_size = render_targets
            .get(camera_entity)
            .ok()
            .and_then(|target| gpu_images.get(target.0.id()))
            .map(|gpu_image| UVec2::new(gpu_image.size.width, gpu_image.size.height));

        let cam_params = CameraViewParams::new(camera, view, target_size, physical_target_size);

        // Pre-estimate capacity
        let mut n_world_items: usize = frame_data.n_world_scenes as usize;
        let mut n_ui_items: usize = frame_data.n_ui_scenes as usize;
        #[cfg(feature = "text")]
        {
            n_world_items += frame_data.n_world_texts as usize;
            n_ui_items += frame_data.n_ui_texts as usize;
        }
        #[cfg(feature = "svg")]
        {
            n_world_items += frame_data.n_world_svgs as usize;
            n_ui_items += frame_data.n_ui_svgs as usize;
        }
        #[cfg(feature = "lottie")]
        {
            n_world_items += frame_data.n_world_lotties as usize;
            n_ui_items += frame_data.n_ui_lotties as usize;
        }

        let mut world_render_queue: Vec<(f32, VelloWorldRenderItem)> =
            Vec::with_capacity(n_world_items);
        let mut ui_render_queue: Vec<(u32, VelloUiRenderItem)> =
            Vec::with_capacity(n_ui_items);

        // --- World Scenes ---
        for scene in view_world_scenes.iter() {
            if !scene.render_layers.intersects(&camera_layers) {
                continue;
            }
            let affine = compute_world_affine(&cam_params, &scene.transform);
            world_render_queue.push((
                scene.transform.translation().z,
                VelloWorldRenderItem::Scene {
                    affine,
                    item: scene.clone(),
                },
            ));
        }

        // --- UI Scenes ---
        for scene in view_ui_scenes.iter() {
            if !scene.render_layers.intersects(&camera_layers) {
                continue;
            }
            let affine = compute_ui_scene_affine(scene);
            ui_render_queue.push((
                scene.ui_node.stack_index,
                VelloUiRenderItem::Scene {
                    affine,
                    item: scene.clone(),
                },
            ));
        }

        // --- World SVGs ---
        #[cfg(feature = "svg")]
        {
            for svg in view_world_svgs.iter() {
                if !svg.render_layers.intersects(&camera_layers) {
                    continue;
                }
                let anchor_offset = compute_svg_anchor_offset(
                    &svg.asset_anchor,
                    svg.asset.width,
                    svg.asset.height,
                );
                let affine =
                    compute_world_affine_with_anchor(&cam_params, &svg.transform, anchor_offset);
                world_render_queue.push((
                    svg.transform.translation().z,
                    VelloWorldRenderItem::Svg {
                        affine,
                        item: svg.clone(),
                    },
                ));
            }
        }

        // --- UI SVGs ---
        #[cfg(feature = "svg")]
        {
            for svg in view_ui_svgs.iter() {
                if !svg.render_layers.intersects(&camera_layers) {
                    continue;
                }
                let affine = compute_ui_aspect_fill_affine(
                    &svg.ui_transform,
                    &svg.ui_node,
                    svg.asset.width,
                    svg.asset.height,
                );
                ui_render_queue.push((
                    svg.ui_node.stack_index,
                    VelloUiRenderItem::Svg {
                        affine,
                        item: svg.clone(),
                    },
                ));
            }
        }

        // --- World Lotties ---
        #[cfg(feature = "lottie")]
        {
            for lottie in view_world_lotties.iter() {
                if !lottie.render_layers.intersects(&camera_layers) {
                    continue;
                }
                let (width, height) = (
                    lottie.asset.composition.width as f32,
                    lottie.asset.composition.height as f32,
                );
                let anchor_offset =
                    compute_lottie_anchor_offset(&lottie.asset_anchor, width, height);
                let affine = compute_world_affine_with_anchor(
                    &cam_params,
                    &lottie.transform,
                    anchor_offset,
                );
                world_render_queue.push((
                    lottie.transform.translation().z,
                    VelloWorldRenderItem::Lottie {
                        affine,
                        item: lottie.clone(),
                    },
                ));
            }
        }

        // --- UI Lotties ---
        #[cfg(feature = "lottie")]
        {
            for lottie in view_ui_lotties.iter() {
                if !lottie.render_layers.intersects(&camera_layers) {
                    continue;
                }
                let (width, height) = (
                    lottie.asset.composition.width as f32,
                    lottie.asset.composition.height as f32,
                );
                let affine = compute_ui_aspect_fill_affine(
                    &lottie.ui_transform,
                    &lottie.ui_node,
                    width,
                    height,
                );
                ui_render_queue.push((
                    lottie.ui_node.stack_index,
                    VelloUiRenderItem::Lottie {
                        affine,
                        item: lottie.clone(),
                    },
                ));
            }
        }

        // --- World Text ---
        #[cfg(feature = "text")]
        {
            for text in view_world_text.iter() {
                if !text.render_layers.intersects(&camera_layers) {
                    continue;
                }
                let affine = compute_world_affine(&cam_params, &text.transform);
                world_render_queue.push((
                    text.transform.translation().z,
                    VelloWorldRenderItem::Text {
                        affine,
                        item: text.clone(),
                    },
                ));
            }
        }

        // --- UI Text ---
        #[cfg(feature = "text")]
        {
            for text in view_ui_text.iter() {
                if !text.render_layers.intersects(&camera_layers) {
                    continue;
                }
                let affine = compute_ui_text_affine(text);
                ui_render_queue.push((
                    text.ui_node.stack_index,
                    VelloUiRenderItem::Text {
                        affine,
                        item: text.clone(),
                    },
                ));
            }
        }

        // Sort
        world_render_queue.sort_unstable_by(|(a_z, _), (b_z, _)| {
            a_z.partial_cmp(b_z)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        ui_render_queue.sort_unstable_by(|(a_idx, _), (b_idx, _)| {
            a_idx
                .partial_cmp(b_idx)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        queue
            .world
            .extend(world_render_queue.into_iter().map(|(_, r)| r));
        queue
            .ui
            .extend(ui_render_queue.into_iter().map(|(_, r)| r));

        render_queues.cameras.insert(camera_entity, queue);
    }
}

// ---------------------------------------------------------------------------
// Render-world system: render each camera's scene to its texture
// ---------------------------------------------------------------------------

#[allow(clippy::complexity)]
pub fn render_frames(
    render_queues: Res<VelloRenderQueues>,
    render_targets: Query<(&VelloRenderTarget, Option<&VelloClearColor>), With<VelloView>>,
    #[cfg(feature = "text")] font_render_assets: Res<RenderAssets<VelloFont>>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    renderer: Res<VelloRenderer>,
    #[cfg(feature = "lottie")] mut velato_renderer: ResMut<super::VelatoRenderer>,
    render_settings: Res<VelloRenderSettings>,
    mut frame_profile: ResMut<VelloFrameProfileData>,
) {
    // Aggregate diagnostics across all cameras
    let mut total_n_paths = 0u32;
    let mut total_n_path_segs = 0u32;
    let mut total_n_clips = 0u32;
    let mut total_n_open_clips = 0u32;

    for (camera_entity, render_queue) in render_queues.cameras.iter() {
        let Ok((target, clear_color)) = render_targets.get(*camera_entity) else {
            continue;
        };
        let Some(gpu_image) = gpu_images.get(target.0.id()) else {
            continue;
        };

        let mut scene_buffer = Scene::new();

        // World Renderables
        for render_item in render_queue.world.iter() {
            match render_item {
                VelloWorldRenderItem::Scene { affine, item } => {
                    scene_buffer.append(&item.scene, Some(*affine));
                }
                #[cfg(feature = "lottie")]
                VelloWorldRenderItem::Lottie { affine, item } => {
                    if item.alpha <= 0.0 {
                        continue;
                    }
                    if item.alpha < 1.0 {
                        scene_buffer.push_layer(
                            vello::peniko::Fill::NonZero,
                            vello::peniko::Mix::Normal,
                            item.alpha,
                            *affine,
                            &vello::kurbo::Rect::new(
                                0.0,
                                0.0,
                                item.asset.composition.width as f64,
                                item.asset.composition.height as f64,
                            ),
                        );
                    }
                    let recolored = item
                        .theme
                        .as_ref()
                        .map(|cs| cs.recolor(&item.asset.composition));
                    let animation = recolored
                        .as_ref()
                        .unwrap_or(&item.asset.composition);
                    velato_renderer.append(
                        animation,
                        item.playhead,
                        *affine,
                        1.0,
                        &mut scene_buffer,
                    );
                    if item.alpha < 1.0 {
                        scene_buffer.pop_layer();
                    }
                }
                #[cfg(feature = "svg")]
                VelloWorldRenderItem::Svg { affine, item } => {
                    if item.alpha <= 0.0 {
                        continue;
                    }
                    if item.alpha < 1.0 {
                        scene_buffer.push_layer(
                            vello::peniko::Fill::NonZero,
                            vello::peniko::Mix::Normal,
                            item.alpha,
                            *affine,
                            &vello::kurbo::Rect::new(
                                0.0,
                                0.0,
                                item.asset.width as f64,
                                item.asset.height as f64,
                            ),
                        );
                    }
                    scene_buffer.append(&item.asset.scene, Some(*affine));
                    if item.alpha < 1.0 {
                        scene_buffer.pop_layer();
                    }
                }
                #[cfg(feature = "text")]
                VelloWorldRenderItem::Text { affine, item } => {
                    if let Some(font) = font_render_assets.get(item.text.style.font.id()) {
                        font.render(
                            &mut scene_buffer,
                            *affine,
                            &item.text.value,
                            &item.text.style,
                            item.text.text_align,
                            item.text.max_advance,
                            item.text_anchor,
                        );
                    }
                }
            }
        }

        // Ui Renderables
        for render_item in render_queue.ui.iter() {
            match render_item {
                VelloUiRenderItem::Scene { affine, item } => {
                    scene_buffer.append(&item.scene, Some(*affine));
                }
                #[cfg(feature = "lottie")]
                VelloUiRenderItem::Lottie { affine, item } => {
                    if item.alpha <= 0.0 {
                        continue;
                    }
                    if item.alpha < 1.0 {
                        scene_buffer.push_layer(
                            vello::peniko::Fill::NonZero,
                            vello::peniko::Mix::Normal,
                            item.alpha,
                            *affine,
                            &vello::kurbo::Rect::new(
                                0.0,
                                0.0,
                                item.asset.composition.width as f64,
                                item.asset.composition.height as f64,
                            ),
                        );
                    }
                    let recolored = item
                        .theme
                        .as_ref()
                        .map(|cs| cs.recolor(&item.asset.composition));
                    let animation = recolored
                        .as_ref()
                        .unwrap_or(&item.asset.composition);
                    velato_renderer.append(
                        animation,
                        item.playhead,
                        *affine,
                        1.0,
                        &mut scene_buffer,
                    );
                    if item.alpha < 1.0 {
                        scene_buffer.pop_layer();
                    }
                }
                #[cfg(feature = "svg")]
                VelloUiRenderItem::Svg { affine, item } => {
                    if item.alpha <= 0.0 {
                        continue;
                    }
                    if item.alpha < 1.0 {
                        scene_buffer.push_layer(
                            vello::peniko::Fill::NonZero,
                            vello::peniko::Mix::Normal,
                            item.alpha,
                            *affine,
                            &vello::kurbo::Rect::new(
                                0.0,
                                0.0,
                                item.asset.width as f64,
                                item.asset.height as f64,
                            ),
                        );
                    }
                    scene_buffer.append(&item.asset.scene, Some(*affine));
                    if item.alpha < 1.0 {
                        scene_buffer.pop_layer();
                    }
                }
                #[cfg(feature = "text")]
                VelloUiRenderItem::Text { affine, item } => {
                    if let Some(font) = font_render_assets.get(item.text.style.font.id()) {
                        font.render(
                            &mut scene_buffer,
                            *affine,
                            &item.text.value,
                            &item.text.style,
                            item.text.text_align,
                            item.text.max_advance,
                            item.text_anchor,
                        );
                    }
                }
            }
        }

        total_n_paths += scene_buffer.encoding().n_paths;
        total_n_path_segs += scene_buffer.encoding().n_path_segments;
        total_n_clips += scene_buffer.encoding().n_clips;
        total_n_open_clips += scene_buffer.encoding().n_open_clips;

        // Vello's compute shaders require a Rgba8Unorm storage view for writes.
        // The gpu_image.texture_view is Rgba8UnormSrgb (for correct display sampling),
        // so we create a separate Rgba8Unorm view for Vello.
        let vello_view = gpu_image.texture.create_view(&TextureViewDescriptor {
            format: Some(TextureFormat::Rgba8Unorm),
            ..default()
        });

        renderer
            .lock()
            .unwrap()
            .render_to_texture(
                device.wgpu_device(),
                &queue,
                &scene_buffer,
                &vello_view,
                &RenderParams {
                    base_color: clear_color
                        .map(|c| c.0)
                        .unwrap_or(vello::peniko::Color::TRANSPARENT),
                    width: gpu_image.size.width,
                    height: gpu_image.size.height,
                    antialiasing_method: render_settings.antialiasing,
                },
            )
            .unwrap();
    }

    frame_profile.n_paths = total_n_paths;
    frame_profile.n_path_segs = total_n_path_segs;
    frame_profile.n_clips = total_n_clips;
    frame_profile.n_open_clips = total_n_open_clips;
}

/// Keeps [`VelloCanvas`] sprites sized to the primary window's logical dimensions.
pub fn resize_vello_canvases(
    mut canvases: Query<&mut Sprite, With<super::VelloCanvas>>,
    window: Option<Single<&Window, With<PrimaryWindow>>>,
) {
    let Some(window) = window.as_deref() else {
        return;
    };
    let size = Vec2::new(window.resolution.width(), window.resolution.height());
    for mut sprite in canvases.iter_mut() {
        if sprite.custom_size.as_ref() != Some(&size) {
            sprite.custom_size = Some(size);
        }
    }
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

// ===========================================================================
// Affine computation helpers
// ===========================================================================

/// Pre-computed camera matrices for affine computation.
/// When `VelloTargetSize` is present, uses a fresh projection for the target
/// dimensions so content maps to the render target's pixel space, not the window's.
struct CameraViewParams {
    ndc_to_pixels: Mat4,
    view_proj: Mat4,
}

impl CameraViewParams {
    fn new(
        camera: &bevy::render::camera::ExtractedCamera,
        view: &bevy::render::view::ExtractedView,
        target_size: Option<&VelloTargetSize>,
        physical_target_size: Option<UVec2>,
    ) -> Self {
        if let Some(size) = target_size {
            // VelloTargetSize is in logical pixels (projection bounds)
            let logical_w = size.0.x as f32;
            let logical_h = size.0.y as f32;

            // Physical size from the actual GPU image (accounts for DPI scaling)
            let physical = physical_target_size.unwrap_or(size.0);
            let physical_w = physical.x as f32;
            let physical_h = physical.y as f32;

            // NDC-to-pixels for the physical texture size
            let ndc_to_pixels = ndc_to_pixels_matrix_from_size(physical_w, physical_h);

            // Ortho projection matching the logical target size
            // (1 world unit = 1 logical pixel)
            let half_w = logical_w / 2.0;
            let half_h = logical_h / 2.0;
            let proj = Mat4::orthographic_rh(-half_w, half_w, -half_h, half_h, -1000.0, 1000.0);

            // View transform (camera position/rotation)
            let mut view_mat = view.world_from_view.to_matrix();
            view_mat.w_axis.y *= -1.0;
            let view_proj = proj * view_mat.inverse();

            Self {
                ndc_to_pixels,
                view_proj,
            }
        } else {
            // Standard: use the actual render target size for ndc_to_pixels
            // (may differ from camera viewport when texture is at logical resolution).
            let ndc_to_pixels = if let Some(phys) = physical_target_size {
                ndc_to_pixels_matrix_from_size(phys.x as f32, phys.y as f32)
            } else {
                ndc_to_pixels_matrix(camera)
            };
            Self {
                ndc_to_pixels,
                view_proj: view_proj_matrix(view),
            }
        }
    }
}

/// Compute a world-space affine for a simple transform (no anchor).
/// Used for VelloScene2d and VelloText2d.
fn compute_world_affine(params: &CameraViewParams, transform: &GlobalTransform) -> Affine {
    let model = {
        let mut m = transform.to_matrix();
        m.w_axis.y *= -1.0;
        m
    };
    let raw = params.ndc_to_pixels * params.view_proj * model;
    mat4_to_world_affine(&raw)
}

/// Compute a world-space affine with an anchor offset.
/// Used for VelloSvg2d and VelloLottie2d.
fn compute_world_affine_with_anchor(
    params: &CameraViewParams,
    transform: &GlobalTransform,
    anchor_offset: Vec3,
) -> Affine {
    let world_transform = transform.compute_transform();
    let Transform {
        translation,
        rotation,
        scale,
    } = world_transform;

    let mut anchor_matrix = Mat4::from_translation(-anchor_offset);
    anchor_matrix.w_axis.y *= -1.0;

    let translation_matrix = Mat4::from_translation(translation);
    let rotation_matrix = Mat4::from_quat(rotation);
    let scale_matrix = Mat4::from_scale(scale);

    let mut model = translation_matrix * rotation_matrix * scale_matrix * anchor_matrix;
    model.w_axis.y *= -1.0;

    let raw = params.ndc_to_pixels * params.view_proj * model;
    mat4_to_world_affine(&raw)
}

/// Compute a UI affine for scenes (pixel_scale + centering).
fn compute_ui_scene_affine(scene: &ExtractedUiVelloScene) -> Affine {
    let pixel_scale = scene.ui_render_target.scale_factor();
    let pixel_scale_matrix = Mat4::from_scale(Vec3::new(pixel_scale, pixel_scale, 1.0));
    let model = ui_model_matrix(&scene.ui_transform);

    let local_center_matrix = {
        let Vec2 {
            x: width,
            y: height,
        } = scene.ui_node.size();
        Mat4::from_translation(Vec3::new(width / 2.0, height / 2.0, 0.0)).inverse()
    };

    let raw = model * local_center_matrix * pixel_scale_matrix;
    mat4_to_ui_affine(&raw)
}

/// Compute a UI affine for text (pixel_scale, no centering).
#[cfg(feature = "text")]
fn compute_ui_text_affine(text: &ExtractedUiVelloText) -> Affine {
    let pixel_scale = text.ui_render_target.scale_factor();
    let pixel_scale_matrix = Mat4::from_scale(Vec3::new(pixel_scale, pixel_scale, 1.0));
    let model = ui_model_matrix(&text.ui_transform);

    let raw = model * pixel_scale_matrix;
    mat4_to_ui_affine(&raw)
}

/// Compute a UI affine with aspect-fill scaling (for SVG and Lottie).
fn compute_ui_aspect_fill_affine(
    ui_transform: &UiGlobalTransform,
    ui_node: &ComputedNode,
    asset_width: f32,
    asset_height: f32,
) -> Affine {
    let model = ui_model_matrix(ui_transform);
    let local_center_matrix =
        Transform::from_translation(Vec3::new(asset_width / 2.0, asset_height / 2.0, 0.0))
            .to_matrix()
            .inverse();
    let aspect_fill_matrix = {
        let asset_size = Vec2::new(asset_width, asset_height);
        let fill_scale = ui_node.size() / asset_size;
        let scale_factor = fill_scale.x.min(fill_scale.y);
        Mat4::from_scale(Vec3::new(scale_factor, scale_factor, 1.0))
    };

    let raw = model * aspect_fill_matrix * local_center_matrix;
    mat4_to_ui_affine(&raw)
}

// ---------------------------------------------------------------------------
// Matrix helpers
// ---------------------------------------------------------------------------

fn ndc_to_pixels_matrix(camera: &bevy::render::camera::ExtractedCamera) -> Mat4 {
    let size_pixels: UVec2 = camera.physical_viewport_size.unwrap();
    ndc_to_pixels_matrix_from_size(size_pixels.x as f32, size_pixels.y as f32)
}

fn ndc_to_pixels_matrix_from_size(px: f32, py: f32) -> Mat4 {
    Mat4::from_cols_array_2d(&[
        [px / 2.0, 0.0, 0.0, px / 2.0],
        [0.0, py / 2.0, 0.0, py / 2.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ])
    .transpose()
}

fn view_proj_matrix(view: &bevy::render::view::ExtractedView) -> Mat4 {
    let mut view_mat = view.world_from_view.to_matrix();
    view_mat.w_axis.y *= -1.0;
    let proj_mat = view.clip_from_view;
    proj_mat * view_mat.inverse()
}

fn ui_model_matrix(ui_transform: &UiGlobalTransform) -> Mat4 {
    let mat2 = ui_transform.matrix2;
    let translation = ui_transform.translation;
    Mat4::from_cols_array_2d(&[
        [mat2.x_axis.x, mat2.x_axis.y, 0.0, 0.0],
        [mat2.y_axis.x, mat2.y_axis.y, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [translation.x, translation.y, 0.0, 1.0],
    ])
}

/// Extract 2D affine from Mat4 for world-space (negates skew for y-up).
fn mat4_to_world_affine(m: &Mat4) -> Affine {
    let t = m.to_cols_array();
    Affine::new([
        t[0] as f64,   // scale_x
        -t[1] as f64,  // skew_y (negated)
        -t[4] as f64,  // skew_x (negated)
        t[5] as f64,   // scale_y
        t[12] as f64,  // translate_x
        t[13] as f64,  // translate_y
    ])
}

/// Extract 2D affine from Mat4 for UI-space (no skew negation).
fn mat4_to_ui_affine(m: &Mat4) -> Affine {
    let t = m.to_cols_array();
    Affine::new([
        t[0] as f64,   // scale_x
        t[1] as f64,   // skew_y
        t[4] as f64,   // skew_x
        t[5] as f64,   // scale_y
        t[12] as f64,  // translate_x
        t[13] as f64,  // translate_y
    ])
}

// ---------------------------------------------------------------------------
// Anchor offset helpers
// ---------------------------------------------------------------------------

#[cfg(feature = "svg")]
fn compute_svg_anchor_offset(
    anchor: &crate::integrations::svg::VelloSvgAnchor,
    width: f32,
    height: f32,
) -> Vec3 {
    use crate::integrations::svg::VelloSvgAnchor;
    match anchor {
        VelloSvgAnchor::TopLeft => Vec3::ZERO,
        VelloSvgAnchor::Left => Vec3::new(0.0, height / 2.0, 0.0),
        VelloSvgAnchor::BottomLeft => Vec3::new(0.0, height, 0.0),
        VelloSvgAnchor::Top => Vec3::new(width / 2.0, 0.0, 0.0),
        VelloSvgAnchor::Center => Vec3::new(width / 2.0, height / 2.0, 0.0),
        VelloSvgAnchor::Bottom => Vec3::new(width / 2.0, height, 0.0),
        VelloSvgAnchor::TopRight => Vec3::new(width, 0.0, 0.0),
        VelloSvgAnchor::Right => Vec3::new(width, height / 2.0, 0.0),
        VelloSvgAnchor::BottomRight => Vec3::new(width, height, 0.0),
    }
}

#[cfg(feature = "lottie")]
fn compute_lottie_anchor_offset(
    anchor: &crate::integrations::lottie::VelloLottieAnchor,
    width: f32,
    height: f32,
) -> Vec3 {
    use crate::integrations::lottie::VelloLottieAnchor;
    match anchor {
        VelloLottieAnchor::TopLeft => Vec3::ZERO,
        VelloLottieAnchor::Left => Vec3::new(0.0, height / 2.0, 0.0),
        VelloLottieAnchor::BottomLeft => Vec3::new(0.0, height, 0.0),
        VelloLottieAnchor::Top => Vec3::new(width / 2.0, 0.0, 0.0),
        VelloLottieAnchor::Center => Vec3::new(width / 2.0, height / 2.0, 0.0),
        VelloLottieAnchor::Bottom => Vec3::new(width / 2.0, height, 0.0),
        VelloLottieAnchor::TopRight => Vec3::new(width, 0.0, 0.0),
        VelloLottieAnchor::Right => Vec3::new(width, height / 2.0, 0.0),
        VelloLottieAnchor::BottomRight => Vec3::new(width, height, 0.0),
    }
}
