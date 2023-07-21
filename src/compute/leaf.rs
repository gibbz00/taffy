//! Computes size using styles and measure functions

use num_traits::real::Real;

use crate::geometry::{Line, Point, Size, Unit};
use crate::style::{AvailableSpace, Display, Overflow, Position, Style};
use crate::tree::{CollapsibleMarginSet, Measurable};
use crate::tree::{SizeBaselinesAndMargins, SizingMode};
use crate::util::MaybeMath;
use crate::util::{MaybeResolve, ResolveOrZero};

#[cfg(feature = "debug")]
use crate::util::debug::NODE_LOGGER;

/// Perform full layout on a leaf node
pub(crate) fn perform_layout<U: Unit>(
    style: &Style<U>,
    measurable: Option<&impl Measurable<U>>,
    known_dimensions: Size<Option<U>>,
    parent_size: Size<Option<U>>,
    available_space: Size<AvailableSpace<U>>,
    sizing_mode: SizingMode,
    _vertical_margins_are_collapsible: Line<bool>,
) -> SizeBaselinesAndMargins<U> {
    compute(style, measurable, known_dimensions, parent_size, available_space, sizing_mode)
}

/// Measure a leaf node's size
pub(crate) fn measure_size<U: Unit>(
    style: &Style<U>,
    measurable: Option<&impl Measurable<U>>,
    known_dimensions: Size<Option<U>>,
    parent_size: Size<Option<U>>,
    available_space: Size<AvailableSpace<U>>,
    sizing_mode: SizingMode,
    _vertical_margins_are_collapsible: Line<bool>,
) -> Size<U> {
    compute(style, measurable, known_dimensions, parent_size, available_space, sizing_mode).size
}

/// Compute the size of a leaf node (node with no children)
pub fn compute<U: Unit>(
    style: &Style<U>,
    measurable: Option<&impl Measurable<U>>,
    known_dimensions: Size<Option<U>>,
    parent_size: Size<Option<U>>,
    available_space: Size<AvailableSpace<U>>,
    sizing_mode: SizingMode,
) -> SizeBaselinesAndMargins<U> {
    // Resolve node's preferred/min/max sizes (width/heights) against the available space (percentages resolve to pixel values)
    // For ContentSize mode, we pretend that the node has no size styles as these should be ignored.
    let (node_size, node_min_size, node_max_size, aspect_ratio) = match sizing_mode {
        SizingMode::ContentSize => {
            let node_size = known_dimensions;
            let node_min_size = Size::NONE;
            let node_max_size = Size::NONE;
            (node_size, node_min_size, node_max_size, None)
        }
        SizingMode::InherentSize => {
            let aspect_ratio = style.aspect_ratio;
            let style_size = style.size.maybe_resolve(parent_size).maybe_apply_aspect_ratio(aspect_ratio);
            let style_min_size = style.min_size.maybe_resolve(parent_size).maybe_apply_aspect_ratio(aspect_ratio);
            let style_max_size = style.max_size.maybe_resolve(parent_size);

            let node_size = known_dimensions.or(style_size);
            (node_size, style_min_size, style_max_size, aspect_ratio)
        }
    };

    // Note: both horizontal and vertical percentage padding/borders are resolved against the container's inline size (i.e. width).
    // This is not a bug, but is how CSS is specified (see: https://developer.mozilla.org/en-US/docs/Web/CSS/padding#values)
    let padding = style.padding.resolve_or_zero(parent_size.width);
    let border = style.border.resolve_or_zero(parent_size.width);
    let padding_border = padding + border;

    // Scrollbar gutters are reserved when the `overflow` property is set to `Overflow::Scroll`.
    // However, the axis are switched (transposed) because a node that scrolls vertically needs
    // *horizontal* space to be reserved for a scrollbar
    let scrollbar_gutter = style.overflow.transpose().map(|overflow| match overflow {
        Overflow::Scroll => style.scrollbar_width,
        _ => 0.0,
    });
    // TODO: make side configurable based on the `direction` property
    let mut content_box_inset = padding_border;
    content_box_inset.right += scrollbar_gutter.x;
    content_box_inset.bottom += scrollbar_gutter.y;

    #[cfg(feature = "block_layout")]
    let is_block = style.display == Display::Block;
    #[cfg(not(feature = "block_layout"))]
    let is_block = false;

    let has_styles_preventing_being_collapsed_through = !is_block
        || style.overflow.x.is_scroll_container()
        || style.overflow.y.is_scroll_container()
        || style.position == Position::Absolute
        || padding.top > 0.0
        || padding.bottom > 0.0
        || border.top > 0.0
        || border.bottom > 0.0;

    #[cfg(feature = "debug")]
    NODE_LOGGER.log("LEAF");
    #[cfg(feature = "debug")]
    NODE_LOGGER.labelled_debug_log("node_size", node_size);
    #[cfg(feature = "debug")]
    NODE_LOGGER.labelled_debug_log("min_size ", node_min_size);
    #[cfg(feature = "debug")]
    NODE_LOGGER.labelled_debug_log("max_size ", node_max_size);

    // Return early if both width and height are known
    if let Size { width: Some(width), height: Some(height) } = node_size {
        let size = Size { width, height }
            .maybe_clamp(node_min_size, node_max_size)
            .maybe_max(padding_border.sum_axes().map(Some));
        return SizeBaselinesAndMargins {
            size,
            first_baselines: Point::NONE,
            top_margin: CollapsibleMarginSet::zero(),
            bottom_margin: CollapsibleMarginSet::zero(),
            margins_can_collapse_through: !has_styles_preventing_being_collapsed_through
                && size.height == 0.0
                && measurable.is_none(),
        };
    };

    if let Some(measurable) = measurable {
        // Compute available space
        let available_space = Size {
            width: available_space.width.maybe_set(node_size.width).maybe_set(node_max_size.width).map_definite_value(
                |size| {
                    size.maybe_clamp(node_min_size.width, node_max_size.width) - content_box_inset.horizontal_axis_sum()
                },
            ),
            height: available_space
                .height
                .maybe_set(node_size.height)
                .maybe_set(node_max_size.height)
                .map_definite_value(|size| {
                    size.maybe_clamp(node_min_size.height, node_max_size.height) - content_box_inset.vertical_axis_sum()
                }),
        };

        // Measure node
        let measured_size = measurable.measure(known_dimensions, available_space);
        let clamped_size =
            node_size.unwrap_or(measured_size + content_box_inset.sum_axes()).maybe_clamp(node_min_size, node_max_size);
        let size = Size {
            width: clamped_size.width,
            height: Real::max(clamped_size.height, aspect_ratio.map(|ratio| clamped_size.width / ratio).unwrap_or(0.0)),
        };
        let size = size.maybe_max(padding_border.sum_axes().map(Some));

        return SizeBaselinesAndMargins {
            size,
            first_baselines: Point::NONE,
            top_margin: CollapsibleMarginSet::zero(),
            bottom_margin: CollapsibleMarginSet::zero(),
            margins_can_collapse_through: !has_styles_preventing_being_collapsed_through
                && size.height == 0.0
                && measured_size.height == 0.0,
        };
    }

    let size = Size {
        width: node_size
            .width
            // .unwrap_or(0.0) + padding.horizontal_axis_sum() + border.horizontal_axis_sum(), // content-box
            .unwrap_or(content_box_inset.horizontal_axis_sum()) // border-box
            .maybe_clamp(node_min_size.width, node_max_size.width)
            .maybe_max(padding_border.horizontal_axis_sum().into()),
        height: node_size
            .height
            // .unwrap_or(0.0) + padding.vertical_axis_sum() + border.vertical_axis_sum(), // content-box
            .unwrap_or(content_box_inset.vertical_axis_sum()) // border-box
            .maybe_clamp(node_min_size.height, node_max_size.height)
            .maybe_max(padding_border.vertical_axis_sum().into()),
    };

    let size = Size {
        width: Real::max(size.width, aspect_ratio.map(|ratio| size.height * ratio).unwrap_or(0.0)),
        height: Real::max(size.height, aspect_ratio.map(|ratio| size.width / ratio).unwrap_or(0.0)),
    };

    SizeBaselinesAndMargins {
        size,
        first_baselines: Point::NONE,
        top_margin: CollapsibleMarginSet::zero(),
        bottom_margin: CollapsibleMarginSet::zero(),
        margins_can_collapse_through: !has_styles_preventing_being_collapsed_through && size.height == 0.0,
    }
}
