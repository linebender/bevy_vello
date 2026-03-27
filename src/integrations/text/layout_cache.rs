use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use bevy::prelude::*;
use parley::Layout;
use vello::peniko::Brush;

use super::{VelloFont, VelloTextAlign, VelloTextStyle};

/// Content-addressed key for cached text layouts.
///
/// Stores the `font_id` alongside a SipHash of (font_id, text, style, align,
/// max_advance).  The hash drives equality/lookup; the `font_id` enables
/// targeted eviction when a single font asset changes without clearing the
/// entire cache.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TextLayoutKey {
    pub font_id: AssetId<VelloFont>,
    pub hash: u64,
}

impl TextLayoutKey {
    pub fn new(
        font_id: AssetId<VelloFont>,
        value: &str,
        style: &VelloTextStyle,
        text_align: VelloTextAlign,
        max_advance: Option<f32>,
    ) -> Self {
        let mut hasher = std::hash::DefaultHasher::new();
        font_id.hash(&mut hasher);
        value.hash(&mut hasher);
        // Hash style fields that affect layout
        style.font_size.to_bits().hash(&mut hasher);
        style.line_height.to_bits().hash(&mut hasher);
        style.word_spacing.to_bits().hash(&mut hasher);
        style.letter_spacing.to_bits().hash(&mut hasher);
        // Hash font axes
        style.font_axes.weight.map(f32::to_bits).hash(&mut hasher);
        style.font_axes.width.map(f32::to_bits).hash(&mut hasher);
        style
            .font_axes
            .optical_size
            .map(f32::to_bits)
            .hash(&mut hasher);
        style.font_axes.italic.hash(&mut hasher);
        style.font_axes.slant.map(f32::to_bits).hash(&mut hasher);
        style.font_axes.grade.map(f32::to_bits).hash(&mut hasher);
        style
            .font_axes
            .thick_stroke
            .map(f32::to_bits)
            .hash(&mut hasher);
        style
            .font_axes
            .thin_stroke
            .map(f32::to_bits)
            .hash(&mut hasher);
        style
            .font_axes
            .counter_width
            .map(f32::to_bits)
            .hash(&mut hasher);
        style
            .font_axes
            .uppercase_height
            .map(f32::to_bits)
            .hash(&mut hasher);
        style
            .font_axes
            .lowercase_height
            .map(f32::to_bits)
            .hash(&mut hasher);
        style
            .font_axes
            .ascender_height
            .map(f32::to_bits)
            .hash(&mut hasher);
        style
            .font_axes
            .descender_depth
            .map(f32::to_bits)
            .hash(&mut hasher);
        style
            .font_axes
            .figure_height
            .map(f32::to_bits)
            .hash(&mut hasher);
        text_align.hash(&mut hasher);
        max_advance.map(f32::to_bits).hash(&mut hasher);
        Self {
            font_id,
            hash: hasher.finish(),
        }
    }
}

/// Double-buffered content-addressed cache for parley text layouts.
///
/// Lives in the render world. Each frame, lookups check `previous` and promote
/// hits into `current`. At the start of the next frame, `previous` is drained
/// (its allocation reused) and the buffers swap. Entries not touched for one
/// frame are naturally dropped — no periodic sweep, no variable-cost pass.
///
/// `Layout<Brush>` clones are cheap because parley's font data uses `Arc`
/// internally.
#[derive(Resource, Default)]
pub struct TextLayoutCache {
    /// Entries promoted (or newly inserted) this frame.
    current: HashMap<TextLayoutKey, Layout<Brush>>,
    /// Last frame's entries, used as a read-only lookup source.
    previous: HashMap<TextLayoutKey, Layout<Brush>>,
}

impl TextLayoutCache {
    /// Returns a cached layout or inserts one computed by the closure.
    ///
    /// Checks `current` first (already promoted this frame), then `previous`.
    /// On miss, calls the closure and inserts into `current`.
    pub fn get_or_insert_with(
        &mut self,
        key: TextLayoutKey,
        f: impl FnOnce() -> Layout<Brush>,
    ) -> Layout<Brush> {
        // Already promoted this frame — fast path.
        if let Some(layout) = self.current.get(&key) {
            return layout.clone();
        }

        // Promote from previous frame's buffer.
        if let Some(layout) = self.previous.remove(&key) {
            self.current.insert(key, layout.clone());
            return layout;
        }

        // Cache miss — compute and insert.
        let layout = f();
        self.current.insert(key, layout.clone());
        layout
    }

    /// Rotate buffers: drain `previous` (keeping its allocation), swap.
    ///
    /// Call once at the start of each frame. Anything left in `previous`
    /// was unused last frame and is dropped here.
    pub fn advance_frame(&mut self) {
        self.previous.clear();
        std::mem::swap(&mut self.current, &mut self.previous);
    }

    /// Evict all cached layouts that used `font_id`.
    ///
    /// Called when a font asset is modified or removed so that only the
    /// affected layouts are recomputed, not the entire cache.
    pub fn evict_font(&mut self, font_id: AssetId<VelloFont>) {
        self.current.retain(|k, _| k.font_id != font_id);
        self.previous.retain(|k, _| k.font_id != font_id);
    }

    /// Number of cached layouts across both buffers.
    #[cfg(test)]
    fn len(&self) -> usize {
        self.current.len() + self.previous.len()
    }

    /// Number of layouts in the current (promoted) buffer.
    #[cfg(test)]
    fn current_len(&self) -> usize {
        self.current.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    fn make_key(n: u64) -> TextLayoutKey {
        TextLayoutKey {
            font_id: AssetId::<VelloFont>::default(),
            hash: n,
        }
    }

    /// Second call with the same key must not invoke the closure.
    #[test]
    fn cache_hit_skips_closure() {
        let mut cache = TextLayoutCache::default();
        let key = make_key(42);

        let call_count = Cell::new(0u32);
        let _layout = cache.get_or_insert_with(key, || {
            call_count.set(call_count.get() + 1);
            Layout::new()
        });
        assert_eq!(call_count.get(), 1);

        let _layout = cache.get_or_insert_with(key, || {
            call_count.set(call_count.get() + 1);
            Layout::new()
        });
        assert_eq!(call_count.get(), 1, "closure must not run on cache hit");
    }

    /// Different keys produce independent cache entries.
    #[test]
    fn different_keys_are_independent() {
        let mut cache = TextLayoutCache::default();

        cache.get_or_insert_with(make_key(1), Layout::new);
        cache.get_or_insert_with(make_key(2), Layout::new);
        assert_eq!(cache.current_len(), 2);

        // Re-request key 1 — still 2 entries, no new insertion.
        cache.get_or_insert_with(make_key(1), || panic!("should be cached"));
        assert_eq!(cache.current_len(), 2);
    }

    /// After advance_frame, entries move to `previous` and are still
    /// accessible. After two advances without access, they're gone.
    #[test]
    fn entry_survives_one_idle_frame() {
        let mut cache = TextLayoutCache::default();
        cache.get_or_insert_with(make_key(1), Layout::new);
        assert_eq!(cache.current_len(), 1);

        // Advance: entry moves to previous, still reachable.
        cache.advance_frame();
        assert_eq!(cache.current_len(), 0);
        assert_eq!(cache.len(), 1);

        // Access promotes it back to current.
        cache.get_or_insert_with(make_key(1), || panic!("should be in previous"));
        assert_eq!(cache.current_len(), 1);
    }

    /// An entry not accessed for two frames is evicted.
    #[test]
    fn entry_evicted_after_two_idle_frames() {
        let mut cache = TextLayoutCache::default();
        cache.get_or_insert_with(make_key(1), Layout::new);

        cache.advance_frame(); // current→previous
        cache.advance_frame(); // previous cleared, old entry dropped
        assert_eq!(
            cache.len(),
            0,
            "entry should be evicted after 2 idle frames"
        );
    }

    fn make_font_key(font_id: AssetId<VelloFont>, n: u64) -> TextLayoutKey {
        TextLayoutKey { font_id, hash: n }
    }

    /// `evict_font` removes only entries for the given font.
    #[test]
    fn evict_font_is_selective() {
        let mut cache = TextLayoutCache::default();
        let font_a = AssetId::<VelloFont>::Uuid {
            uuid: bevy::asset::uuid::Uuid::from_u128(1),
        };
        let font_b = AssetId::<VelloFont>::Uuid {
            uuid: bevy::asset::uuid::Uuid::from_u128(2),
        };

        let key_a = make_font_key(font_a, 100);
        let key_b = make_font_key(font_b, 200);

        cache.get_or_insert_with(key_a, Layout::new);
        cache.get_or_insert_with(key_b, Layout::new);
        assert_eq!(cache.len(), 2);

        cache.evict_font(font_a);
        assert_eq!(cache.len(), 1, "only font_a entries should be evicted");

        // font_b entry still accessible.
        cache.get_or_insert_with(key_b, || panic!("font_b should still be cached"));
    }

    /// `evict_font` removes entries from both buffers.
    #[test]
    fn evict_font_spans_both_buffers() {
        let mut cache = TextLayoutCache::default();
        let font_a = AssetId::<VelloFont>::Uuid {
            uuid: bevy::asset::uuid::Uuid::from_u128(1),
        };
        let font_b = AssetId::<VelloFont>::Uuid {
            uuid: bevy::asset::uuid::Uuid::from_u128(2),
        };

        cache.get_or_insert_with(make_font_key(font_a, 10), Layout::new);
        cache.advance_frame(); // font_a entry moves to previous
        cache.get_or_insert_with(make_font_key(font_b, 20), Layout::new); // in current

        assert_eq!(cache.len(), 2);
        cache.evict_font(font_a);
        assert_eq!(cache.len(), 1, "font_a in previous should be evicted");
    }

    /// `TextLayoutKey::new` produces different keys for different text content.
    #[test]
    fn key_differs_on_text_content() {
        let style = VelloTextStyle::default();
        let font_id = AssetId::<VelloFont>::default();

        let key_a = TextLayoutKey::new(font_id, "hello", &style, VelloTextAlign::Start, None);
        let key_b = TextLayoutKey::new(font_id, "world", &style, VelloTextAlign::Start, None);
        assert_ne!(key_a, key_b);
    }

    /// Same inputs produce the same key (deterministic).
    #[test]
    fn key_stable_for_same_inputs() {
        let style = VelloTextStyle::default();
        let font_id = AssetId::<VelloFont>::default();

        let key_a = TextLayoutKey::new(font_id, "hello", &style, VelloTextAlign::Start, None);
        let key_b = TextLayoutKey::new(font_id, "hello", &style, VelloTextAlign::Start, None);
        assert_eq!(key_a, key_b);
    }

    /// Brush (text color) is intentionally excluded from the cache key
    /// because it does not affect text layout geometry.
    #[test]
    fn key_ignores_brush_color() {
        let font_id = AssetId::<VelloFont>::default();
        let mut style_white = VelloTextStyle::default();
        style_white.brush = vello::peniko::Brush::Solid(vello::peniko::Color::WHITE);
        let mut style_red = VelloTextStyle::default();
        style_red.brush =
            vello::peniko::Brush::Solid(vello::peniko::Color::new([1.0, 0.0, 0.0, 1.0]));

        let key_a = TextLayoutKey::new(font_id, "hello", &style_white, VelloTextAlign::Start, None);
        let key_b = TextLayoutKey::new(font_id, "hello", &style_red, VelloTextAlign::Start, None);
        assert_eq!(key_a, key_b, "brush must not affect layout cache key");
    }

    /// Key changes when style fields change.
    #[test]
    fn key_differs_on_font_size() {
        let font_id = AssetId::<VelloFont>::default();
        let mut style_a = VelloTextStyle::default();
        style_a.font_size = 16.0;
        let mut style_b = VelloTextStyle::default();
        style_b.font_size = 32.0;

        let key_a = TextLayoutKey::new(font_id, "hi", &style_a, VelloTextAlign::Start, None);
        let key_b = TextLayoutKey::new(font_id, "hi", &style_b, VelloTextAlign::Start, None);
        assert_ne!(key_a, key_b);
    }
}
