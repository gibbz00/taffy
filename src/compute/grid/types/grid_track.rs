//! Contains GridTrack used to represent a single grid track (row/column) during layout
use crate::{
    geometry::Unit,
    style::{LengthPercentage, MaxTrackSizingFunction, MinTrackSizingFunction},
};
use num_traits::real::Real;

/// Whether a GridTrack represents an actual track or a gutter.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(in super::super) enum GridTrackKind {
    /// Track is an actual track
    Track,
    /// Track is a gutter (aka grid line) (aka gap)
    Gutter, // { name: Option<u16> },
}

/// Internal sizing information for a single grid track (row/column)
/// Gutters between tracks are sized similarly to actual tracks, so they
/// are also represented by this struct
#[derive(Debug, Clone)]
pub(in super::super) struct GridTrack<U: Unit = f32> {
    #[allow(dead_code)] // Used in tests + may be useful in future
    /// Whether the track is a full track, a gutter, or a placeholder that has not yet been initialised
    pub kind: GridTrackKind,

    /// Whether the track is a collapsed track/gutter. Collapsed tracks are effectively treated as if
    /// they don't exist for the purposes of grid sizing. Gutters between collapsed tracks are also collapsed.
    pub is_collapsed: bool,

    /// The minimum track sizing function of the track
    pub min_track_sizing_function: MinTrackSizingFunction<U>,

    /// The maximum track sizing function of the track
    pub max_track_sizing_function: MaxTrackSizingFunction<U>,

    /// The distance of the start of the track from the start of the grid container
    pub offset: U,

    /// The size (width/height as applicable) of the track
    pub base_size: U,

    /// A temporary scratch value when sizing tracks
    /// Note: can be infinity
    pub growth_limit: U,

    /// A temporary scratch value when sizing tracks. Is used as an additional amount to add to the
    /// estimate for the available space in the opposite axis when content sizing items
    pub content_alignment_adjustment: U,

    /// A temporary scratch value when "distributing space" to avoid clobbering planned increase variable
    pub item_incurred_increase: U,
    /// A temporary scratch value when "distributing space" to avoid clobbering the main variable
    pub base_size_planned_increase: U,
    /// A temporary scratch value when "distributing space" to avoid clobbering the main variable
    pub growth_limit_planned_increase: U,
    /// A temporary scratch value when "distributing space"
    /// See: https://www.w3.org/TR/css3-grid-layout/#infinitely-growable
    pub infinitely_growable: bool,
}

impl<U: Unit> GridTrack<U> {
    /// GridTrack constructor with all configuration parameters for the other constructors exposed
    fn new_with_kind(
        kind: GridTrackKind,
        min_track_sizing_function: MinTrackSizingFunction<U>,
        max_track_sizing_function: MaxTrackSizingFunction<U>,
    ) -> Self {
        GridTrack {
            kind,
            is_collapsed: false,
            min_track_sizing_function,
            max_track_sizing_function,
            offset: U::zero(),
            base_size: U::zero(),
            growth_limit: U::zero(),
            content_alignment_adjustment: U::zero(),
            item_incurred_increase: U::zero(),
            base_size_planned_increase: U::zero(),
            growth_limit_planned_increase: U::zero(),
            infinitely_growable: false,
        }
    }

    /// Create new GridTrack representing an actual track (not a gutter)
    pub fn new(
        min_track_sizing_function: MinTrackSizingFunction<U>,
        max_track_sizing_function: MaxTrackSizingFunction<U>,
    ) -> Self {
        Self::new_with_kind(GridTrackKind::Track, min_track_sizing_function, max_track_sizing_function)
    }

    /// Create a new GridTrack representing a gutter
    pub fn gutter(size: LengthPercentage<U>) -> Self {
        Self::new_with_kind(
            GridTrackKind::Gutter,
            MinTrackSizingFunction::Fixed(size),
            MaxTrackSizingFunction::Fixed(size),
        )
    }

    /// Mark a GridTrack as collapsed. Also sets both of the track's sizing functions
    /// to fixed zero-sized sizing functions.
    pub fn collapse(&mut self) {
        self.is_collapsed = true;
        self.min_track_sizing_function = MinTrackSizingFunction::Fixed(LengthPercentage::Length(U::zero()));
        self.max_track_sizing_function = MaxTrackSizingFunction::Fixed(LengthPercentage::Length(U::zero()));
    }

    #[inline(always)]
    /// Returns true if the track is flexible (has a Flex MaxTrackSizingFunction), else false.
    pub fn is_flexible(&self) -> bool {
        matches!(self.max_track_sizing_function, MaxTrackSizingFunction::Fraction(_))
    }

    #[inline(always)]
    /// Returns true if the track is flexible (has a Flex MaxTrackSizingFunction), else false.
    pub fn uses_percentage(&self) -> bool {
        self.min_track_sizing_function.uses_percentage() || self.max_track_sizing_function.uses_percentage()
    }

    #[inline(always)]
    /// Returns true if the track has an intrinsic min and or max sizing function
    pub fn has_intrinsic_sizing_function(&self) -> bool {
        self.min_track_sizing_function.is_intrinsic() || self.max_track_sizing_function.is_intrinsic()
    }

    #[inline]
    /// Returns true if the track is flexible (has a Flex MaxTrackSizingFunction), else false.
    pub fn fit_content_limit(&self, axis_available_grid_space: Option<U>) -> U {
        match self.max_track_sizing_function {
            MaxTrackSizingFunction::FitContent(LengthPercentage::Length(limit)) => limit,
            MaxTrackSizingFunction::FitContent(LengthPercentage::Percent(fraction)) => {
                match axis_available_grid_space {
                    Some(space) => space * fraction,
                    None => U::INFINITY,
                }
            }
            _ => U::INFINITY,
        }
    }

    #[inline]
    /// Returns true if the track is flexible (has a Flex MaxTrackSizingFunction), else false.
    pub fn fit_content_limited_growth_limit(&self, axis_available_grid_space: Option<U>) -> U {
        Real::min(self.growth_limit, self.fit_content_limit(axis_available_grid_space))
    }

    #[inline]
    /// Returns the track's flex factor if it is a flex track, else 0.
    pub fn flex_factor(&self) -> U {
        match self.max_track_sizing_function {
            MaxTrackSizingFunction::Fraction(flex_factor) => flex_factor,
            _ => U::zero(),
        }
    }
}
