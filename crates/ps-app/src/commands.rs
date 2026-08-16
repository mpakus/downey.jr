//! Thin Tauri command wrappers around `ps-core`.

use std::path::PathBuf;

use ps_core::config::Config;
use ps_core::docio::{DocChunkEvent, DocDoneEvent, DocOpenResult, DocumentSource};
use ps_core::fsops::ConflictStrategy;
use ps_core::projects::{Project, ProjectsListQuery, ProjectsListResult};
use ps_core::tree::TreeNode;
use tauri::{Emitter, State};

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

#[tauri::command]
pub(crate) async fn doc_source(
    state: State<'_, AppState>,
    project_id: String,
    rel_path: PathBuf,
) -> Result<DocumentSource, String> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || state.doc_source(project_id, rel_path))
        .await
        .map_err(|error| error.to_string())
        .and_then(|result| result.map_err(to_command_error))
}

#[tauri::command]
pub(crate) async fn doc_open(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    project_id: String,
    rel_path: PathBuf,
) -> Result<DocOpenResult, String> {
    let state = state.inner().clone();
    let prepared =
        tauri::async_runtime::spawn_blocking(move || state.doc_open(project_id, rel_path))
            .await
            .map_err(|error| error.to_string())
            .and_then(|result| result.map_err(to_command_error))?;

    let result = prepared.result;
    let remaining_chunks = prepared.remaining_chunks;
    let project_id = result.meta.project_id.clone();
    let rel_path = result.meta.rel_path.clone();
    let chunk_count = result.meta.chunk_count;
    tauri::async_runtime::spawn(async move {
        for (offset, html) in remaining_chunks.into_iter().enumerate() {
            let index = u32::try_from(offset.saturating_add(1)).unwrap_or(u32::MAX);
            let _ = app.emit(
                "doc://chunk",
                DocChunkEvent {
                    project_id: project_id.clone(),
                    rel_path: rel_path.clone(),
                    index,
                    html,
                },
            );
        }
        let _ = app.emit(
            "doc://done",
            DocDoneEvent {
                project_id,
                rel_path,
                chunk_count,
            },
        );
    });
    Ok(result)
}
