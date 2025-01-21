use loomz_shared::api::WorldActorId;
use loomz_shared::base_types::{PositionF64, PositionF32, rect};
use loomz_shared::inputs::InputBuffer;
use loomz_shared::CommonError;
use crate::animations::PawnAnimationType;
use crate::{GameState, LoomzClient, Player};

impl LoomzClient {

    pub(crate) fn init_gameplay(&mut self) -> Result<(), CommonError> {
        self.init_gameplay_gui()?;
        self.init_player();
        self.state = GameState::Game;
        Ok(())
    }

    pub(crate) fn gameplay(&mut self) {
        if let Some(new_input) = self.api.read_inputs() {
           self.gameplay_inputs(new_input);
        }

        self.gameplay_loop();
    }

    fn gameplay_inputs(&mut self, new_inputs: InputBuffer) {
        let mut gui_updates = crate::gui::GuiUpdates::default();

        if let Some(cursor_position) = new_inputs.cursor_position() {
            self.target_position = cursor_position.as_f32();
            gui_updates.cursor_position = Some(cursor_position.as_f32());
        }

        if let Some(new_size) = new_inputs.screen_size() {
            gui_updates.view = Some(rect(0.0, 0.0, new_size.width, new_size.height));
        }

        if let Some(buttons) = new_inputs.mouse_buttons() {
            gui_updates.left_mouse_down = Some(buttons.left_button_down());
        }

        self.menu.update(&self.api, &gui_updates);
    }

    fn gameplay_loop(&mut self) {
        let world = self.api.world();
        let position = self.player.position;
        let target = self.target_position;

        if position.out_of_range(target, 2.0) {
            let speed = 200.0 * self.timing.delta_ms;
            let angle = f32::atan2(target.y - position.y, target.x - position.x) as f64;
            let speed_x = speed * f64::cos(angle);
            let speed_y = speed * f64::sin(angle);

            self.player.position += PositionF64 { x: speed_x, y: speed_y };
            world.update_actor_position(&self.player.id, self.player.position);

            if self.player.animation != PawnAnimationType::Walk {
                world.update_actor_animation(&self.player.id, &self.animations.warrior.walk);
                self.player.animation = PawnAnimationType::Walk;
            }

            if speed_x < 0.0 && !self.player.flip {
                self.player.flip = true;
                world.flip_actor(&self.player.id, true);
            } else if speed_x > 0.0 && self.player.flip {
                self.player.flip = false;
                world.flip_actor(&self.player.id, false);
            }
        } else {
            if self.player.animation != PawnAnimationType::Idle {
                world.update_actor_animation(&self.player.id, &self.animations.warrior.idle);
                self.player.animation = PawnAnimationType::Idle;
            }
        }
    }

    fn init_player(&mut self) {
        let start_position = PositionF32 { x: 100.0, y: 500.0 };
        let player = Player {
            id: WorldActorId::new(),
            position: start_position,
            animation: PawnAnimationType::Idle,
            flip: false,
        };

        self.api.world().create_actor(
            &player.id,
            player.position,
            &self.animations.warrior.idle,
        );

        self.player = player;
        self.target_position = start_position;
    }

    fn init_gameplay_gui(&mut self) -> Result<(), CommonError> {
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
