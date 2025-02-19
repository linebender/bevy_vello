use super::{
    extract::{ExtractedVelloText, SSRenderTarget},
    prepare::PreparedAffine,
    VelloCanvasMaterial, VelloCanvasSettings, VelloEntityCountData, VelloFrameProfileData,
    VelloRenderItem, VelloRenderQueue, VelloRenderSettings, VelloRenderer,
};
use crate::{render::extract::ExtractedVelloScene, CoordinateSpace, VelloFont};
use bevy::{
    prelude::*,
    render::{
        mesh::Indices,
        render_asset::{RenderAssetUsages, RenderAssets},
        render_resource::{
            Extent3d, PrimitiveTopology, TextureDescriptor, TextureDimension, TextureFormat,
            TextureUsages,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::GpuImage,
        view::NoFrustumCulling,
    },
    sprite::MeshMaterial2d,
    window::{PrimaryWindow, WindowResized, WindowResolution},
};
use vello::{RenderParams, Scene};

#[cfg(feature = "lottie")]
use crate::integrations::lottie::render::ExtractedLottieAsset;
#[cfg(feature = "svg")]
use crate::integrations::svg::render::ExtractedVelloSvg;

pub fn setup_image(images: &mut Assets<Image>, window: &WindowResolution) -> Handle<Image> {
    let size = Extent3d {
        width: window.physical_width(),
        height: window.physical_height(),
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
    view_text: Query<(&PreparedAffine, &ExtractedVelloText)>,
    #[cfg(feature = "svg")] view_svgs: Query<(&PreparedAffine, &ExtractedVelloSvg)>,
    #[cfg(feature = "lottie")] view_lotties: Query<(&PreparedAffine, &ExtractedLottieAsset)>,
    mut final_render_queue: ResMut<VelloRenderQueue>,
) {
    let mut render_queue: Vec<(f32, CoordinateSpace, VelloRenderItem)> = vec![];
    #[cfg(feature = "svg")]
    render_queue.extend(view_svgs.into_iter().map(|(&affine, asset)| {
        (
            asset.transform.translation().z,
            asset.render_mode,
            VelloRenderItem::Svg {
                affine: *affine,
                item: asset.clone(),
            },
        )
    }));
    #[cfg(feature = "lottie")]
    render_queue.extend(view_lotties.into_iter().map(|(&affine, asset)| {
        (
            asset.transform.translation().z,
            asset.render_mode,
            VelloRenderItem::Lottie {
                affine: *affine,
                item: asset.clone(),
            },
        )
    }));
    render_queue.extend(view_scenes.iter().map(|(&affine, scene)| {
        (
            scene.transform.translation().z,
            scene.render_mode,
            VelloRenderItem::Scene {
                affine: *affine,
                item: scene.clone(),
            },
        )
    }));
    render_queue.extend(view_text.iter().map(|(&affine, text)| {
        (
            text.transform.translation().z,
            text.render_space,
            VelloRenderItem::Text {
                affine: *affine,
                item: text.clone(),
            },
        )
    }));

    // Sort by render mode with screen space on top, then by z-index
    render_queue.sort_by(
        |(a_z_index, a_coord_space, _), (b_z_index, b_coord_space, _)| {
            let z_index = a_z_index
                .partial_cmp(b_z_index)
                .unwrap_or(std::cmp::Ordering::Equal);
            let render_mode = a_coord_space.cmp(b_coord_space);
            render_mode.then(z_index)
        },
    );

    // Render queue is drained on render
    final_render_queue.clear();
    final_render_queue.extend(render_queue.into_iter().map(|(_, _, r)| r));
}

/// Transforms all the vectors extracted from the game world and places them in
/// a scene, and renders the scene to a texture with WGPU
#[allow(clippy::complexity)]
pub fn render_frame(
    ss_render_target: Single<&SSRenderTarget>,
    mut font_render_assets: ResMut<RenderAssets<VelloFont>>,
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
                if *alpha < 1.0 {
                    scene_buffer.push_layer(
                        vello::peniko::Mix::Normal,
                        *alpha,
                        *affine,
                        &vello::kurbo::Rect::new(0.0, 0.0, asset.width as f64, asset.height as f64),
                    );
                }
                velato_renderer.append(
                    {
                        theme
                            .as_ref()
                            .map(|cs| cs.recolor(&asset.composition))
                            .as_ref()
                            .unwrap_or(&asset.composition)
                    },
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
            VelloRenderItem::Text {
                affine,
                item:
                    ExtractedVelloText {
                        text, text_anchor, ..
                    },
            } => {
                if let Some(font) = font_render_assets.get_mut(text.style.font.id()) {
                    font.render(&mut scene_buffer, *affine, text, *text_anchor);
                }
            }
        }
    }

    frame_profile.n_path_segs = scene_buffer.encoding().n_path_segments;

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
                width: gpu_image.size.x,
                height: gpu_image.size.y,
                antialiasing_method: render_settings.antialiasing,
            },
        )
        .unwrap();
}

pub fn resize_rendertargets(
    mut window_resize_events: EventReader<WindowResized>,
    mut query: Query<(&mut SSRenderTarget, &MeshMaterial2d<VelloCanvasMaterial>)>,
    mut images: ResMut<Assets<Image>>,
    mut target_materials: ResMut<Assets<VelloCanvasMaterial>>,
    window: Option<Single<&Window, With<PrimaryWindow>>>,
) {
    let Some(window) = window.as_deref() else {
        // We only support rendering to the primary window right now.
        return;
    };
    if window_resize_events.read().last().is_some() {
        let size = Extent3d {
            width: window.resolution.physical_width(),
            height: window.resolution.physical_height(),
            ..default()
        };
        if size.width == 0 || size.height == 0 {
            return;
        }
        for (mut target, target_mat_handle) in query.iter_mut() {
            let image = setup_image(&mut images, &window.resolution);
            if let Some(mat) = target_materials.get_mut(target_mat_handle.id()) {
                target.0 = image.clone();
                mat.texture = image;
            }
            debug!(
                size = format!(
                    "Resized Vello render image to {:?}",
                    (size.width, size.height)
                )
            );
        }
    }
}

pub fn setup_ss_rendertarget(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut custom_materials: ResMut<Assets<VelloCanvasMaterial>>,
    window: Option<Single<&Window, With<PrimaryWindow>>>,
    mut render_target_mesh_handle: Local<Option<Handle<Mesh>>>,
    settings: Res<VelloCanvasSettings>,
) {
    let Some(window) = window.as_deref() else {
        // We only support rendering to the primary window right now.
        return;
    };
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
    let texture_image = setup_image(&mut images, &window.resolution);

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
        info!("Render settings changed, re-initializing vello...");
        commands.remove_resource::<VelloRenderer>();
        commands.init_resource::<VelloRenderer>();
    }
}

/// Hide the render target canvas if there is nothing to render
pub fn hide_when_empty(
    mut query_render_target: Option<Single<&mut Visibility, With<SSRenderTarget>>>,
    entity_count: Res<VelloEntityCountData>,
) {
    let is_empty = entity_count.n_scenes == 0 && entity_count.n_texts == 0;
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
