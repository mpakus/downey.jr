use ps_core::Error;
use ps_core::config::{Config, ViewMode};

#[test]
fn defaults_match_the_product_plan() {
    let config = Config::default();

    assert_eq!(config.schema_version, 1);
    assert_eq!(config.appearance.theme, "paper-light");
    assert_eq!(config.appearance.theme_dark, "paper-dark");
    assert!(config.appearance.follow_system);
    assert_eq!(config.typography.font_size, 16);
    assert_eq!(config.typography.measure_ch, 72);
    assert_eq!(config.viewer.default_mode, ViewMode::Preview);
    assert_eq!(config.editor.autosave_ms, 800);
    assert_eq!(config.editor.assets_dir, "assets");
    assert_eq!(config.history.interval_min, 5);
    assert_eq!(config.history.global_cap_mb, 2048);
    assert_eq!(config.files.export_ignore.len(), 4);
    assert_eq!(config.window.width, 1180);
    assert!(config.updates.check_on_launch);
    config.validate().expect("valid defaults");
}

#[test]
fn rejects_typography_values_outside_the_supported_ranges() {
    let mut config = Config::default();
    config.typography.font_size = 9;
    assert!(matches!(
        config.validate(),
        Err(Error::InvalidConfig {
            field: "font_size",
            ..
        })
    ));

    config.typography.font_size = 16;
    config.typography.measure_ch = 121;
    assert!(matches!(
        config.validate(),
        Err(Error::InvalidConfig {
            field: "measure_ch",
            ..
        })
    ));
}
