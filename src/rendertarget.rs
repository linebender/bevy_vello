//! A shader that samples a texture with view-independent UV coordinates.

use bevy::{
    prelude::*,
    reflect::{TypePath, TypeUuid},
    render::{
        mesh::{Indices, MeshVertexBufferLayout},
        render_resource::{
            AsBindGroup, Extent3d, PrimitiveTopology, RenderPipelineDescriptor, ShaderRef,
            SpecializedMeshPipelineError, TextureDescriptor, TextureDimension, TextureFormat,
            TextureUsages, VertexBufferLayout, VertexFormat, VertexStepMode,
        },
        view::NoFrustumCulling,
    },
    sprite::{Material2d, Material2dKey, MaterialMesh2dBundle, Mesh2dHandle},
    window::{WindowResized, WindowResolution},
};

use crate::{renderer::SSRenderTarget, RenderMode, SSRT_SHADER_HANDLE};

#[derive(Component)]
struct MainCamera;

pub fn setup_image(
    _commands: &mut Commands,
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

            let image_handle = images.add(image);

            if let Some(mat) = target_materials.get_mut(target_mat_handle) {
                target.0 = image_handle.clone();
                mat.texture = image_handle;
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
    // query_vectors: Query<Entity, Added<Handle<VelloVector>>>,
    mut render_target_mesh_handle: Local<Option<Handle<Mesh>>>,
) {
    let Ok(window) = windows.get_single() else {
        return;
    };

    let mesh_handle = render_target_mesh_handle.get_or_insert_with(|| {
        let mut rendertarget_quad = Mesh::new(PrimitiveTopology::TriangleList);

        let mut v_pos = vec![[-1.0, -1.0, 0.0]];
        v_pos.push([1.0, -1.0, 0.0]);
        v_pos.push([1.0, 1.0, 0.0]);
        v_pos.push([-1.0, 1.0, 0.0]);

        // Set the position attribute
        rendertarget_quad.insert_attribute(Mesh::ATTRIBUTE_POSITION, v_pos);

        let v_pos = vec![[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [1.0, 1.0]];

        rendertarget_quad.insert_attribute(Mesh::ATTRIBUTE_UV_0, v_pos);

        let indices = vec![0, 1, 2, 0, 2, 3];
        rendertarget_quad.set_indices(Some(Indices::U32(indices)));

        meshes.add(rendertarget_quad)
    });
    let texture_image = setup_image(&mut commands, &mut images, &window.resolution);
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

#[derive(AsBindGroup, TypeUuid, TypePath, Asset, Clone)]
#[uuid = "b62bb455-a72c-4b56-87bb-81e0554e234f"]
pub struct VelloCanvasMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Handle<Image>,
}

impl Material2d for VelloCanvasMaterial {
    fn vertex_shader() -> ShaderRef {
        SSRT_SHADER_HANDLE.into()
    }

    fn fragment_shader() -> ShaderRef {
        SSRT_SHADER_HANDLE.into()
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let formats = vec![
            // Position
            VertexFormat::Float32x3,
            VertexFormat::Float32x2,
        ];

        let vertex_layout =
            VertexBufferLayout::from_vertex_formats(VertexStepMode::Vertex, formats);

        descriptor.vertex.buffers = vec![vertex_layout];

        Ok(())
    }
}

/// Hide the RenderTarget canvas if there is nothing to render
pub fn clear_when_empty(
    mut query_render_target: Query<&mut Visibility, With<SSRenderTarget>>,
    render_items: Query<(&mut RenderMode, &ViewVisibility)>,
) {
    if let Ok(mut visibility) = query_render_target.get_single_mut() {
        if render_items.is_empty() {
            *visibility = Visibility::Hidden;
        } else {
            *visibility = Visibility::Inherited;
        }
    }
}
