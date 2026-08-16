//! Thin Tauri command wrappers around `ps-core`.

use std::path::PathBuf;

use ps_core::config::Config;
use ps_core::projects::{Project, ProjectsListQuery, ProjectsListResult};
use tauri::State;

use crate::state::AppState;

fn to_command_error(error: ps_core::Error) -> String {
    error.to_string()
}

#[tauri::command]
pub(crate) fn config_get(state: State<'_, AppState>) -> Config {
    state.config_get()
}

#[tauri::command]
pub(crate) async fn config_set(state: State<'_, AppState>, config: Config) -> Result<(), String> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || state.config_set(config))
        .await
        .map_err(|error| error.to_string())
        .and_then(|result| result.map_err(to_command_error))
}

#[tauri::command]
pub(crate) fn projects_list(
    state: State<'_, AppState>,
    query: ProjectsListQuery,
) -> ProjectsListResult {
    state.projects_list(query)
}

#[tauri::command]
pub(crate) async fn projects_add(
    state: State<'_, AppState>,
    name: String,
    path: PathBuf,
) -> Result<Project, String> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || state.projects_add(name, path))
        .await
        .map_err(|error| error.to_string())
        .and_then(|result| result.map_err(to_command_error))
}

#[tauri::command]
pub(crate) async fn projects_rename(
    state: State<'_, AppState>,
    id: String,
    name: String,
) -> Result<(), String> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || state.projects_rename(id, name))
        .await
        .map_err(|error| error.to_string())
        .and_then(|result| result.map_err(to_command_error))
}

#[tauri::command]
pub(crate) async fn projects_remove(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || state.projects_remove(id))
        .await
        .map_err(|error| error.to_string())
        .and_then(|result| result.map_err(to_command_error))
}
