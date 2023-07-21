//! Final data structures that represent the high-level UI layout

use crate::{
    geometry::{Point, Size, Unit},
    prelude::TaffyZero,
};
use num_traits::real::Real;

/// Whether we are performing a full layout, or we merely need to size the node
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RunMode {
    /// A full layout for this node and all children should be computed
    PerformLayout,
    /// The layout algorithm should be executed such that an accurate container size for the node can be determined.
    /// Layout steps that aren't necessary for determining the container size of the current node can be skipped.
    ComputeSize,
}

/// Whether styles should be taken into account when computing size
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SizingMode {
    /// Only content contributions should be taken into account
    ContentSize,
    /// Inherent size styles should be taken into account in addition to content contributions
    InherentSize,
}

/// A set of margins that are available for collapsing with for block layout's margin collapsing
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CollapsibleMarginSet<U: Unit = f32> {
    /// The largest positive margin
    positive: U,
    /// The smallest negative margin (with largest absolute value)
    negative: U,
}

impl<U: Unit> CollapsibleMarginSet<U> {
    /// A default margin set with no collapsible margins
    pub fn zero() -> Self {
        Self { positive: U::zero(), negative: U::zero() }
    }

    /// Create a set from a single margin
    pub fn from_margin(margin: U) -> Self {
        if margin.is_positive() {
            Self { positive: margin, negative: U::zero() }
        } else {
            Self { positive: U::zero(), negative: margin }
        }
    }

    /// Collapse a single margin with this set
    pub fn collapse_with_margin(mut self, margin: U) -> Self {
        if margin.is_positive() {
            self.positive = Real::max(self.positive, margin);
        } else {
            self.negative = Real::min(self.negative, margin);
        }
        self
    }

    /// Collapse another margin set with this set
    pub fn collapse_with_set(mut self, other: CollapsibleMarginSet<U>) -> Self {
        self.positive = Real::max(self.positive, other.positive);
        self.negative = Real::min(self.negative, other.negative);
        self
    }

    /// Resolve the resultant margin from this set once all collapsible margins
    /// have been collapsed into it
    pub fn resolve(&self) -> U {
        self.positive + self.negative
    }
}

/// A struct containing both the size of a node and it's first baseline in each dimension (if it has any)
///
/// A baseline is the line on which text sits. Your node likely has a baseline if it is a text node, or contains
/// children that may be text nodes. See <https://www.w3.org/TR/css-writing-modes-3/#intro-baselines> for details.
/// If your node does not have a baseline (or you are unsure how to compute it), then simply return `Point::NONE`
/// for the first_baselines field
#[derive(Debug, Copy, Clone)]
pub struct SizeBaselinesAndMargins<U: Unit = f32> {
    /// The size of the node
    pub size: Size<U>,
    /// The first baseline of the node in each dimension, if any
    pub first_baselines: Point<Option<U>>,
    /// Top margin that can be collapsed with. This is used for CSS block layout and can be set to
    /// `CollapsibleMarginSet::ZERO` for other layout modes that don't support margin collapsing
    pub top_margin: CollapsibleMarginSet<U>,
    /// Bottom margin that can be collapsed with. This is used for CSS block layout and can be set to
    /// `CollapsibleMarginSet::ZERO` for other layout modes that don't support margin collapsing
    pub bottom_margin: CollapsibleMarginSet<U>,
    /// Whether margins can be collapsed through this node. This is used for CSS block layout and can
    /// be set to `false` for other layout modes that don't support margin collapsing
    pub margins_can_collapse_through: bool,
}

impl<U: Unit> SizeBaselinesAndMargins<U> {
    /// An all-zero `SizeBaselinesAndMargins` for hidden nodes
    pub fn hidden() -> Self {
        Self {
            size: Size::zero(),
            first_baselines: None,
            top_margin: CollapsibleMarginSet::zero(),
            bottom_margin: CollapsibleMarginSet::zero(),
            margins_can_collapse_through: false,
        }
    }

    /// Constructor to create a `SizeBaselinesAndMargins` from just the size and baselines
    pub fn from_size_and_baselines(size: Size<U>, first_baselines: Point<Option<U>>) -> Self {
        Self {
            size,
            first_baselines,
            top_margin: CollapsibleMarginSet::zero(),
            bottom_margin: CollapsibleMarginSet::zero(),
            margins_can_collapse_through: false,
        }
    }
}

impl<U: Unit> From<Size<U>> for SizeBaselinesAndMargins {
    fn from(size: Size<U>) -> Self {
        Self {
            size,
            first_baselines: Point::NONE,
            top_margin: CollapsibleMarginSet::zero(),
            bottom_margin: CollapsibleMarginSet::zero(),
            margins_can_collapse_through: false,
        }
    }
}

/// The final result of a layout algorithm for a single node.
#[derive(Debug, Copy, Clone)]
pub struct Layout<U: Unit = f32> {
    /// The relative ordering of the node
    ///
    /// Nodes with a higher order should be rendered on top of those with a lower order.
    /// This is effectively a topological sort of each tree.
    pub order: u32,
    /// The width and height of the node
    pub size: Size<U>,
    /// The top-left corner of the node
    pub location: Point<U>,
}

impl TaffyZero for Layout {
    fn zero() -> Self {
        Layout { order: 0, size: Size::zero(), location: Point::zero() }
    }
}

impl Layout {
    /// Creates a new zero-[`Layout`].
    ///
    /// The Zero-layout has size and location set to ZERO.
    /// The `order` value of this layout is set to the minimum value of 0.
    /// This means it should be rendered below all other [`Layout`]s.
    #[must_use]
    pub fn new() -> Self {
        Self::zero()
    }

    /// Creates a new zero-[`Layout`] with the supplied `order` value.
    ///
    /// Nodes with a higher order should be rendered on top of those with a lower order.
    /// The Zero-layout has size and location set to ZERO.
    #[must_use]
    pub fn with_order(order: u32) -> Self {
        Self { order, ..Self::zero() }
    }
}
