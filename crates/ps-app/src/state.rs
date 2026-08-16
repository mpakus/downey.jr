use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use ps_core::Result;
use ps_core::config::Config;
use ps_core::paths::AppPaths;
use ps_core::projects::{Project, ProjectStore, ProjectsListQuery, ProjectsListResult};
use ps_core::search::ProjectSearch;
use ps_core::store::JsonStore;

#[derive(Clone)]
pub(crate) struct AppState {
    #[allow(dead_code)]
    paths: AppPaths,
    config: Arc<Mutex<JsonStore<Config>>>,
    projects: Arc<Mutex<ProjectStore>>,
    search: Arc<Mutex<ProjectSearch>>,
}

impl AppState {
    pub(crate) fn open(paths: AppPaths) -> Result<Self> {
        paths.ensure()?;
        let config = JsonStore::open(paths.config_file())?;
        let projects = ProjectStore::open(paths.projects_file())?;
        let mut search = ProjectSearch::new();
        for project in projects.list() {
            search.upsert(project.clone());
        }

        Ok(Self {
            paths,
            config: Arc::new(Mutex::new(config)),
            projects: Arc::new(Mutex::new(projects)),
            search: Arc::new(Mutex::new(search)),
        })
    }

    pub(crate) fn config_get(&self) -> Config {
        self.config
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .value()
            .clone()
    }

    pub(crate) fn config_set(&self, config: Config) -> Result<()> {
        config.validate()?;
        let mut store = self
            .config
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        store.replace(config);
        store.flush().map(|_| ())
    }

    pub(crate) fn projects_list(&self, query: ProjectsListQuery) -> ProjectsListResult {
        let limit = usize::try_from(query.limit).unwrap_or(usize::MAX);
        let offset = usize::try_from(query.offset).unwrap_or(usize::MAX);
        let trimmed = query
            .query
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());

        let (items, total) = if let Some(needle) = trimmed {
            self.search
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .page(needle, limit, offset)
        } else {
            let projects = self
                .projects
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            let all = projects.list();
            let total = all.len();
            (
                all.iter().skip(offset).take(limit).cloned().collect(),
                total,
            )
        };

        ProjectsListResult {
            items,
            total: u32::try_from(total).unwrap_or(u32::MAX),
        }
    }

    pub(crate) fn projects_add(&self, name: String, path: PathBuf) -> Result<Project> {
        let mut projects = self
            .projects
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let project = projects.add(name, path)?;
        self.search
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .upsert(project.clone());
        projects.flush()?;
        Ok(project)
    }

    pub(crate) fn projects_rename(&self, id: String, name: String) -> Result<()> {
        let mut projects = self
            .projects
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        projects.rename(&id, name)?;
        if let Some(project) = projects
            .list()
            .iter()
            .find(|project| project.id == id)
            .cloned()
        {
            self.search
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .upsert(project);
        }
        projects.flush().map(|_| ())
    }

    pub(crate) fn projects_remove(&self, id: String) -> Result<()> {
        let mut projects = self
            .projects
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        projects.remove(&id)?;
        self.search
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .remove(&id);
        projects.flush().map(|_| ())
    }

    #[cfg(test)]
    pub(crate) fn paths(&self) -> &AppPaths {
        &self.paths
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use ps_core::config::Config;
    use ps_core::paths::AppPaths;
    use ps_core::projects::ProjectsListQuery;

    use super::AppState;

    fn open_state() -> (tempfile::TempDir, AppState) {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let state = AppState::open(AppPaths::from_root(temporary.path()))
            .expect("application state should open");
        (temporary, state)
    }

    #[test]
    fn opens_default_stores_under_the_application_root() {
        let (temporary, state) = open_state();

        assert_eq!(state.paths().root(), temporary.path());
        assert_eq!(state.config_get(), Config::default());
        assert_eq!(
            state
                .projects_list(ProjectsListQuery {
                    limit: 50,
                    ..ProjectsListQuery::default()
                })
                .total,
            0
        );
    }

    #[test]
    fn config_set_validates_and_persists() {
        let (temporary, state) = open_state();
        let mut config = Config::default();
        config.typography.font_size = 18;
        state.config_set(config.clone()).expect("config saved");
        assert_eq!(state.config_get().typography.font_size, 18);

        let reopened = AppState::open(AppPaths::from_root(temporary.path())).expect("reopen");
        assert_eq!(reopened.config_get().typography.font_size, 18);

        config.typography.font_size = 9;
        assert!(state.config_set(config).is_err());
        assert_eq!(state.config_get().typography.font_size, 18);
    }

    #[test]
    fn project_commands_never_touch_the_folder() {
        let (temporary, state) = open_state();
        let project_root = temporary.path().join("notes");
        fs::create_dir(&project_root).expect("project directory");
        let document = project_root.join("keep.md");
        fs::write(&document, b"keep").expect("document");

        let added = state
            .projects_add("Notes".into(), project_root.clone())
            .expect("add");
        let listed = state.projects_list(ProjectsListQuery {
            limit: 10,
            ..ProjectsListQuery::default()
        });
        assert_eq!(listed.total, 1);
        assert_eq!(listed.items[0].id, added.id);

        let searched = state.projects_list(ProjectsListQuery {
            query: Some("nts".into()),
            limit: 10,
            offset: 0,
        });
        assert_eq!(searched.total, 1);

        state
            .projects_rename(added.id.clone(), "Writing".into())
            .expect("rename");
        state.projects_remove(added.id).expect("remove");

        assert_eq!(
            state
                .projects_list(ProjectsListQuery {
                    limit: 10,
                    ..ProjectsListQuery::default()
                })
                .total,
            0
        );
        assert_eq!(fs::read(&document).expect("preserved"), b"keep");
    }
}
