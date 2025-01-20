use loomz_shared::CommonError;
use crate::{GameState, LoomzClient};

impl LoomzClient {

    pub(crate) fn init_sandbox(&mut self) -> Result<(), CommonError> {
        self.init_sandbox_gui()?;
        self.state = GameState::Sandbox;
        Ok(())
    }

    pub(crate) fn sandbox(&mut self) {
    }

    fn init_sandbox_gui(&mut self) -> Result<(), CommonError> {
        use crate::gui::GuiLayoutType::VBox;

        let screen_size = self.api.inputs().screen_size_value();
        let view = loomz_shared::RectF32::from_size(screen_size);

        self.menu.build_style(&self.api, |style| {
            style.root_layout(VBox);
        })?;

        self.menu.build(&self.api, &view, |_gui| {

        })?;

        Ok(())
    }

}
