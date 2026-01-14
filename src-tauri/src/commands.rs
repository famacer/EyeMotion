use eyemotion_core::Theme;
use tauri::{State, Window};

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
pub fn reset_game(state: State<'_, super::state::AppState>, w: f64, h: f64) -> eyemotion_core::GameState {
    if let Ok(mut game_state) = state.game_state.lock() {
        game_state.reset(w, h);
        game_state.clone()
    } else {
        eyemotion_core::GameState::new(w, h)
    }
}

#[tauri::command]
pub fn resize_game(state: State<'_, super::state::AppState>, w: f64, h: f64) {
    if let Ok(mut game_state) = state.game_state.lock() {
        game_state.resize(w, h);
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
pub fn next_stage(state: State<'_, super::state::AppState>) -> Result<eyemotion_core::GameState, String> {
    if let Ok(mut game_state) = state.game_state.lock() {
        let mut events = Vec::new();
        game_state.next_stage(&mut events);
        Ok(game_state.clone())
    } else {
        Err("Failed to lock game state".to_string())
    }
}

#[tauri::command]
pub fn prev_stage(state: State<'_, super::state::AppState>) -> Result<eyemotion_core::GameState, String> {
    if let Ok(mut game_state) = state.game_state.lock() {
        let mut events = Vec::new();
        game_state.prev_stage(&mut events);
        Ok(game_state.clone())
    } else {
        Err("Failed to lock game state".to_string())
    }
}

#[tauri::command]
pub fn exit_app() {
    std::process::exit(0);
}

#[tauri::command]
pub fn minimize_window(window: Window) {
    let _ = window.minimize();
}

#[tauri::command]
pub fn toggle_fullscreen(window: Window) {
    let is_fullscreen = window.is_fullscreen().unwrap_or(false);
    let _ = window.set_fullscreen(!is_fullscreen);
}

#[tauri::command]
pub fn show_main_window(window: Window) {
    // 强制设置全屏状态。在移除 window-state 插件后，
    // 这里确保窗口在从隐藏转为显示时，已经是全屏模式。
    let _ = window.set_fullscreen(true);
    let _ = window.show();
    let _ = window.set_focus();
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

#[tauri::command]
pub fn get_config(state: State<'_, super::state::AppState>) -> eyemotion_core::UserConfig {
    if let Ok(config) = state.user_config.lock() {
        config.clone()
    } else {
        eyemotion_core::UserConfig::default()
    }
}
