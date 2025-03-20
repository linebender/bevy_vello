use std::cell::RefCell;

use parley::{
    FontContext, LayoutContext,
    fontique::{Collection, CollectionOptions},
};
use vello::peniko::Brush;

thread_local! {
    pub static FONT_CONTEXT: RefCell<FontContext> = RefCell::new(FontContext {
        collection: Collection::new(CollectionOptions {
            shared: false,
            system_fonts: false,
        }),
        ..Default::default()
    });
    pub static LAYOUT_CONTEXT: RefCell<LayoutContext<Brush>> = RefCell::new(LayoutContext::new());
}
