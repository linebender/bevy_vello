use std::{thread::sleep, time::Duration};

use super::{context::FONT_CONTEXT, font::VelloFont};
use crate::integrations::VectorLoaderError;
use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    prelude::*,
    tasks::ComputeTaskPool,
};

#[derive(Default)]
pub struct VelloFontLoader;

pub struct VelloFontLoaderPlugin;

impl Plugin for VelloFontLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            main_thread_and_compute_pool_font_context_update_system,
        );
    }
}

// Sync the font context with the bevy main thread and thread pools that needs to use it
fn main_thread_and_compute_pool_font_context_update_system(world: &mut World) {
    let asset_events = world
        .get_resource::<Events<AssetEvent<VelloFont>>>()
        .unwrap();

    if asset_events.is_empty() {
        return;
    }

    let vello_fonts = world.get_resource::<Assets<VelloFont>>().unwrap();

    if let Some(compute_task_pool) = ComputeTaskPool::try_get() {
        for (_handle, font) in vello_fonts.iter() {
            let compute_threads = compute_task_pool.thread_num();

            FONT_CONTEXT.with_borrow_mut(|font_context| {
                font_context.collection.register_fonts(font.bytes.clone());
            });

            for _ in 0..compute_threads {
                let font = font.clone();

                compute_task_pool
                    .spawn(async move {
                        debug!(
                            "Compute Thread {:?} registering font {:?}",
                            std::thread::current().id(),
                            font.family_name
                        );
                        FONT_CONTEXT.with_borrow_mut(|font_context| {
                            font_context.collection.register_fonts(font.bytes.clone());
                        });
                        // TODO: investigate implementing thread local gen tracking instead of
                        // sleeping
                        sleep(Duration::from_millis(100));
                    })
                    .detach();
            }
        }
    }
}

impl AssetLoader for VelloFontLoader {
    type Asset = VelloFont;

    type Settings = ();

    type Error = VectorLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        FONT_CONTEXT.with_borrow_mut(|font_context| {
            let registered_fonts = font_context.collection.register_fonts(bytes.clone());
            // TODO: handle multiple fonts in the same font file
            let (family_id, _font_info_vec) = registered_fonts.first().unwrap();
            let family_name = font_context.collection.family_name(*family_id).unwrap();
            let vello_font = VelloFont {
                family_name: family_name.to_string(),
                bytes,
            };
            Ok(vello_font)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["ttf"]
    }
}
