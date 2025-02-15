use loomz_shared::base_types::rect;
use loomz_shared::inputs::keys;
use loomz_shared::CommonError;
use crate::{GameState, LoomzClient};

const RETURN_GAMEPLAY: u64 = 300;
const EXIT_GAMEPLAY: u64 = 301;


impl LoomzClient {

    pub(crate) fn init_gameplay(&mut self) -> Result<(), CommonError> {
        self.init_gameplay_gui()?;
        self.api.world().toggle_world(true);
        self.state = GameState::Game;
        Ok(())
    }

    pub(crate) fn gameplay(&mut self) -> Result<(), CommonError> {
        self.gameplay_updates();

        if self.gui.visible() {
            self.gameplay_gui_events()?;
        } else {
            self.gameplay_loop();
        }

        Ok(())
    }

    fn gameplay_updates(&mut self) {
        if let Some(keystate) = self.api.keys_ref().read_updates() {
            let size = self.api.inputs_ref().screen_size_value();
            if keystate.just_pressed(keys::ESC) {
                self.gui.resize(&self.api, &rect(0.0, 0.0, size.width, size.height));
                self.gui.toggle(&self.api, !self.gui.visible());
            }
        }

        ()
    }

    fn gameplay_gui_events(&mut self) -> Result<(), CommonError> {
        self.gui.read_inputs(&self.api);

        while let Some(event) = self.gui.next_event() {
            match event {
                RETURN_GAMEPLAY => { self.gui.toggle(&self.api, false); },
                EXIT_GAMEPLAY => { self.init_main_menu()?; },
                _ => {}
            }
        }

        Ok(())
    }

    fn gameplay_loop(&mut self) {
        // let world = self.api.world();
        // let position = self.player.position;
        // let target = self.target_position;

        // if position.out_of_range(target, 16.0) {
        //     let speed = 200.0 * self.timing.delta_ms;
        //     let angle = f32::atan2(target.y - position.y, target.x - position.x) as f64;
        //     let speed_x = speed * f64::cos(angle);
        //     let speed_y = speed * f64::sin(angle);

        //     self.player.position += PositionF64 { x: speed_x, y: speed_y };
        //     world.update_actor_position(&self.player.id, self.player.position);

        //     if self.player.animation != PawnAnimationType::Walk {
        //         world.update_actor_animation(&self.player.id, &self.animations.warrior.walk);
        //         self.player.animation = PawnAnimationType::Walk;
        //     }

        //     if speed_x < 0.0 && !self.player.flip {
        //         self.player.flip = true;
        //         world.flip_actor(&self.player.id, true);
        //     } else if speed_x > 0.0 && self.player.flip {
        //         self.player.flip = false;
        //         world.flip_actor(&self.player.id, false);
        //     }
        // } else {
        //     if self.player.animation != PawnAnimationType::Idle {
        //         world.update_actor_animation(&self.player.id, &self.animations.warrior.idle);
        //         self.player.animation = PawnAnimationType::Idle;
        //     }
        // }
    }

    fn init_player(&mut self) {
        // let start_position = PositionF32 { x: 100.0, y: 500.0 };
        // let player = Player {
        //     id: WorldActorId::new(),
        //     position: start_position,
        //     animation: PawnAnimationType::Idle,
        //     flip: false,
        // };

        // self.api.world().create_actor(
        //     &player.id,
        //     player.position,
        //     &self.animations.warrior.idle,
        // );

        // self.player = player;
        // self.target_position = start_position;
    }

    fn init_gameplay_gui(&mut self) -> Result<(), CommonError> {
        use crate::gui::{GuiLayoutType, GuiLabelCallback, GuiLayoutPosition};

        let screen_size = self.api.inputs().screen_size_value();
        let view = loomz_shared::RectF32::from_size(screen_size);

        self.gui.toggle(&self.api, false);

        self.gui.build_style(&self.api, |style| {
            style.root_layout(GuiLayoutType::VBox, GuiLayoutPosition::Center);
            super::shared::main_panel_style(style);
        })?;

        self.gui.build(&self.api, &view, |gui| {
            gui.layout(GuiLayoutType::VBox, GuiLayoutPosition::Center);
            gui.layout_item(400.0, 300.0);
            gui.frame("main_panel_style", |gui| {
                gui.layout_item(300.0, 100.0);

                gui.label_callback(GuiLabelCallback::Click, RETURN_GAMEPLAY);
                gui.label("Continue", "menu_item");

                gui.label_callback(GuiLabelCallback::Click, EXIT_GAMEPLAY);
                gui.label("Exit", "menu_item");
            });
        })?;

        Ok(())
    }


}
