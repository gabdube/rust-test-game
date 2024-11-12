use loomz_shared::_2d::{Position, pos};

pub struct Player {
    pub pos: Position,
    pub visible: bool,
}

pub enum GameState {
    Uninitialized,
    Gameplay,
}

pub struct LoomzClient {
    state: GameState,
    player: Player
}

impl LoomzClient {

    pub fn init() -> Self {
        let player = Player {
            pos: pos(0.0, 0.0),
            visible: false,
        };

        LoomzClient {
            state: GameState::Uninitialized,
            player,
        }
    }

    pub fn update(&mut self) {
        match self.state {
            GameState::Uninitialized => self.uninitialized(),
            GameState::Gameplay => self.gameplay(),
        }
    }

    fn uninitialized(&mut self) {
        self.player.visible = true;
        self.state = GameState::Gameplay;
    }

    fn gameplay(&mut self) {

    }

}
