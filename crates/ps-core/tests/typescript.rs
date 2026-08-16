use std::fs;

use ps_core::typescript::{bindings_path, ipc_typescript};

#[test]
fn generated_typescript_matches_committed_bindings() {
    let generated = ipc_typescript();
    let path = bindings_path();

    if std::env::var_os("UPDATE_TS_BINDINGS").is_some() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("generated TypeScript directory");
        }
        fs::write(&path, &generated).expect("write TypeScript bindings");
        return;
    }

    let committed = fs::read_to_string(&path).unwrap_or_default();
    assert_eq!(
        committed,
        generated,
        "TypeScript bindings are stale at {}. Run UPDATE_TS_BINDINGS=1 cargo test -p ps-core --test typescript",
        path.display()
    );
}
