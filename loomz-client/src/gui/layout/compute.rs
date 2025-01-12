use loomz_shared::RectF32;
use super::super::Gui;
use super::{GuiLayout, GuiLayoutType, GuiLayoutItem};

struct LayoutComputeState<'a> {
    item_index: usize,
    layout_items: &'a mut [GuiLayoutItem],
    view: RectF32,
}

/// Compute the position of the items in the layout.
/// Sizing the layout and the layout item is done a build time in `builder`
pub fn compute(gui: &mut Gui) {
    let components = &mut gui.components;
    let view = components.base_view;
    let root = components.root_layout;

    let mut state = LayoutComputeState {
        item_index: 0,
        layout_items: &mut components.layout_items,
        view,
    };

    compute_layout(&mut state, root);
}

fn compute_layout(state: &mut LayoutComputeState, layout: GuiLayout) {
    if layout.children_count == 0 {
        return;
    }

    match layout.ty {
        GuiLayoutType::VBox => vbox_layout(state, layout),
    }
}

fn vbox_layout(state: &mut LayoutComputeState, layout: GuiLayout) {
    let view_width = state.view.width();
    let view_height = state.view.height();

    let offset_x = state.view.left + ((view_width - layout.width) * 0.5);
    let mut offset_y = state.view.top + ((view_height - layout.height) * 0.5);

    for _ in 0..layout.children_count {
        let mut item = state.layout_items[state.item_index];
        item.position.x = offset_x;
        item.position.y = offset_y;
        offset_y += item.size.height;

        state.layout_items[state.item_index] = item;
        state.item_index += 1;
    }
}

#[cfg(test)]
mod tests {
    use loomz_shared::{LoomzApi, RectF32, PositionF32, SizeF32, size, rect, rgb};
    use super::super::GuiLayoutType::VBox;
    use super::super::super::Gui;

    macro_rules! assert_layout {
        ($layout:expr, $ty:expr, $w:literal, $h:literal, $first:literal, $last:expr) => {
            {
                let layout = &$layout;
                assert_eq!(layout.ty, $ty, "Mismatched layout type");
                // assert_eq!(layout.width, $w, "Mismatched layout width");
                // assert_eq!(layout.height, $h, "Mismatched layout height");
                // assert_eq!(layout.first_component, $first, "Mismatched layout first component index");
                // assert_eq!(layout.last_component, $last, "Mismatched last component index");
            }
        };
    }

    macro_rules! assert_layout_item {
        ($item:expr, $x:literal, $y:literal, $w:literal, $h:literal) => {
            {
                let item = &$item;
                assert_eq!(item.position, PositionF32 { x: $x, y: $y });
                assert_eq!(item.size, SizeF32 { width: $w, height: $h });
            }
        };
    }

    fn set_dir() {
        ::std::env::set_current_dir(::std::fs::canonicalize("..").unwrap()).unwrap();
    }

    #[test]
    fn test_layout() {
        set_dir();
 
        let api = LoomzApi::init().unwrap();
        let view = RectF32 { left: 0.0, top: 0.0, right: 1000.0, bottom: 1000.0 };
        let mut gui = Gui::default();
        let build_result = gui.build(&api, &view, |gui| {
            gui.font_style("item1", "bubblegum", 100.0, rgb(204, 142, 100));
            gui.root_layout(VBox);

            gui.layout(VBox);
            gui.layout_item(300.0, 300.0);
            gui.frame_style("gui", rect(0.0, 0.0, 2.0, 2.0), rgb(27, 19, 15));
            gui.frame(size(300.0, 300.0), |gui| {
            }); 

            gui.layout(VBox);
            gui.layout_item(300.0, 300.0);
            gui.frame_style("gui", rect(0.0, 0.0, 2.0, 2.0), rgb(117, 55, 24));
            gui.frame(size(300.0, 300.0), |gui| {
            });
        });

        assert!(build_result.is_ok(), "Gui build failed: {:?}", build_result);

        let components = &gui.components;
        //let layouts = &components.layouts;
        let items = &components.layout_items;

        //assert_eq!(layouts.len(), 2);
        assert_eq!(items.len(), 2);

        assert_layout!(components.root_layout, VBox, 0.0, 0.0, 0, 1);
        // assert_layout!(layouts[0], VBox, 0.0, 0.0, 0, u32::MAX);
        // assert_layout!(layouts[1], VBox, 0.0, 0.0, 1, u32::MAX);

        assert_layout_item!(items[0], 350.0, 200.0, 300.0, 300.0);
        assert_layout_item!(items[1], 350.0, 500.0, 300.0, 300.0);

    }

}
