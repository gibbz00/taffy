//! The layout algorithms themselves

pub(crate) mod common;
pub(crate) mod leaf;

pub use leaf::compute;

#[cfg(feature = "block_layout")]
pub(crate) mod block;

#[cfg(feature = "flexbox")]
pub(crate) mod flexbox;

#[cfg(feature = "grid")]
pub(crate) mod grid;

use crate::geometry::{Line, Size, Unit};
use crate::style::AvailableSpace;
use crate::tree::{Layout, LayoutTree, NodeId, SizeBaselinesAndMargins, SizingMode};
use core::marker::PhantomData;

#[cfg(feature = "block_layout")]
pub use self::block::BlockAlgorithm;

#[cfg(feature = "flexbox")]
pub use self::flexbox::FlexboxAlgorithm;

#[cfg(feature = "grid")]
pub use self::grid::CssGridAlgorithm;

#[cfg(feature = "taffy_tree")]
pub(crate) mod taffy_tree;

/// A common interface that all Taffy layout algorithms conform to
pub trait LayoutAlgorithm<U: Unit = f32> {
    /// The name of the algorithm (mainly used for debug purposes)
    const NAME: &'static str;

    /// Compute the size of the node given the specified constraints
    fn measure_size(
        tree: &mut impl LayoutTree<U>,
        node: NodeId,
        known_dimensions: Size<Option<U>>,
        parent_size: Size<Option<U>>,
        available_space: Size<AvailableSpace<U>>,
        sizing_mode: SizingMode,
        vertical_margins_are_collapsible: Line<bool>,
    ) -> Size<U>;

    /// Perform a full layout on the node given the specified constraints
    fn perform_layout(
        tree: &mut impl LayoutTree<U>,
        node: NodeId,
        known_dimensions: Size<Option<U>>,
        parent_size: Size<Option<U>>,
        available_space: Size<AvailableSpace<U>>,
        sizing_mode: SizingMode,
        vertical_margins_are_collapsible: Line<bool>,
    ) -> SizeBaselinesAndMargins;
}

/// The public interface to Taffy's hidden node algorithm implementation
pub struct HiddenAlgorithm<U: Unit = f32> {
    unit: PhantomData<U>,
}
impl<U: Unit> LayoutAlgorithm for HiddenAlgorithm<U> {
    const NAME: &'static str = "NONE";

    fn perform_layout(
        tree: &mut impl LayoutTree<U>,
        node: NodeId,
        _known_dimensions: Size<Option<U>>,
        _parent_size: Size<Option<U>>,
        _available_space: Size<AvailableSpace<U>>,
        _sizing_mode: SizingMode,
        _vertical_margins_are_collapsible: Line<bool>,
    ) -> SizeBaselinesAndMargins {
        perform_hidden_layout(tree, node);
        SizeBaselinesAndMargins::hidden()
    }

    fn measure_size(
        _tree: &mut impl LayoutTree<U>,
        _node: NodeId,
        _known_dimensions: Size<Option<U>>,
        _parent_size: Size<Option<U>>,
        _available_space: Size<AvailableSpace<U>>,
        _sizing_mode: SizingMode,
        _vertical_margins_are_collapsible: Line<bool>,
    ) -> Size<U> {
        Size::ZERO
    }
}

/// Creates a layout for this node and its children, recursively.
/// Each hidden node has zero size and is placed at the origin
fn perform_hidden_layout<U: Unit>(tree: &mut impl LayoutTree<U>, node: NodeId) {
    /// Recursive function to apply hidden layout to all descendents
    fn perform_hidden_layout_inner<U: Unit>(tree: &mut impl LayoutTree<U>, node: NodeId, order: u32) {
        *tree.layout_mut(node) = Layout::with_order(order);
        for order in 0..tree.child_count(node) {
            perform_hidden_layout_inner(tree, tree.child(node, order), order as _);
        }
    }

    for order in 0..tree.child_count(node) {
        perform_hidden_layout_inner(tree, tree.child(node, order), order as _);
    }
}

#[cfg(test)]
mod tests {
    use super::perform_hidden_layout;
    use crate::geometry::{Point, Size};
    use crate::style::{Display, Style};
    use crate::Taffy;

    #[test]
    fn hidden_layout_should_hide_recursively() {
        let mut taffy = Taffy::new();

        let style: Style = Style { display: Display::Flex, size: Size::from_lengths(50.0, 50.0), ..Default::default() };

        let grandchild_00 = taffy.new_leaf(style.clone());
        let grandchild_01 = taffy.new_leaf(style.clone());
        let child_00 = taffy.new_with_children(style.clone(), &[grandchild_00, grandchild_01]);

        let grandchild_02 = taffy.new_leaf(style.clone());
        let child_01 = taffy.new_with_children(style.clone(), &[grandchild_02]);

        let root = taffy.new_with_children(
            Style { display: Display::None, size: Size::from_lengths(50.0, 50.0), ..Default::default() },
            &[child_00, child_01],
        );

        perform_hidden_layout(&mut taffy, root);

        // Whatever size and display-mode the nodes had previously,
        // all layouts should resolve to ZERO due to the root's DISPLAY::NONE
        for (node, _) in taffy.nodes.iter().filter(|(node, _)| *node != root.into()) {
            let layout = taffy.layout(node.into());
            assert_eq!(layout.size, Size::zero());
            assert_eq!(layout.location, Point::zero());
        }
    }
}
