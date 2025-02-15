use loomz_shared::CommonError;
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
        self.main_menu_gui_events()?;
        Ok(())
    }

    fn main_menu_gui_events(&mut self) -> Result<(), CommonError> {
        self.gui.read_inputs(&self.api);

        while let Some(event) = self.gui.next_event() {
            match event {
                START_GAME => { self.init_gameplay()?; },
                START_SANDBOX => { self.init_editor()?; },
                EXIT_GAME => { self.api.exit(); },
                _ => {}
            }
        }

        Ok(())
    }

    fn init_main_menu_menu(&mut self) -> Result<(), CommonError> {
        use crate::gui::{GuiLayoutType, GuiLabelCallback, GuiLayoutPosition};

        let screen_size = self.api.inputs().screen_size_value();
        let view = loomz_shared::RectF32::from_size(screen_size);

        self.gui.build_style(&self.api, |style| {
            style.root_layout(GuiLayoutType::VBox, GuiLayoutPosition::Center);
            super::shared::main_panel_style(style);
        })?;

        self.gui.build(&self.api, &view, |gui| {
            gui.layout(GuiLayoutType::VBox, GuiLayoutPosition::Center);
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
