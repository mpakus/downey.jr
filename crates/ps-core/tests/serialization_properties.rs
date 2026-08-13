use std::path::PathBuf;

use proptest::prelude::*;
use ps_core::config::Config;
use ps_core::projects::{Project, ProjectsDocument};
use ulid::Ulid;

proptest! {
    #[test]
    fn config_json_round_trip_preserves_valid_values(
        font_size in 10_u16..=32,
        measure_ch in 40_u16..=120,
        follow_system in any::<bool>(),
    ) {
        let mut config = Config::default();
        config.typography.font_size = font_size;
        config.typography.measure_ch = measure_ch;
        config.appearance.follow_system = follow_system;

        let encoded = serde_json::to_vec(&config)?;
        let decoded: Config = serde_json::from_slice(&encoded)?;

        prop_assert_eq!(decoded, config);
    }

    #[test]
    fn projects_json_round_trip_preserves_records(
        records in prop::collection::vec(
            (
                any::<u128>(),
                "[A-Za-z0-9][A-Za-z0-9 ]{0,31}",
                "[a-z0-9_-]{1,24}",
                any::<bool>(),
            ),
            0..64,
        ),
    ) {
        let projects = records
            .into_iter()
            .map(|(id, name, directory, pinned)| Project {
                id: Ulid::from(id).to_string(),
                name,
                path: PathBuf::from(format!("/tmp/{directory}")),
                added_at: "2026-08-13T12:00:00Z".to_owned(),
                last_opened_at: None,
                pinned,
                accent: None,
                last_file: None,
                available: None,
            })
            .collect();
        let document = ProjectsDocument {
            schema_version: 1,
            projects,
        };

        let encoded = serde_json::to_vec(&document)?;
        let decoded: ProjectsDocument = serde_json::from_slice(&encoded)?;

        prop_assert_eq!(decoded, document);
    }
}
