use bevy::{
    prelude::*,
    render::{
        extract_component::ExtractComponent,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
            TextureViewDescriptor,
        },
    },
};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum VelloExtractStep {
    // Extract renderable types, e.g. SVG, Lottie, Text, Scenes
    ExtractAssets,
    // Measure frame
    RunDiagnostics,
}

/// Internal render target for Vello rendering.
///
/// Users should provide a [`bevy::camera::RenderTarget::Image`] on their
/// [`VelloView`] camera instead. The library automatically extracts the handle
/// and creates this component internally.
#[derive(Component, Default, Clone, ExtractComponent)]
pub(crate) struct VelloRenderTarget(pub Handle<Image>);

/// Optional component to override the render target size.
/// If present on a [`VelloView`] camera, the render target uses this fixed size
/// instead of the camera's viewport size.
#[derive(Component, Clone, Copy, ExtractComponent)]
pub struct VelloTargetSize(pub UVec2);

/// Optional component to set the background color for a [`VelloView`] camera's render target.
/// Defaults to [`vello::peniko::Color::TRANSPARENT`] if not present.
#[derive(Component, Clone, Copy, ExtractComponent)]
pub struct VelloClearColor(pub vello::peniko::Color);

/// Helper for creating [`Image`]s configured for Vello rendering.
///
/// When you need a custom render target (e.g. rendering to a 3D cube texture,
/// multiple independent textures, or headless screenshots), create an image with
/// this helper and attach it via [`bevy::camera::RenderTarget::Image`]:
///
/// ```no_run
/// # use bevy::prelude::*;
/// # use bevy_vello::prelude::*;
/// # fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
/// let image = images.add(VelloImage::new(512, 512));
/// commands.spawn((
///     Camera2d,
///     VelloView,
///     RenderTarget::Image(image.clone().into()),
/// ));
/// # }
/// ```
pub struct VelloImage;

impl VelloImage {
    /// Creates an [`Image`] with the correct format, usage flags, and sRGB
    /// texture view descriptor for Vello rendering.
    ///
    /// The returned image uses `Rgba8Unorm` as the storage format (required by
    /// Vello's compute shaders) with an `Rgba8UnormSrgb` default view for
    /// correct display sampling.
    pub fn new(width: u32, height: u32) -> Image {
        let size = Extent3d {
            width,
            height,
            ..default()
        };

        let mut image = Image {
            texture_descriptor: TextureDescriptor {
                label: None,
                size,
                dimension: TextureDimension::D2,
                // Base format must be Rgba8Unorm for STORAGE_BINDING (required by Vello).
                format: TextureFormat::Rgba8Unorm,
                mip_level_count: 1,
                sample_count: 1,
                usage: TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST
                    | TextureUsages::COPY_SRC
                    | TextureUsages::STORAGE_BINDING
                    | TextureUsages::RENDER_ATTACHMENT,
                // Allow creating an sRGB view for display sampling.
                view_formats: &[TextureFormat::Rgba8UnormSrgb],
            },
            ..default()
        };

        // Bevy's default texture view will use this sRGB format, so when Sprites,
        // StandardMaterials, etc. sample the texture, the GPU automatically applies
        // sRGB-to-linear conversion. Vello writes via a separate Rgba8Unorm view.
        image.texture_view_descriptor = Some(TextureViewDescriptor {
            format: Some(TextureFormat::Rgba8UnormSrgb),
            ..default()
        });

        image.resize(size);
        image
    }
}

pub(crate) fn setup_image(images: &mut Assets<Image>, width: u32, height: u32) -> Handle<Image> {
    images.add(VelloImage::new(width, height))
}
