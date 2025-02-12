use loomz_shared::inputs::keys;
use loomz_shared::{CommonError, rect};
use crate::{LoomzClient, GameState, GameInputFlags};

const RETURN_EDITOR: u64 = 200;
const EXIT_EDITOR: u64 = 201;

impl LoomzClient {

    pub(crate) fn init_editor(&mut self) -> Result<(), CommonError> {
        self.init_editor_gui()?;
        self.init_editor_terrain()?;
        self.api.world().toggle_world(true);
        self.state = GameState::Editor;
        Ok(())
    }

    pub(crate) fn editor(&mut self) -> Result<(), CommonError> {
        self.editor_global_update();

        if self.gui.visible() {
            self.editor_gui_updates();
            self.editor_gui_events()?;
        } else {
            self.editor_updates();
        }

        Ok(())
    }

    fn editor_global_update(&mut self) {
        let new_inputs = match self.api.read_inputs() {
            Some(inputs) => inputs,
            None => { return; }
        };

        if let Some(new_size) = new_inputs.screen_size() {
            self.terrain.set_view(0.0, 0.0, new_size.width, new_size.height);
            self.terrain.sync(&self.api);
        }

        if let Some(new_mouse) = new_inputs.mouse_buttons() {
            match new_mouse.right_button_down() {
                true => { self.input_flags.insert(GameInputFlags::DRAGGING_VIEW); },
                false => { self.input_flags.remove(GameInputFlags::DRAGGING_VIEW); }
            }
        }

        // size must be outside of `keystate` to prevent a deadlock
        let size = new_inputs.screen_size_value();
        if let Some(keystate) = new_inputs.keystate() {
            if keystate.just_pressed(keys::ESC) {
                self.gui.resize(&self.api, &rect(0.0, 0.0, size.width, size.height));
                self.gui.toggle(&self.api, !self.gui.visible());
            }
        }

        ()
    }

    fn editor_gui_updates(&mut self) {
        let new_inputs = match self.api.read_inputs() {
            Some(inputs) => inputs,
            None => { return; }
        };

        let mut gui_updates = crate::gui::GuiUpdates::default();

        if let Some(cursor_position) = new_inputs.cursor_position() {
            gui_updates.cursor_position = Some(cursor_position.as_f32());
        }

        if let Some(buttons) = new_inputs.mouse_buttons() {
            gui_updates.left_mouse_down = Some(buttons.left_button_down());
        }

        if let Some(new_size) = new_inputs.screen_size() {
            gui_updates.view = Some(rect(0.0, 0.0, new_size.width, new_size.height));
        }

        self.gui.update(&self.api, &gui_updates);
    }

    fn editor_gui_events(&mut self) -> Result<(), CommonError> { 
        let mut ret = false;
        let mut ext = false;

        for event in self.gui.drain_events() {
            match event {
                RETURN_EDITOR => { ret = true },
                EXIT_EDITOR => { ext = true; },
                _ => {}
            }
        }

        if ret {
            self.gui.toggle(&self.api, false);
        } else if ext {
            self.init_main_menu()?;
        }

        Ok(())
    }

    fn editor_updates(&mut self) {
        if self.input_flags.contains(GameInputFlags::DRAGGING_VIEW) {
            let delta = self.api.inputs().cursor_position_delta();
            self.terrain.move_view(-delta.x as f32, -delta.y as f32);
            self.terrain.sync(&self.api);
        }
    }

    fn init_editor_gui(&mut self) -> Result<(), CommonError> {
        use crate::gui::{GuiLayoutType, GuiLabelCallback};

        let screen_size = self.api.inputs().screen_size_value();
        let view = loomz_shared::RectF32::from_size(screen_size);

        self.gui.toggle(&self.api, false);

        self.gui.build_style(&self.api, |style| {
            style.root_layout(GuiLayoutType::VBox);
            super::shared::main_panel_style(style);
        })?;

        self.gui.build(&self.api, &view, |gui| {
            gui.layout(GuiLayoutType::VBox);
            gui.layout_item(400.0, 300.0);
            gui.frame("main_panel_style", |gui| {
                gui.layout_item(300.0, 105.0);

                gui.label_callback(GuiLabelCallback::Click, RETURN_EDITOR);
                gui.label("Continue", "menu_item");

                gui.label_callback(GuiLabelCallback::Click, EXIT_EDITOR);
                gui.label("Exit", "menu_item");
            });
        })?;

        Ok(())
    }

    fn init_editor_terrain(&mut self) -> Result<(), CommonError> {
        let screen_size = self.api.inputs().screen_size_value();
        self.terrain.set_view(0.0, 0.0, screen_size.width, screen_size.height);
        self.terrain.set_world_size(16, 16);
        //self.terrain.set_cells(0, 0, 31, 1, &[loomz_shared::api::TerrainType::Sand; 31]);
        self.terrain.sync(&self.api);
        Ok(())
    }

}
