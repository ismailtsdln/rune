use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub theme: String,
    pub show_line_numbers: bool,
    pub tab_size: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: String::from("dark"),
            show_line_numbers: true,
            tab_size: 4,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let config_path = "rune.toml";
        if Path::new(config_path).exists() {
            if let Ok(content) = fs::read_to_string(config_path) {
                if let Ok(config) = toml::from_str(&content) {
                    return config;
                }
            }
        }
        Self::default()
    }
}
