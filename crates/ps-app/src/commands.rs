//! Thin Tauri command wrappers around `ps-core`.

use std::path::PathBuf;

use ps_core::config::Config;
use ps_core::fsops::ConflictStrategy;
use ps_core::projects::{Project, ProjectsListQuery, ProjectsListResult};
use ps_core::tree::TreeNode;
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

#[tauri::command]
pub(crate) async fn tree_read_dir(
    state: State<'_, AppState>,
    project_id: String,
    rel_path: PathBuf,
) -> Result<Vec<TreeNode>, String> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || state.tree_read_dir(project_id, rel_path))
        .await
        .map_err(|error| error.to_string())
        .and_then(|result| result.map_err(to_command_error))
}

#[tauri::command]
pub(crate) async fn fs_mkdir(
    state: State<'_, AppState>,
    project_id: String,
    rel_path: PathBuf,
) -> Result<TreeNode, String> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || state.fs_mkdir(project_id, rel_path))
        .await
        .map_err(|error| error.to_string())
        .and_then(|result| result.map_err(to_command_error))
}

#[tauri::command]
pub(crate) async fn fs_create_file(
    state: State<'_, AppState>,
    project_id: String,
    rel_path: PathBuf,
) -> Result<TreeNode, String> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || state.fs_create_file(project_id, rel_path))
        .await
        .map_err(|error| error.to_string())
        .and_then(|result| result.map_err(to_command_error))
}

#[tauri::command]
pub(crate) async fn fs_rename(
    state: State<'_, AppState>,
    project_id: String,
    from: PathBuf,
    to: PathBuf,
) -> Result<TreeNode, String> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || state.fs_rename(project_id, from, to))
        .await
        .map_err(|error| error.to_string())
        .and_then(|result| result.map_err(to_command_error))
}

#[tauri::command]
pub(crate) async fn fs_copy(
    state: State<'_, AppState>,
    project_id: String,
    from: Vec<PathBuf>,
    to_dir: PathBuf,
    conflict: ConflictStrategy,
) -> Result<Vec<TreeNode>, String> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || state.fs_copy(project_id, from, to_dir, conflict))
        .await
        .map_err(|error| error.to_string())
        .and_then(|result| result.map_err(to_command_error))
}

#[tauri::command]
pub(crate) async fn fs_move(
    state: State<'_, AppState>,
    project_id: String,
    from: Vec<PathBuf>,
    to_dir: PathBuf,
    conflict: ConflictStrategy,
) -> Result<Vec<TreeNode>, String> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || state.fs_move(project_id, from, to_dir, conflict))
        .await
        .map_err(|error| error.to_string())
        .and_then(|result| result.map_err(to_command_error))
}

#[tauri::command]
pub(crate) async fn fs_trash(
    state: State<'_, AppState>,
    project_id: String,
    rel_paths: Vec<PathBuf>,
) -> Result<(), String> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || state.fs_trash(project_id, rel_paths))
        .await
        .map_err(|error| error.to_string())
        .and_then(|result| result.map_err(to_command_error))
}
