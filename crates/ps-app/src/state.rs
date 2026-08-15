use std::sync::Mutex;

use ps_core::Result;
use ps_core::config::Config;
use ps_core::paths::AppPaths;
use ps_core::projects::ProjectStore;
use ps_core::store::JsonStore;

pub(crate) struct AppState {
    _paths: AppPaths,
    _config: Mutex<JsonStore<Config>>,
    _projects: Mutex<ProjectStore>,
}

impl AppState {
    pub(crate) fn open(paths: AppPaths) -> Result<Self> {
        paths.ensure()?;
        let config = JsonStore::open(paths.config_file())?;
        let projects = ProjectStore::open(paths.projects_file())?;

        Ok(Self {
            _paths: paths,
            _config: Mutex::new(config),
            _projects: Mutex::new(projects),
        })
    }
}

#[cfg(test)]
mod tests {
    use ps_core::config::Config;
    use ps_core::paths::AppPaths;

    use super::AppState;

    #[test]
    fn opens_default_stores_under_the_application_root() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let state = AppState::open(AppPaths::from_root(temporary.path()))
            .expect("application state should open");

        assert_eq!(state._paths.root(), temporary.path());
        assert_eq!(
            state._config.lock().expect("config lock").value(),
            &Config::default()
        );
        assert!(
            state
                ._projects
                .lock()
                .expect("projects lock")
                .list()
                .is_empty()
        );
    }
}
