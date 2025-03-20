use std::cell::RefCell;

use parley::{
    fontique::{Collection, CollectionOptions, SourceCache},
    FontContext, LayoutContext,
};
use vello::peniko::Brush;

static GLOBAL_FONT_CONTEXT: std::sync::OnceLock<FontContext> = std::sync::OnceLock::new();

pub(crate) fn get_global_font_context() -> &'static FontContext {
    GLOBAL_FONT_CONTEXT.get_or_init(|| FontContext {
        collection: Collection::new(CollectionOptions {
            shared: true,
            system_fonts: false,
        }),
        source_cache: SourceCache::new_shared(),
    })
}

thread_local! {
    pub static LOCAL_FONT_CONTEXT: RefCell<Option<FontContext>> = const { RefCell::new(None) };
    pub static LOCAL_LAYOUT_CONTEXT: RefCell<LayoutContext<Brush>> = RefCell::new(LayoutContext::new());
}
