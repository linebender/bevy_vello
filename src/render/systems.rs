use crate::{CoordinateSpace, VectorFile, VelloCanvasMaterial, VelloFont};
use bevy::{
    prelude::*,
    render::{
        mesh::Indices,
        render_asset::{RenderAssetUsages, RenderAssets},
        render_resource::{
            Extent3d, PrimitiveTopology, TextureDescriptor, TextureDimension,
            TextureFormat, TextureUsages,
        },
        renderer::{RenderDevice, RenderQueue},
        view::NoFrustumCulling,
    },
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    window::{WindowResized, WindowResolution},
};
use vello::{RenderParams, Scene};

use super::{
    extract::{ExtractedRenderText, ExtractedRenderVector, SSRenderTarget},
    prepare::PreparedAffine,
    BevyVelloRenderer, LottieRenderer,
};

pub fn setup_image(
    images: &mut Assets<Image>,
    window: &WindowResolution,
) -> Handle<Image> {
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

/// Transforms all the vectors extracted from the game world and places them in
/// a scene, and renders the scene to a texture with WGPU
#[allow(clippy::complexity)]
pub fn render_scene(
    ss_render_target: Query<&SSRenderTarget>,
    render_vectors: Query<(&PreparedAffine, &ExtractedRenderVector)>,
    query_render_texts: Query<(&PreparedAffine, &ExtractedRenderText)>,
    mut font_render_assets: ResMut<RenderAssets<VelloFont>>,
    gpu_images: Res<RenderAssets<Image>>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    vello_renderer: Option<NonSendMut<BevyVelloRenderer>>,
    mut velottie_renderer: ResMut<LottieRenderer>,
) {
    let mut renderer = if let Some(renderer) = vello_renderer {
        renderer
    } else {
        return;
    };

    if let Ok(SSRenderTarget(render_target_image)) =
        ss_render_target.get_single()
    {
        let gpu_image = gpu_images.get(render_target_image).unwrap();
        let mut scene = Scene::new();

        enum RenderItem<'a> {
            Vector(&'a ExtractedRenderVector),
            Text(&'a ExtractedRenderText),
        }
        let mut render_queue: Vec<(
            f32,
            CoordinateSpace,
            (&PreparedAffine, RenderItem),
        )> = render_vectors
            .iter()
            .map(|(a, b)| {
                (b.z_index, b.render_mode, (a, RenderItem::Vector(b)))
            })
            .collect();
        render_queue.extend(query_render_texts.iter().map(|(a, b)| {
            (
                b.transform.translation().z,
                b.render_mode,
                (a, RenderItem::Text(b)),
            )
        }));

        // Sort by render mode with screen space on top, then by z-index
        render_queue.sort_by(
            |(a_z_index, a_render_mode, _), (b_z_index, b_render_mode, _)| {
                let z_index = a_z_index
                    .partial_cmp(b_z_index)
                    .unwrap_or(std::cmp::Ordering::Equal);
                let render_mode = a_render_mode.cmp(b_render_mode);

                render_mode.then(z_index)
            },
        );

        // Apply transforms to the respective fragments and add them to the
        // scene to be rendered
        for (_, _, (&PreparedAffine(affine), render_item)) in
            render_queue.iter_mut()
        {
            match render_item {
                RenderItem::Vector(ExtractedRenderVector {
                    asset,
                    theme,
                    alpha,
                    playhead,
                    ..
                }) => match &asset.data {
                    VectorFile::Svg { scene: svg, .. } => {
                        scene.append(svg, Some(affine));
                    }
                    VectorFile::Lottie { composition } => {
                        debug!("playhead: {playhead}");

                        velottie_renderer.0.render(
                            {
                                theme
                                    .as_ref()
                                    .map(|cs| cs.recolor(composition))
                                    .as_ref()
                                    .unwrap_or(composition)
                            },
                            *playhead,
                            affine,
                            *alpha,
                            &mut scene,
                        );
                    }
                },
                RenderItem::Text(ExtractedRenderText {
                    font, text, ..
                }) => {
                    if let Some(font) = font_render_assets.get_mut(font) {
                        font.render(&mut scene, affine, text);
                    }
                }
            }
        }

        if !render_queue.is_empty() {
            renderer
                .0
                .render_to_texture(
                    device.wgpu_device(),
                    &queue,
                    &scene,
                    &gpu_image.texture_view,
                    &RenderParams {
                        base_color: vello::peniko::Color::BLACK
                            .with_alpha_factor(0.0),
                        width: gpu_image.size.x as u32,
                        height: gpu_image.size.y as u32,
                        antialiasing_method: vello::AaConfig::Area,
                    },
                )
                .unwrap();
        }
    }
}

pub fn resize_rendertargets(
    mut window_resize_events: EventReader<WindowResized>,
    mut query: Query<(&mut SSRenderTarget, &Handle<VelloCanvasMaterial>)>,
    mut images: ResMut<Assets<Image>>,
    mut target_materials: ResMut<Assets<VelloCanvasMaterial>>,
    windows: Query<&Window>,
) {
    let Ok(window) = windows.get_single() else {
        return;
    };
    if window_resize_events.read().last().is_some() {
        let size = Extent3d {
            width: window.resolution.physical_width(),
            height: window.resolution.physical_height(),
            ..default()
        };
        for (mut target, target_mat_handle) in query.iter_mut() {
            let image = setup_image(&mut images, &window.resolution);
            if let Some(mat) = target_materials.get_mut(target_mat_handle) {
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
    windows: Query<&Window>,
    mut render_target_mesh_handle: Local<Option<Handle<Mesh>>>,
) {
    let Ok(window) = windows.get_single() else {
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
    let render_target = SSRenderTarget(texture_image.clone());
    let mesh = Mesh2dHandle(mesh_handle.clone());
    let material = custom_materials.add(VelloCanvasMaterial {
        texture: texture_image,
    });

    commands
        .spawn(MaterialMesh2dBundle {
            mesh,
            material,
            transform: Transform::from_translation(0.001 * Vec3::NEG_Z), // Make sure the vello canvas renders behind Gizmos
            ..Default::default()
        })
        .insert(NoFrustumCulling)
        .insert(render_target);
}

/// Hide the RenderTarget canvas if there is nothing to render
pub fn clear_when_empty(
    mut query_render_target: Query<&mut Visibility, With<SSRenderTarget>>,
    render_items: Query<(&mut CoordinateSpace, &ViewVisibility)>,
) {
    if let Ok(mut visibility) = query_render_target.get_single_mut() {
        if render_items.is_empty() {
            *visibility = Visibility::Hidden;
        } else {
            *visibility = Visibility::Inherited;
        }
    }
}
