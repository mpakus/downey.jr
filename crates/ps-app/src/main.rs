//! Desktop application entry point for 1537paperstreet.

#![warn(missing_docs)]

use std::error::Error;

use ps_core::paths::AppPaths;

mod commands;
mod state;

use commands::{
    config_get, config_set, doc_open, doc_source, fs_copy, fs_create_file, fs_mkdir, fs_move,
    fs_rename, fs_trash, projects_add, projects_list, projects_remove, projects_rename,
    tree_read_dir,
};
use state::AppState;

fn main() -> Result<(), Box<dyn Error>> {
    let state = AppState::open(AppPaths::discover()?)?;

    tauri::Builder::default()
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            config_get,
            config_set,
            projects_list,
            projects_add,
            projects_rename,
            projects_remove,
            tree_read_dir,
            fs_mkdir,
            fs_create_file,
            fs_rename,
            fs_copy,
            fs_move,
            fs_trash,
            doc_open,
            doc_source,
        ])
        .run(tauri::generate_context!())
        .map_err(Into::into)
}
