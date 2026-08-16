use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use ps_core::config::Config;
use ps_core::fsops::{self, ConflictStrategy, CopyOutcome, MoveOutcome};
use ps_core::paths::AppPaths;
use ps_core::projects::{Project, ProjectStore, ProjectsListQuery, ProjectsListResult};
use ps_core::search::ProjectSearch;
use ps_core::store::JsonStore;
use ps_core::tree::{self, TreeNode};
use ps_core::{Error, Result};

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

    pub(crate) fn tree_read_dir(
        &self,
        project_id: String,
        rel_path: PathBuf,
    ) -> Result<Vec<TreeNode>> {
        let root = self.project_root(&project_id)?;
        let show_hidden = self.config_get().files.show_hidden;
        tree::read_dir(&root, &rel_path, show_hidden)
    }

    pub(crate) fn fs_mkdir(&self, project_id: String, rel_path: PathBuf) -> Result<TreeNode> {
        let root = self.project_root(&project_id)?;
        let absolute = fsops::mkdir(&root, &rel_path)?;
        tree::node_at(&root, &absolute)
    }

    pub(crate) fn fs_create_file(&self, project_id: String, rel_path: PathBuf) -> Result<TreeNode> {
        let root = self.project_root(&project_id)?;
        let absolute = fsops::create_file(&root, &rel_path)?;
        tree::node_at(&root, &absolute)
    }

    pub(crate) fn fs_rename(
        &self,
        project_id: String,
        from: PathBuf,
        to: PathBuf,
    ) -> Result<TreeNode> {
        let root = self.project_root(&project_id)?;
        let absolute = fsops::rename(&root, &from, &to)?;
        tree::node_at(&root, &absolute)
    }

    pub(crate) fn fs_copy(
        &self,
        project_id: String,
        from: Vec<PathBuf>,
        to_dir: PathBuf,
        conflict: ConflictStrategy,
    ) -> Result<Vec<TreeNode>> {
        let root = self.project_root(&project_id)?;
        let mut nodes = Vec::with_capacity(from.len());
        for rel_path in from {
            let file_name = rel_path.file_name().ok_or_else(|| Error::UnsafePath {
                path: rel_path.clone(),
                reason: "the copy source does not have a file name",
            })?;
            let destination = to_dir.join(file_name);
            let outcome = fsops::copy(
                &root,
                &rel_path,
                &destination,
                conflict,
                pending_history_snapshot,
                |_| {},
            )?;
            let absolute = match outcome {
                CopyOutcome::Copied { path } | CopyOutcome::Skipped { path } => path,
            };
            nodes.push(tree::node_at(&root, &absolute)?);
        }
        Ok(nodes)
    }

    pub(crate) fn fs_move(
        &self,
        project_id: String,
        from: Vec<PathBuf>,
        to_dir: PathBuf,
        conflict: ConflictStrategy,
    ) -> Result<Vec<TreeNode>> {
        let root = self.project_root(&project_id)?;
        fsops::move_items(
            &root,
            &from,
            &to_dir,
            conflict,
            pending_history_snapshot,
            |_| {},
        )?
        .into_iter()
        .map(|outcome| {
            let absolute = match outcome {
                MoveOutcome::Moved { path, .. } | MoveOutcome::Skipped { path, .. } => path,
            };
            tree::node_at(&root, &absolute)
        })
        .collect()
    }

    pub(crate) fn fs_trash(&self, project_id: String, rel_paths: Vec<PathBuf>) -> Result<()> {
        let root = self.project_root(&project_id)?;
        fsops::trash(&root, &rel_paths, pending_history_snapshot)
    }

    fn project_root(&self, id: &str) -> Result<PathBuf> {
        let projects = self
            .projects
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        Ok(projects.get(id)?.path.clone())
    }

    #[cfg(test)]
    pub(crate) fn paths(&self) -> &AppPaths {
        &self.paths
    }
}

/// History snapshots for destructive filesystem operations land in T-200.
fn pending_history_snapshot(_: &Path) -> Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use ps_core::config::Config;
    use ps_core::fsops::ConflictStrategy;
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

    #[test]
    fn filesystem_commands_stay_inside_the_project() {
        let (temporary, state) = open_state();
        let project_root = temporary.path().join("notes");
        fs::create_dir(&project_root).expect("project directory");
        let project = state
            .projects_add("Notes".into(), project_root.clone())
            .expect("add");

        let folder = state
            .fs_mkdir(project.id.clone(), PathBuf::from("inbox"))
            .expect("mkdir");
        assert_eq!(folder.name, "inbox");
        assert_eq!(folder.kind, ps_core::tree::TreeNodeKind::Directory);

        let created = state
            .fs_create_file(project.id.clone(), PathBuf::from("inbox/draft.md"))
            .expect("create");
        assert_eq!(created.name, "draft.md");
        assert_eq!(created.kind, ps_core::tree::TreeNodeKind::File);

        let renamed = state
            .fs_rename(
                project.id.clone(),
                PathBuf::from("inbox/draft.md"),
                PathBuf::from("inbox/note.md"),
            )
            .expect("rename");
        assert_eq!(renamed.rel_path, PathBuf::from("inbox/note.md"));

        let copied = state
            .fs_copy(
                project.id.clone(),
                vec![PathBuf::from("inbox/note.md")],
                PathBuf::new(),
                ConflictStrategy::KeepBoth,
            )
            .expect("copy");
        assert_eq!(copied.len(), 1);
        assert_eq!(copied[0].name, "note.md");
        assert_eq!(fs::read(project_root.join("note.md")).expect("copied"), b"");

        let moved = state
            .fs_move(
                project.id.clone(),
                vec![PathBuf::from("note.md")],
                PathBuf::from("inbox"),
                ConflictStrategy::KeepBoth,
            )
            .expect("move");
        assert_eq!(moved[0].name, "note 2.md");
        assert!(!project_root.join("note.md").exists());
        assert_eq!(
            fs::read(project_root.join("inbox/note 2.md")).expect("moved"),
            b""
        );

        let nodes = state
            .tree_read_dir(project.id.clone(), PathBuf::from("inbox"))
            .expect("tree");
        let names = nodes
            .iter()
            .map(|node| node.name.as_str())
            .collect::<Vec<_>>();
        assert!(names.contains(&"note.md"));
        assert!(names.contains(&"note 2.md"));

        fs::write(project_root.join(".hidden.md"), b"secret").expect("hidden");
        let filtered = state
            .tree_read_dir(project.id.clone(), PathBuf::new())
            .expect("filtered tree");
        assert!(filtered.iter().all(|node| node.name != ".hidden.md"));

        let mut config = state.config_get();
        config.files.show_hidden = true;
        state.config_set(config).expect("show hidden");
        let shown = state
            .tree_read_dir(project.id.clone(), PathBuf::new())
            .expect("complete tree");
        assert!(shown.iter().any(|node| node.name == ".hidden.md"));

        assert!(
            state
                .fs_mkdir(project.id.clone(), PathBuf::from("../escape"))
                .is_err()
        );
        assert!(!temporary.path().join("escape").exists());
        assert!(
            state
                .tree_read_dir(project.id.clone(), PathBuf::from(".."))
                .is_err()
        );
        assert!(
            state
                .tree_read_dir("missing".into(), PathBuf::new())
                .is_err()
        );

        state
            .fs_trash(project.id.clone(), vec![PathBuf::from("inbox/note.md")])
            .expect("trash");
        assert!(!project_root.join("inbox/note.md").exists());
        assert!(project_root.join("inbox/note 2.md").exists());
    }
}
