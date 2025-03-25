use bevy::prelude::*;

use super::{
    context::{get_global_font_context, LOCAL_FONT_CONTEXT},
    VelloFont, VelloTextSection,
};

pub(crate) struct DefaultFontPlugin;

impl Plugin for DefaultFontPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_default_font)
            .add_systems(Update, (attach_default_font, on_vello_default_font_updated));
    }
}

#[derive(Default, Debug, Resource, Deref)]
struct DefaultFontFamilyHandle(pub Handle<VelloFont>);

fn setup_default_font(mut commands: Commands, asset_server: Res<AssetServer>) {
    LOCAL_FONT_CONTEXT.with_borrow_mut(|font_context| {
        if font_context.is_none() {
            *font_context = Some(get_global_font_context().clone());
        }
        let font_context = font_context.as_mut().unwrap();
        // get the family_name of the bevy default font that VelloFont needs for Parley
        let bytes = bevy::text::DEFAULT_FONT_DATA.to_vec();
        let registered_fonts = font_context.collection.register_fonts(bytes.clone());
        let maybe_font = registered_fonts.first();
        if maybe_font.is_none() {
            warn!("Failed to register default font");
        }
        let (family_id, _font_info_vec) = maybe_font.unwrap();
        let family_name = font_context.collection.family_name(*family_id).unwrap();
        let default_font_handle = asset_server.add(VelloFont {
            bytes,
            family_name: family_name.to_string(),
        });
        commands.insert_resource(DefaultFontFamilyHandle(default_font_handle));
    });
}

fn on_vello_default_font_updated(
    default_font_family_handle: Res<DefaultFontFamilyHandle>,
    mut vello_text_section_q: Query<&mut VelloTextSection>,
) {
    if default_font_family_handle.is_changed() {
        for mut vello_text_section in vello_text_section_q.iter_mut() {
            if vello_text_section.style.font == Handle::<VelloFont>::default() {
                vello_text_section.style.font = default_font_family_handle.clone();
            }
        }
    }
}

fn attach_default_font(
    mut vello_text_section_q: Query<&mut VelloTextSection, Added<VelloTextSection>>,
    default_font_family_handle: Res<DefaultFontFamilyHandle>,
) {
    for mut vello_text_section in vello_text_section_q.iter_mut() {
        if vello_text_section.style.font == Handle::<VelloFont>::default() {
            vello_text_section.style.font = default_font_family_handle.clone();
        }
    }
}
