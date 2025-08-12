use serde::{Deserialize, Serialize};
use crate::session::theme::Theme;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    pub name: Option<String>,
    pub description: Option<String>,
    pub provider: Option<String>,
    pub api_key: String,
    pub api_url: String,
    pub api_version: Option<String>,
    pub model: String
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub max_sessions: usize,
    pub auto_save: bool,
    pub default_model: Model,
    pub models: Option<Vec<Model>>,
    pub theme: Theme,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            max_sessions: 100,
            auto_save: true,
            default_model: Model {
                name: Some("deepseek-chat".to_string()),
                api_version: Some("v3".to_string()),
                provider: Some("v3".to_string()),
                description: Some("v3".to_string()),
                model: "deepseek-chat".to_string(),
                api_key: "".to_string(),
                api_url: "https://api.deepseek.com/chat/completions".to_string(),
            },
            models: None,
            theme: Theme::Dark,
        }
    }
}