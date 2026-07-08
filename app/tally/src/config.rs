use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TallyConfig {
    pub file: Option<PathBuf>,
    pub theme: Option<String>,
    #[serde(default)]
    pub budgets: Vec<BudgetDef>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BudgetDef {
    pub account: String,
    pub monthly: f64,
    pub label: Option<String>,
}

impl TallyConfig {
    pub fn load() -> Self {
        if let Ok(text) = std::fs::read_to_string("tally.toml") {
            if let Ok(cfg) = toml::from_str(&text) {
                return cfg;
            }
        }
        if let Some(dirs) = directories::ProjectDirs::from("com", "tally", "tally") {
            let path = dirs.config_dir().join("tally.toml");
            if let Ok(text) = std::fs::read_to_string(path) {
                if let Ok(cfg) = toml::from_str(&text) {
                    return cfg;
                }
            }
        }
        Self::default()
    }
}
