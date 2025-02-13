use loomz_shared::{CommonError, rect};
use crate::{LoomzClient, GameState};

const START_GAME: u64 = 100;
const START_SANDBOX: u64 = 101;
const EXIT_GAME: u64 = 102;

impl LoomzClient {

    pub(crate) fn init_main_menu(&mut self) -> Result<(), CommonError> {
        self.init_main_menu_menu()?;
        self.api.world().toggle_world(false);
        self.state = GameState::MainMenu;

        Ok(())
    }

    pub(crate) fn main_menu(&mut self) -> Result<(), CommonError> {
        let inputs = self.api.inputs_ref();
        let mut gui_updates = crate::gui::GuiUpdates::default();

        if let Some(cursor_position) = inputs.cursor_position() {
            gui_updates.cursor_position = Some(cursor_position.as_f32());
        }

        if let Some(buttons) = inputs.mouse_buttons() {
            gui_updates.left_mouse_down = Some(buttons.left_button_down());
        }

        if let Some(new_size) = inputs.screen_size() {
            gui_updates.view = Some(rect(0.0, 0.0, new_size.width, new_size.height));
        }

        self.gui.update(&self.api, &gui_updates);
        self.main_menu_gui_events()?;

        Ok(())
    }

    fn main_menu_gui_events(&mut self) -> Result<(), CommonError> {
        let mut start_game = false;
        let mut start_sandbox = false;

        for event in self.gui.drain_events() {
            match event {
                START_GAME => { start_game = true; },
                START_SANDBOX => { start_sandbox = true; },
                EXIT_GAME => { self.api.exit(); },
                _ => {}
            }
        }

        if start_game {
            self.init_gameplay()?;
        } else if start_sandbox {
            self.init_editor()?;
        }

        Ok(())
    }

    fn init_main_menu_menu(&mut self) -> Result<(), CommonError> {
        use crate::gui::{GuiLayoutType, GuiLabelCallback};

        let screen_size = self.api.inputs().screen_size_value();
        let view = loomz_shared::RectF32::from_size(screen_size);

        self.gui.build_style(&self.api, |style| {
            style.root_layout(GuiLayoutType::VBox);
            super::shared::main_panel_style(style);
        })?;

        self.gui.build(&self.api, &view, |gui| {
            gui.layout(GuiLayoutType::VBox);
            gui.layout_item(500.0, 440.0);
            gui.frame("main_panel_style", |gui| {
                gui.layout_item(300.0, 110.0);

                gui.label_callback(GuiLabelCallback::Click, START_GAME);
                gui.label("New Game", "menu_item");

                gui.label_callback(GuiLabelCallback::Click, START_SANDBOX);
                gui.label("Editor", "menu_item");

                gui.label_callback(GuiLabelCallback::Click, EXIT_GAME);
                gui.label("Exit", "menu_item");
            });
        })
    }

}
