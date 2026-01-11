use eyemotion_core::{GameState, UserConfig};
use std::sync::Mutex;

pub struct AppState {
    pub game_state: Mutex<GameState>,
    pub user_config: Mutex<UserConfig>,
}
