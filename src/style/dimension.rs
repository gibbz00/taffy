//! Style types for representing lengths / sizes

use crate::geometry::{Rect, Size, Unit};
use crate::style_helpers::{FromLength, FromPercent, TaffyAuto, TaffyMaxContent, TaffyMinContent, TaffyZero};
use crate::util::sys::abs;

/// A unit of linear measurement
///
/// This is commonly combined with [`Rect`], [`Point`](crate::geometry::Point) and [`Size<T>`].
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum LengthPercentage<U: Unit = f32> {
    /// An absolute length in some abstract units. Users of Taffy may define what they correspond
    /// to in their application (pixels, logical pixels, mm, etc) as they see fit.
    Length(U),
    /// The dimension is stored in percentage relative to the parent item.
    Percent(U),
}
impl<U: Unit> TaffyZero for LengthPercentage<U> {
    fn zero() -> Self {
        Self::Length(U::zero())
    }
}
impl<U: Unit> FromLength for LengthPercentage<U> {
    fn from_length<Input: Into<U> + Copy>(value: Input) -> Self {
        Self::Length(value.into())
    }
}
impl<U: Unit> FromPercent for LengthPercentage<U> {
    fn from_percent<Input: Into<U> + Copy>(percent: Input) -> Self {
        Self::Percent(percent.into())
    }
}

/// A unit of linear measurement
///
/// This is commonly combined with [`Rect`], [`Point`](crate::geometry::Point) and [`Size<T>`].
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum LengthPercentageAuto<U: Unit = f32> {
    /// An absolute length in some abstract units. Users of Taffy may define what they correspond
    /// to in their application (pixels, logical pixels, mm, etc) as they see fit.
    Length(U),
    /// The dimension is stored in percentage relative to the parent item.
    Percent(U),
    /// The dimension should be automatically computed
    Auto,
}
impl<U: Unit> TaffyAuto for LengthPercentageAuto<U> {
    const AUTO: Self = Self::Auto;
}
impl<U: Unit> TaffyZero for LengthPercentageAuto<U> {
    fn zero() -> Self {
        Self::Length(U::zero())
    }
}
impl<U: Unit> FromLength for LengthPercentageAuto<U> {
    fn from_length<Input: Into<U> + Copy>(value: Input) -> Self {
        Self::Length(value.into())
    }
}
impl<U: Unit> FromPercent for LengthPercentageAuto<U> {
    fn from_percent<Input: Into<U> + Copy>(percent: Input) -> Self {
        Self::Percent(percent.into())
    }
}

impl From<LengthPercentage> for LengthPercentageAuto {
    fn from(input: LengthPercentage) -> Self {
        match input {
            LengthPercentage::Length(value) => Self::Length(value),
            LengthPercentage::Percent(value) => Self::Percent(value),
        }
    }
}

impl<U: Unit> LengthPercentageAuto<U> {
    /// Returns:
    ///   - Some(length) for Length variants
    ///   - Some(resolved) using the provided context for Percent variants
    ///   - None for Auto variants
    #[inline(always)]
    pub fn resolve_to_option(self, context: U) -> Option<U> {
        match self {
            Self::Length(length) => Some(length),
            Self::Percent(percent) => Some(context * percent),
            Self::Auto => None,
        }
    }

    /// Returns true if value is LengthPercentageAuto::Auto
    #[inline(always)]
    pub fn is_auto(self) -> bool {
        self == Self::Auto
    }
}

/// A unit of linear measurement
///
/// This is commonly combined with [`Rect`], [`Point`](crate::geometry::Point) and [`Size<T>`].
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Dimension<U: Unit = f32> {
    /// An absolute length in some abstract units. Users of Taffy may define what they correspond
    /// to in their application (pixels, logical pixels, mm, etc) as they see fit.
    Length(U),
    /// The dimension is stored in percentage relative to the parent item.
    Percent(U),
    /// The dimension should be automatically computed
    Auto,
}
impl<U: Unit> TaffyAuto for Dimension<U> {
    const AUTO: Self = Self::Auto;
}
impl<U: Unit> TaffyZero for Dimension<U> {
    fn zero() -> Self {
        Self::Length(U::zero())
    }
}
impl<U: Unit> FromLength for Dimension<U> {
    fn from_length<Input: Into<U> + Copy>(value: Input) -> Self {
        Self::Length(value.into())
    }
}
impl<U: Unit> FromPercent for Dimension<U> {
    fn from_percent<Input: Into<U> + Copy>(percent: Input) -> Self {
        Self::Percent(percent.into())
    }
}

impl From<LengthPercentage> for Dimension {
    fn from(input: LengthPercentage) -> Self {
        match input {
            LengthPercentage::Length(value) => Self::Length(value),
            LengthPercentage::Percent(value) => Self::Percent(value),
        }
    }
}

impl From<LengthPercentageAuto> for Dimension {
    fn from(input: LengthPercentageAuto) -> Self {
        match input {
            LengthPercentageAuto::Length(value) => Self::Length(value),
            LengthPercentageAuto::Percent(value) => Self::Percent(value),
            LengthPercentageAuto::Auto => Self::Auto,
        }
    }
}

impl<U: Unit> Dimension<U> {
    /// Get Length value if value is Length variant
    #[cfg(feature = "grid")]
    pub fn into_option(self) -> Option<U> {
        match self {
            Dimension::Length(value) => Some(value),
            _ => None,
        }
    }
}

impl<U: Unit> Rect<Dimension<U>> {
    /// Create a new Rect with [`Dimension::Length`]
    #[must_use]
    pub const fn from_length(start: U, end: U, top: U, bottom: U) -> Self {
        Rect {
            left: Dimension::Length(start),
            right: Dimension::Length(end),
            top: Dimension::Length(top),
            bottom: Dimension::Length(bottom),
        }
    }

    /// Create a new Rect with [`Dimension::Percent`]
    #[must_use]
    pub const fn from_percent(start: U, end: U, top: U, bottom: U) -> Self {
        Rect {
            left: Dimension::Percent(start),
            right: Dimension::Percent(end),
            top: Dimension::Percent(top),
            bottom: Dimension::Percent(bottom),
        }
    }
}

/// The amount of space available to a node in a given axis
/// <https://www.w3.org/TR/css-sizing-3/#available>
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum AvailableSpace<U: Unit> {
    /// The amount of space available is the specified number of pixels
    Definite(U),
    /// The amount of space available is indefinite and the node should be laid out under a min-content constraint
    MinContent,
    /// The amount of space available is indefinite and the node should be laid out under a max-content constraint
    MaxContent,
}
impl<U: Unit> TaffyMaxContent for AvailableSpace<U> {
    const MAX_CONTENT: Self = Self::MaxContent;
}
impl<U: Unit> TaffyMinContent for AvailableSpace<U> {
    const MIN_CONTENT: Self = Self::MinContent;
}
impl<U: Unit> TaffyZero for AvailableSpace<U> {
    fn zero() -> Self {
        Self::Definite(U::zero())
    }
}
impl<U: Unit> FromLength for AvailableSpace<U> {
    fn from_length<Input: Into<U> + Copy>(value: Input) -> Self {
        Self::Definite(value.into())
    }
}

impl<U: Unit> AvailableSpace<U> {
    /// Returns true for definite values, else false
    pub fn is_definite(self) -> bool {
        matches!(self, AvailableSpace::Definite(_))
    }

    /// Convert to Option
    /// Definite values become Some(value). Contraints become None.
    pub fn into_option(self) -> Option<U> {
        match self {
            AvailableSpace::Definite(value) => Some(value),
            _ => None,
        }
    }

    /// Return the definite value or a default value
    pub fn unwrap_or(self, default: U) -> U {
        self.into_option().unwrap_or(default)
    }

    /// Return the definite value. Panic is the value is not definite.
    #[track_caller]
    pub fn unwrap(self) -> U {
        self.into_option().unwrap()
    }

    /// Return self if definite or a default value
    pub fn or(self, default: AvailableSpace<U>) -> AvailableSpace<U> {
        match self {
            AvailableSpace::Definite(_) => self,
            _ => default,
        }
    }

    /// Return self if definite or a the result of the default value callback
    pub fn or_else(self, default_cb: impl FnOnce() -> AvailableSpace<U>) -> AvailableSpace<U> {
        match self {
            AvailableSpace::Definite(_) => self,
            _ => default_cb(),
        }
    }

    /// Return the definite value or the result of the default value callback
    pub fn unwrap_or_else(self, default_cb: impl FnOnce() -> U) -> U {
        self.into_option().unwrap_or_else(default_cb)
    }

    /// If passed value is Some then return AvailableSpace::Definite containing that value, else return self
    pub fn maybe_set(self, value: Option<U>) -> AvailableSpace<U> {
        match value {
            Some(value) => AvailableSpace::Definite(value),
            None => self,
        }
    }

    /// If passed value is Some then return AvailableSpace::Definite containing that value, else return self
    pub fn map_definite_value(self, map_function: impl FnOnce(U) -> U) -> AvailableSpace<U> {
        match self {
            AvailableSpace::Definite(value) => AvailableSpace::Definite(map_function(value)),
            _ => self,
        }
    }

    /// Compute free_space given the passed used_space
    pub fn compute_free_space(&self, used_space: U) -> U {
        match self {
            AvailableSpace::MaxContent => U::INFINITY,
            AvailableSpace::MinContent => 0.0,
            AvailableSpace::Definite(available_space) => available_space - used_space,
        }
    }

    /// Compare equality with another AvailableSpace, treating definite values
    /// that are within U::EPSILON of each other as equal
    pub fn is_roughly_equal(self, other: AvailableSpace<U>) -> bool {
        use AvailableSpace::*;
        match (self, other) {
            (Definite(a), Definite(b)) => abs(a - b) < U::EPSILON,
            (MinContent, MinContent) => true,
            (MaxContent, MaxContent) => true,
            _ => false,
        }
    }
}

impl<U: Unit> From<U> for AvailableSpace<U> {
    fn from(value: U) -> Self {
        Self::Definite(value)
    }
}

impl<U: Unit> From<Option<U>> for AvailableSpace<U> {
    fn from(option: Option<U>) -> Self {
        match option {
            Some(value) => Self::Definite(value),
            None => Self::MaxContent,
        }
    }
}

impl<U: Unit> Size<AvailableSpace<U>> {
    /// Convert `Size<AvailableSpace>` into `Size<Option<U>>`
    pub fn into_options(self) -> Size<Option<U>> {
        Size { width: self.width.into_option(), height: self.height.into_option() }
    }

    /// If passed value is Some then return AvailableSpace::Definite containing that value, else return self
    pub fn maybe_set(self, value: Size<Option<U>>) -> Size<AvailableSpace<U>> {
        Size { width: self.width.maybe_set(value.width), height: self.height.maybe_set(value.height) }
    }
}
