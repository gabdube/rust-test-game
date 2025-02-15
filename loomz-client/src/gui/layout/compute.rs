use loomz_shared::RectF32;
use super::super::Gui;
use super::{GuiLayout, GuiLayoutType, GuiLayoutPosition, GuiLayoutItem};

struct LayoutComputeState<'a> {
    layout_items: &'a mut [GuiLayoutItem],
    layouts: &'a mut [GuiLayout],
    item_index: i32,
    layout_index: i32,
    view: RectF32,
}

/// Compute the position of the items in the layout.
/// Sizing the layout and the layout item is done a build time in `builder`
pub fn compute(gui: &mut Gui) {
    // Layouts may be empty if `gui.resize` was called on an uninitialized gui
    let root_children_count = gui.layouts.get(0).map(|layout| layout.children_count ).unwrap_or(0);
    if root_children_count == 0 {
        return;
    }

    let mut state = LayoutComputeState {
        layout_items: &mut gui.layout_items,
        layouts: &mut gui.layouts,
        item_index: 0,
        layout_index: -1,
        view: gui.base_view,
    };

    compute_layout(&mut state);
}

fn compute_layout(state: &mut LayoutComputeState) {
    state.layout_index += 1;

    let layout_index = state.layout_index as usize;
    let layout = state.layouts[layout_index];

    match layout.ty {
        GuiLayoutType::VBox => vbox_layout(state, layout),
        GuiLayoutType::HBox => hbox_layout(state, layout),
    }
}

fn vbox_layout(state: &mut LayoutComputeState, layout: GuiLayout) {
    let view = state.view;
    let view_width = view.width();
    let view_height = view.height();
    let mut offset_x = 0.0; 
    let mut offset_y = 0.0; 

    match layout.position {
        GuiLayoutPosition::TopLeft => {},
        GuiLayoutPosition::Center => {
            offset_x = view.left + ((view_width - layout.width) * 0.5);
            offset_y = view.top + ((view_height - layout.height) * 0.5);
        }
    }

    for _ in 0..layout.children_count {
        let item_index = state.item_index as usize;
        let mut item = state.layout_items[item_index];

        item.position.x = offset_x;
        item.position.y = offset_y;
        offset_y += item.size.height;

        state.layout_items[item_index] = item;
        state.item_index += 1;

        if item.has_layout {
            state.view = RectF32::from_position_and_size(item.position, item.size);
            compute_layout(state);
        }
    }
}

fn hbox_layout(state: &mut LayoutComputeState, layout: GuiLayout) {
    let view = state.view;
    let view_width = view.width();
    let view_height = view.height();
    let mut offset_x = 0.0;
    let mut offset_y = 0.0;

    match layout.position {
        GuiLayoutPosition::TopLeft => {},
        GuiLayoutPosition::Center => {
            offset_x =  view.left + ((view_width - layout.width) * 0.5);
            offset_y =  view.top + ((view_height - layout.height) * 0.5);
        }
    }

    for _ in 0..layout.children_count {
        let item_index = state.item_index as usize;
        let mut item = state.layout_items[item_index];

        item.position.x = offset_x;
        item.position.y = offset_y;
        offset_x += item.size.width;

        state.layout_items[item_index] = item;
        state.item_index += 1;

        if item.has_layout {
            state.view = RectF32::from_position_and_size(item.position, item.size);
            compute_layout(state);
        }
    }
}

#[cfg(test)]
mod tests {
    use loomz_shared::{LoomzApi, RectF32, PositionF32, SizeF32, rect, rgb};
    use super::super::super::{Gui, GuiStyleState};
    use super::super::{GuiLayoutType::VBox, GuiLayoutPosition};
    

    macro_rules! assert_layout {
        ($layout:expr, $ty:expr, $w:literal, $h:literal, $children_count:literal) => {
            {
                let layout = &$layout;
                assert_eq!(layout.ty, $ty, "Mismatched layout type");
                assert_eq!(layout.width, $w, "Mismatched layout width");
                assert_eq!(layout.height, $h, "Mismatched layout height");
                assert_eq!(layout.children_count, $children_count, "Mismatched layout children_count");
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
 
        let view_size = SizeF32 { width: 1000.0, height: 1000.0 };
        let api = LoomzApi::init(view_size).unwrap();
        let view = RectF32 { left: 0.0, top: 0.0, right: view_size.width, bottom: view_size.height };
        let mut gui = Gui::default();

        let style_result = gui.build_style(&api, |style| {
            style.root_layout(VBox, GuiLayoutPosition::Center);
            style.label("item", GuiStyleState::Base, "bubblegum", 100.0, rgb(204, 142, 100));
            style.frame("frame", GuiStyleState::Base, "gui", rect(0.0, 0.0, 2.0, 2.0), rgb(27, 19, 15));
        });

        assert!(style_result.is_ok(), "Gui styling failed: {:?}", style_result);

        let build_result = gui.build(&api, &view, |gui| {
            gui.layout(VBox, GuiLayoutPosition::Center);
            gui.layout_item(300.0, 300.0);
            gui.frame("frame", |gui| {
                gui.layout_item(200.0, 200.0);
                gui.frame("frame", |_| {});
            }); 

            gui.layout(VBox, GuiLayoutPosition::Center);
            gui.layout_item(300.0, 300.0);
            gui.frame("frame", |gui| {
                gui.layout_item(200.0, 200.0);
                gui.frame("frame", |_| {});
            });
        });

        assert!(build_result.is_ok(), "Gui build failed: {:?}", build_result);
 
        let items = &gui.layout_items;
        let layouts = &gui.layouts;

        assert_eq!(layouts.len(), 5);
        assert_eq!(items.len(), 4);

        assert_layout!(layouts[0], VBox, 300.0, 600.0, 2);
        // assert_layout!(layouts[0], VBox, 0.0, 0.0, 0, u32::MAX);
        // assert_layout!(layouts[1], VBox, 0.0, 0.0, 1, u32::MAX);

        assert_layout_item!(items[0], 350.0, 200.0, 300.0, 300.0);
        assert_layout_item!(items[1], 400.0, 250.0, 200.0, 200.0);
        assert_layout_item!(items[2], 350.0, 500.0, 300.0, 300.0);
        assert_layout_item!(items[3], 400.0, 550.0, 200.0, 200.0);
    }

}
