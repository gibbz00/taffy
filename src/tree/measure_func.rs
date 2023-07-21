//! Measure function type and trait definitions

use crate::geometry::{Size, Unit};
use crate::style::AvailableSpace;
#[cfg(any(feature = "std", feature = "alloc"))]
use crate::util::sys::Box;

/// A function type that can be used in a [`MeasureFunc`]
///
/// This trait is automatically implemented for all types (including closures) that define a function with the appropriate type signature.
pub trait Measurable<U: Unit = f32>: Send + Sync {
    /// Measure node
    fn measure(&self, known_dimensions: Size<Option<U>>, available_space: Size<AvailableSpace<U>>) -> Size<U>;
}

/// A function that can be used to compute the intrinsic size of a node
pub enum MeasureFunc<U: Unit = f32> {
    /// Stores an unboxed function
    Raw(fn(Size<Option<U>>, Size<AvailableSpace<U>>) -> Size<U>),

    /// Stores a boxed function
    #[cfg(any(feature = "std", feature = "alloc"))]
    Boxed(Box<dyn Measurable>),
}

impl<U: Unit> Measurable<U> for MeasureFunc<U> {
    /// Call the measure function to measure to the node
    #[inline(always)]
    fn measure(&self, known_dimensions: Size<Option<U>>, available_space: Size<AvailableSpace<U>>) -> Size<U> {
        match self {
            Self::Raw(measure) => measure(known_dimensions, available_space),
            #[cfg(any(feature = "std", feature = "alloc"))]
            Self::Boxed(measurable) => measurable.measure(known_dimensions, available_space),
        }
    }
}

#[cfg(test)]
mod test {
    use super::MeasureFunc;

    #[test]
    fn measure_func_is_send_and_sync() {
        fn is_send_and_sync<T: Send + Sync>() {}
        is_send_and_sync::<MeasureFunc>();
    }
}
