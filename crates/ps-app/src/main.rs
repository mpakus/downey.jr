//! Desktop application entry point for 1537paperstreet.

#![warn(missing_docs)]

use std::error::Error;

use ps_core::paths::AppPaths;

mod state;

use state::AppState;

fn main() -> Result<(), Box<dyn Error>> {
    let state = AppState::open(AppPaths::discover()?)?;

    tauri::Builder::default()
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .manage(state)
        .run(tauri::generate_context!())
        .map_err(Into::into)
}
