mod commands;
mod state;

use commands::*;
use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().build())
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            game_state: std::sync::Mutex::new(eyemotion_core::GameState::new(800.0, 600.0)),
            user_config: std::sync::Mutex::new(eyemotion_core::UserConfig::load()),
        })
        .invoke_handler(tauri::generate_handler![
            tick,
            reset_game,
            resize_game,
            toggle_pause,
            start_game,
            next_stage,
                prev_stage,
                exit_app,
            minimize_window,
            toggle_fullscreen,
            show_main_window,
            get_theme,
            set_language,
            get_language,
            get_config
        ])
        .setup(|_app| {
            println!("Tauri setup started");
            Ok(())
        })
        .on_window_event(|_window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                std::process::exit(0);
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
