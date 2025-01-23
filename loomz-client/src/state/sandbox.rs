use loomz_shared::inputs::keys;
use loomz_shared::{CommonError, rect};
use crate::{GameState, LoomzClient};

const RETURN_SANDBOX: u64 = 200;
const EXIT_SANDBOX: u64 = 201;

impl LoomzClient {

    pub(crate) fn init_sandbox(&mut self) -> Result<(), CommonError> {
        self.init_sandbox_gui()?;
        self.state = GameState::Sandbox;
        Ok(())
    }

    pub(crate) fn sandbox(&mut self) -> Result<(), CommonError> {
        self.sandbox_update()?;

        if self.menu.visible() {
            self.sandbox_gui_updates();
            self.sandbox_gui_events()?;
        }

        Ok(())
    }

    fn sandbox_update(&mut self) -> Result<(), CommonError> {
        let new_inputs = match self.api.read_inputs() {
            Some(inputs) => inputs,
            None => { return Ok(()); }
        };

        if let Some(keystate) = new_inputs.keystate() {
            if keystate.clicked(keys::ESC) {
                self.menu.toggle(&self.api, !self.menu.visible());
            }
        }

        Ok(())
    }

    fn sandbox_gui_updates(&mut self) {
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

        self.menu.update(&self.api, &gui_updates);
    }

    fn sandbox_gui_events(&mut self) -> Result<(), CommonError> {
        let mut cont_sandbox = false;
        let mut exit_sandbox = false;

        for event in self.menu.drain_events() {
            match event {
                RETURN_SANDBOX => { cont_sandbox = true },
                EXIT_SANDBOX => { exit_sandbox = true; },
                _ => {}
            }
        }

        if cont_sandbox {
            self.menu.toggle(&self.api, false);
        } else if exit_sandbox {
            self.init_main_menu()?;
        }

        Ok(())
    }

    fn init_sandbox_gui(&mut self) -> Result<(), CommonError> {
        use crate::gui::{GuiLayoutType, GuiLabelCallback};

        let screen_size = self.api.inputs().screen_size_value();
        let view = loomz_shared::RectF32::from_size(screen_size);

        self.menu.build_style(&self.api, |style| {
            style.root_layout(GuiLayoutType::VBox);
            super::shared::main_panel_style(style);
        })?;

        self.menu.build(&self.api, &view, |gui| {
            gui.layout(GuiLayoutType::VBox);
            gui.layout_item(400.0, 300.0);
            gui.frame("main_panel_style", |gui| {
                gui.layout_item(300.0, 100.0);

                gui.label_callback(GuiLabelCallback::Click, RETURN_SANDBOX);
                gui.label("Continue", "menu_item");

                gui.label_callback(GuiLabelCallback::Click, EXIT_SANDBOX);
                gui.label("Exit", "menu_item");
            });
        })?;

        Ok(())
    }

}
