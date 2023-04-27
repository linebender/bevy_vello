mod tessellation;
mod vertex_buffer;

pub use tessellation::generate_buffer;
pub mod usvg_draw;

/// A locally defined [`std::convert::Into`] surrogate to overcome orphan rules.
pub trait Convert<T>: Sized {
    /// Converts the value to `T`.
    fn convert(self) -> T;
}
