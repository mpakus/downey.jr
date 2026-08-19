//! Built-in and user color themes mapped onto CSS variables.

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{Error, Result};

const BUILTIN_FILES: &[(&str, &str)] = &[
    ("paper-light", include_str!("../themes/paper-light.json")),
    ("paper-dark", include_str!("../themes/paper-dark.json")),
    (
        "solarized-light",
        include_str!("../themes/solarized-light.json"),
    ),
    (
        "solarized-dark",
        include_str!("../themes/solarized-dark.json"),
    ),
    ("nord", include_str!("../themes/nord.json")),
    (
        "gruvbox-light",
        include_str!("../themes/gruvbox-light.json"),
    ),
    ("gruvbox-dark", include_str!("../themes/gruvbox-dark.json")),
    (
        "catppuccin-latte",
        include_str!("../themes/catppuccin-latte.json"),
    ),
    (
        "catppuccin-mocha",
        include_str!("../themes/catppuccin-mocha.json"),
    ),
    ("tokyo-night", include_str!("../themes/tokyo-night.json")),
    ("github-light", include_str!("../themes/github-light.json")),
    ("github-dark", include_str!("../themes/github-dark.json")),
];

/// Light or dark appearance for a theme.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum ThemeAppearance {
    /// Light background.
    Light,
    /// Dark background.
    Dark,
}

/// A theme listed in Settings and applied via `data-theme`.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ThemeInfo {
    /// Stable identifier used as `data-theme`.
    pub id: String,
    /// User-visible name.
    pub name: String,
    /// Whether the theme is light or dark.
    pub appearance: ThemeAppearance,
    /// Whether the theme ships with the application.
    pub builtin: bool,
}

#[derive(Clone, Debug, Deserialize)]
struct ThemeFile {
    id: String,
    name: String,
    appearance: ThemeAppearance,
    tokens: ThemeTokens,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct ThemeTokens {
    bg: String,
    bg_elev: String,
    sidebar: String,
    fg: String,
    fg_muted: String,
    border: String,
    accent: String,
    selection: String,
    code_bg: String,
    hl_kw: String,
    hl_str: String,
    hl_num: String,
    hl_com: String,
    hl_fn: String,
    hl_type: String,
    ed_cursor: String,
    ed_sel: String,
    ed_active_line: String,
    ed_syntax: String,
}

struct LoadedTheme {
    info: ThemeInfo,
    tokens: ThemeTokens,
}

/// Built-in themes plus valid user themes from `~/.1537paperstreet/themes/`.
pub struct ThemeCatalog {
    themes: Vec<LoadedTheme>,
    /// Parse errors for skipped user theme files. Safe to show in logs.
    pub warnings: Vec<String>,
}

impl ThemeCatalog {
    /// Loads the twelve built-in themes and any valid JSON files in `user_dir`.
    pub fn load(user_dir: &Path) -> Self {
        let mut themes = Vec::with_capacity(BUILTIN_FILES.len());
        let mut warnings = Vec::new();
        for json in BUILTIN_FILES.iter().map(|(_, json)| *json) {
            match parse_theme(json, true) {
                Ok(theme) => themes.push(theme),
                Err(error) => warnings.push(error.to_string()),
            }
        }

        if user_dir.is_dir() {
            match fs::read_dir(user_dir) {
                Ok(entries) => {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                            continue;
                        }
                        match load_user_theme(&path) {
                            Ok(theme) => {
                                if themes
                                    .iter()
                                    .any(|existing| existing.info.id == theme.info.id)
                                {
                                    warnings.push(format!(
                                        "skipped '{}': a theme named '{}' is already loaded",
                                        path.display(),
                                        theme.info.id
                                    ));
                                    continue;
                                }
                                themes.push(theme);
                            }
                            Err(error) => {
                                warnings.push(format!("skipped '{}': {error}", path.display()))
                            }
                        }
                    }
                }
                Err(source) => {
                    warnings.push(format!("couldn't read '{}': {source}", user_dir.display()))
                }
            }
        }

        Self { themes, warnings }
    }

    /// Summaries for Settings and the theme attribute.
    pub fn infos(&self) -> Vec<ThemeInfo> {
        self.themes.iter().map(|theme| theme.info.clone()).collect()
    }

    /// CSS that maps every loaded theme onto variables. Switching themes only
    /// changes `data-theme` on `<html>`.
    pub fn css(&self) -> String {
        let mut css = String::new();
        for theme in &self.themes {
            css.push_str(&theme_css(
                &theme.info.id,
                theme.info.appearance,
                &theme.tokens,
            ));
        }
        css
    }
}

fn load_user_theme(path: &Path) -> Result<LoadedTheme> {
    let json = fs::read_to_string(path)
        .map_err(|source| Error::io("read the theme file", path, source))?;
    parse_theme(&json, false)
}

fn parse_theme(json: &str, builtin: bool) -> Result<LoadedTheme> {
    let file: ThemeFile = serde_json::from_str(json).map_err(|_| Error::InvalidTheme {
        reason: "the file is not valid theme JSON",
    })?;
    validate_theme(&file)?;
    Ok(LoadedTheme {
        info: ThemeInfo {
            id: file.id,
            name: file.name,
            appearance: file.appearance,
            builtin,
        },
        tokens: file.tokens,
    })
}

fn validate_theme(file: &ThemeFile) -> Result<()> {
    if file.id.trim().is_empty()
        || !file
            .id
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '-')
    {
        return Err(Error::InvalidTheme {
            reason: "the theme id must be a slug of ASCII letters, digits, and hyphens",
        });
    }
    if file.name.trim().is_empty() {
        return Err(Error::InvalidTheme {
            reason: "the theme name is empty",
        });
    }
    for (_name, value) in token_pairs(&file.tokens) {
        if !is_hex_color(value) {
            return Err(Error::InvalidTheme {
                reason: "a theme color is not a #RRGGBB value",
            });
        }
    }
    Ok(())
}

fn is_hex_color(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 7 && bytes[0] == b'#' && bytes[1..].iter().all(|byte| byte.is_ascii_hexdigit())
}

fn token_pairs(tokens: &ThemeTokens) -> [(&'static str, &str); 19] {
    [
        ("bg", &tokens.bg),
        ("bg-elev", &tokens.bg_elev),
        ("sidebar", &tokens.sidebar),
        ("fg", &tokens.fg),
        ("fg-muted", &tokens.fg_muted),
        ("border", &tokens.border),
        ("accent", &tokens.accent),
        ("selection", &tokens.selection),
        ("code-bg", &tokens.code_bg),
        ("hl-kw", &tokens.hl_kw),
        ("hl-str", &tokens.hl_str),
        ("hl-num", &tokens.hl_num),
        ("hl-com", &tokens.hl_com),
        ("hl-fn", &tokens.hl_fn),
        ("hl-type", &tokens.hl_type),
        ("ed-cursor", &tokens.ed_cursor),
        ("ed-sel", &tokens.ed_sel),
        ("ed-active-line", &tokens.ed_active_line),
        ("ed-syntax", &tokens.ed_syntax),
    ]
}

fn theme_css(id: &str, appearance: ThemeAppearance, tokens: &ThemeTokens) -> String {
    let scheme = match appearance {
        ThemeAppearance::Light => "light",
        ThemeAppearance::Dark => "dark",
    };
    let mut css = format!("[data-theme='{id}'] {{\n  color-scheme: {scheme};\n");
    for (name, value) in token_pairs(tokens) {
        css.push_str("  --");
        css.push_str(name);
        css.push_str(": ");
        css.push_str(value);
        css.push_str(";\n");
    }
    css.push_str("}\n");
    css
}
