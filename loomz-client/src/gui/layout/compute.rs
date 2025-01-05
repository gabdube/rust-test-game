use loomz_shared::RectF32;
use super::super::Gui;
use super::{GuiLayout, GuiLayoutType, GuiLayoutItem};

struct LayoutComputeState<'a> {
    base: RectF32,
    layout_items: &'a mut [GuiLayoutItem]
}

pub fn compute(gui: &mut Gui) {
    let components = &mut gui.components;

    let base = components.base_view;
    let layout_count = components.layouts.len();
    let mut current_layout = 0;

    let mut state = LayoutComputeState {
        base,
        layout_items: &mut components.layout_items,
    };

    while current_layout != layout_count {
        let layout = components.layouts[current_layout];
        match layout.ty {
            GuiLayoutType::VBox => vbox_layout(&mut state, &layout)
        }
        current_layout += 1;
    }
}

fn vbox_layout(state: &mut LayoutComputeState, vbox: &GuiLayout) {
    let mut index = vbox.first_component as usize;
    let last = index + (vbox.component_count as usize);

    let mut y_offset = (state.base.height() - vbox.height) / 2.0;

    while index != last {
        let layout_item = &mut state.layout_items[index];
        layout_item.position.x = (state.base.width() - layout_item.size.width) / 2.0;
        layout_item.position.y = y_offset;
        y_offset += layout_item.size.height;
        index += 1;
    }
}

#[cfg(test)]
mod tests {
    use loomz_shared::{LoomzApi, RectF32, PositionF32, SizeF32, size, rect, rgb};
    use super::super::GuiLayoutType;
    use super::super::super::Gui;

    macro_rules! assert_layout {
        ($layout:expr, $ty:expr, $w:literal, $h:literal) => {
            {
                let layout = &$layout;
                assert_eq!(layout.ty, $ty);
                assert_eq!(layout.width, $w);
                assert_eq!(layout.height, $h);
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
            gui.font_style("item1", "bubblegum", 100.0, rgb(255, 255, 255));
            gui.frame_style("main", "gui", rect(0.0, 0.0, 2.0, 2.0), rgb(0, 0, 0));

            gui.vbox_layout(0.0, 300.0);
            gui.frame("main", size(300.0, 300.0), |gui| {
                gui.label("Start", "item1");
                gui.label("Debug", "item1");
                gui.label("Exit", "item1");
            })
        });

        assert!(build_result.is_ok(), "Gui build failed: {:?}", build_result);

        let components = &gui.components;
        let layouts = &components.layouts;
        let items = &components.layout_items;

        assert_eq!(layouts.len(), 2);
        assert_eq!(items.len(), 4);

        assert_layout!(layouts[0], GuiLayoutType::VBox, 0.0, 300.0);
        assert_layout!(layouts[1], GuiLayoutType::VBox, 0.0, 0.0);

        assert_layout_item!(items[0], 350.0, 350.0, 300.0, 300.0);
        assert_layout_item!(items[1], 350.0, 350.0, 300.0, 300.0);

    }

}
