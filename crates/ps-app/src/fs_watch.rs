//! Debounced project watcher that emits `fs://changed` to the UI.

use std::path::PathBuf;
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::time::Duration;

use ps_core::watch::{FsChangedEvent, ProjectWatcher};
use tauri::{AppHandle, Emitter};

use crate::state::AppState;

const POLL: Duration = Duration::from_millis(50);

enum WatchCommand {
    Start {
        project_id: String,
        root: PathBuf,
        expanded: Vec<PathBuf>,
    },
    SetExpanded(Vec<PathBuf>),
    Stop,
}

/// Handle used by commands to control the single active project watcher.
#[derive(Clone)]
pub(crate) struct WatchHub {
    commands: Sender<WatchCommand>,
}

impl WatchHub {
    pub(crate) fn spawn(app: AppHandle) -> Self {
        let (commands, incoming) = mpsc::channel();
        std::thread::Builder::new()
            .name("paperstreet-watch-hub".to_owned())
            .spawn(move || hub_loop(app, incoming))
            .ok();
        Self { commands }
    }

    pub(crate) fn start(
        &self,
        project_id: String,
        root: PathBuf,
        expanded: Vec<PathBuf>,
    ) -> Result<(), String> {
        self.commands
            .send(WatchCommand::Start {
                project_id,
                root,
                expanded,
            })
            .map_err(|error| error.to_string())
    }

    pub(crate) fn set_expanded(&self, expanded: Vec<PathBuf>) -> Result<(), String> {
        self.commands
            .send(WatchCommand::SetExpanded(expanded))
            .map_err(|error| error.to_string())
    }

    pub(crate) fn stop(&self) -> Result<(), String> {
        self.commands
            .send(WatchCommand::Stop)
            .map_err(|error| error.to_string())
    }
}

fn hub_loop(app: AppHandle, incoming: mpsc::Receiver<WatchCommand>) {
    let mut active: Option<(String, ProjectWatcher)> = None;
    loop {
        match incoming.recv_timeout(POLL) {
            Ok(WatchCommand::Start {
                project_id,
                root,
                expanded,
            }) => match ProjectWatcher::start(&root) {
                Ok(watcher) => {
                    let _ = watcher.set_expanded(&expanded);
                    active = Some((project_id, watcher));
                }
                Err(_) => active = None,
            },
            Ok(WatchCommand::SetExpanded(expanded)) => {
                if let Some((_, watcher)) = &active {
                    let _ = watcher.set_expanded(&expanded);
                }
            }
            Ok(WatchCommand::Stop) => active = None,
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return,
        }
        if let Some((project_id, watcher)) = &active
            && let Some(update) = watcher.try_recv()
        {
            let _ = app.emit(
                "fs://changed",
                FsChangedEvent {
                    project_id: project_id.clone(),
                    update,
                },
            );
        }
    }
}

/// Starts watching the project that is currently shown.
pub(crate) fn start_for_project(
    hub: &WatchHub,
    state: &AppState,
    project_id: String,
) -> Result<(), String> {
    let root = state
        .project_root(&project_id)
        .map_err(|error| error.to_string())?;
    let expanded = state
        .tree_expanded_get(project_id.clone())
        .into_iter()
        .map(PathBuf::from)
        .collect();
    hub.start(project_id, root, expanded)
}
