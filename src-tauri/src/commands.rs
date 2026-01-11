use eyemotion_core::Theme;
use tauri::State;

#[tauri::command]
pub async fn tick(
    state: State<'_, super::state::AppState>,
    dt: f64,
) -> Result<(eyemotion_core::GameState, Vec<eyemotion_core::GameEvent>), String> {
    let mut game_state = state.game_state.lock().map_err(|e| e.to_string())?;

    let update = game_state.update(dt).map_err(|e| e.to_string())?;

    Ok((game_state.clone(), update.events))
}

#[tauri::command]
pub fn reset_game(state: State<'_, super::state::AppState>, w: f64, h: f64) {
    if let Ok(mut game_state) = state.game_state.lock() {
        game_state.reset(w, h);
    }
}

#[tauri::command]
pub fn toggle_pause(state: State<'_, super::state::AppState>) {
    if let Ok(mut game_state) = state.game_state.lock() {
        game_state.paused = !game_state.paused;
    }
}

#[tauri::command]
pub fn start_game(state: State<'_, super::state::AppState>) {
    if let Ok(mut game_state) = state.game_state.lock() {
        game_state.is_start_screen = false;
    }
}

#[tauri::command]
pub fn exit_app(app_handle: tauri::AppHandle) {
    app_handle.exit(0);
}

#[tauri::command]
pub fn get_theme() -> Theme {
    Theme::default()
}

#[tauri::command]
pub fn set_language(state: State<'_, super::state::AppState>, language: String) {
    if let Ok(mut config) = state.user_config.lock() {
        config.language = language.clone();
        let _ = config.save();
    }
}

#[tauri::command]
pub fn get_language(state: State<'_, super::state::AppState>) -> String {
    if let Ok(config) = state.user_config.lock() {
        config.language.clone()
    } else {
        "en".to_string()
    }
}
