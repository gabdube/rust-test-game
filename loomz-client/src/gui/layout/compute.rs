use loomz_shared::RectF32;
use super::super::Gui;
use super::{GuiLayout, GuiLayoutType, GuiLayoutView};

struct LayoutComputeState<'a> {
    base: RectF32,
    views: &'a mut [GuiLayoutView]
}

pub fn compute(gui: &mut Gui) {
    let components = &mut gui.components;

    let base = components.base_view;
    let layout_count = components.layouts.len();
    let mut current_layout = 0;

    let mut state = LayoutComputeState {
        base,
        views: &mut components.views,
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
        let view = &mut state.views[index];
        view.position.x = (state.base.width() - view.size.width) / 2.0;
        view.position.y = y_offset;
        y_offset += view.size.height;
        index += 1;
    }
}
