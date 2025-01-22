use loomz_shared::{rect, rgb};
use crate::gui::{GuiStyleBuilder, GuiStyleState};

pub fn main_panel_style(style: &mut GuiStyleBuilder) {
    style.label("menu_item", GuiStyleState::Base, "bubblegum", 90.0, rgb(71, 43, 26));
    style.label("menu_item", GuiStyleState::Hovered, "bubblegum", 90.0, rgb(71, 26, 26));
    style.label("menu_item", GuiStyleState::Selected, "bubblegum", 90.0, rgb(110, 34, 34));
    style.frame("main_panel_style", GuiStyleState::Base, "gui", rect(0.0, 0.0, 2.0, 2.0), rgb(24, 18, 15));
}
