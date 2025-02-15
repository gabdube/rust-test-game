use loomz_shared::inputs::keys;
use loomz_shared::CommonError;
use crate::{LoomzClient, GameState, GameInputFlags};

const RETURN_EDITOR: u64 = 200;
const EXIT_EDITOR: u64 = 201;

impl LoomzClient {

    pub(crate) fn init_editor(&mut self) -> Result<(), CommonError> {
        self.gui.toggle(&self.api, false);

        self.init_editor_gui()?;
        self.build_editor_gui()?;
    
        self.init_editor_terrain()?;
        self.api.world().toggle_world(true);
        self.state = GameState::Editor;

        Ok(())
    }

    pub(crate) fn editor(&mut self) -> Result<(), CommonError> {
        self.editor_global_update()?;

        if self.gui.visible() {
            self.editor_gui_events()?;
        } else {
            self.editor_updates();
        }

        Ok(())
    }

    fn editor_global_update(&mut self) -> Result<(), CommonError> {
        let inputs = self.api.inputs_ref();

        if let Some(new_size) = inputs.screen_size() {
            self.terrain.resize_view(new_size.width, new_size.height);
            self.terrain.sync(&self.api);
        }

        if let Some(buttons) = inputs.mouse_buttons() {
            match buttons.right_button_down() {
                true => { self.input_flags.insert(GameInputFlags::DRAGGING_VIEW); },
                false => { self.input_flags.remove(GameInputFlags::DRAGGING_VIEW); }
            }
        }

        if let Some(keystate) = self.api.keys_ref().read_updates() {
            if keystate.just_pressed(keys::ESC) {
                self.gui.toggle(&self.api, !self.gui.visible());
            }
        }

        if inputs.screen_size().is_some() {
            self.build_editor_gui()?;
        }

        Ok(())
    }

    fn editor_gui_events(&mut self) -> Result<(), CommonError> { 
        self.gui.read_inputs(&self.api);

        while let Some(event) = self.gui.next_event() {
            match event {
                RETURN_EDITOR => { self.gui.toggle(&self.api, false); },
                EXIT_EDITOR => { self.init_main_menu()?; },
                _ => {}
            }
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
        use crate::gui::{GuiLayoutType, GuiLayoutPosition};

        self.gui.build_style(&self.api, |style| {
            style.root_layout(GuiLayoutType::VBox, GuiLayoutPosition::Center);
            super::shared::main_panel_style(style);
        })
    }

    fn build_editor_gui(&mut self) -> Result<(), CommonError> {
        use crate::gui::GuiLabelCallback;

        let screen_size = self.api.inputs().screen_size_value();
        let view = loomz_shared::RectF32::from_size(screen_size);

        self.gui.build(&self.api, &view, |gui| {
            gui.layout_item(screen_size.width, screen_size.height);
            gui.frame("shadow", |gui| {
                gui.layout_item(400.0, 300.0);
                gui.frame("main_panel_style", |gui| {
                    gui.layout_item(300.0, 105.0);
    
                    gui.label_callback(GuiLabelCallback::Click, RETURN_EDITOR);
                    gui.label("Continue", "menu_item");
    
                    gui.label_callback(GuiLabelCallback::Click, EXIT_EDITOR);
                    gui.label("Exit", "menu_item");
                });
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
