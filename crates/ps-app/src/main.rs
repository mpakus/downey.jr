//! Desktop application entry point for 1537paperstreet.

#![warn(missing_docs)]

fn main() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("failed to run 1537paperstreet");
}
