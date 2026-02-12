use serde::Deserialize;
use std::path::Path;

#[derive(Clone, Debug, Deserialize)]
pub struct CitySelectorConfig {
    pub lobby: LobbyConfig,
    pub servers: ServersConfig,
}

#[derive(Clone, Debug, Deserialize)]
pub struct LobbyConfig {
    #[serde(default = "default_true")]
    pub give_compass_on_join: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, Deserialize)]
pub struct ServersConfig {
    #[serde(default = "default_ratsku")]
    pub ratsku: String,
    #[serde(default = "default_vatsku")]
    pub vatsku: String,
    #[serde(default = "default_ratuskuu")]
    pub ratuskuu: String,
    #[serde(default = "default_ajkaz")]
    pub ajkaz: String,
}

fn default_ratsku() -> String {
    "ratsku".into()
}
fn default_vatsku() -> String {
    "vatsku".into()
}
fn default_ratuskuu() -> String {
    "ratuskuu".into()
}
fn default_ajkaz() -> String {
    "ajkaz".into()
}

impl Default for CitySelectorConfig {
    fn default() -> Self {
        Self {
            lobby: LobbyConfig {
                give_compass_on_join: true,
            },
            servers: ServersConfig {
                ratsku: "ratsku".into(),
                vatsku: "vatsku".into(),
                ratuskuu: "ratuskuu".into(),
                ajkaz: "ajkaz".into(),
            },
        }
    }
}

impl CitySelectorConfig {
    pub fn load(path: &Path) -> Result<Self, String> {
        let s = std::fs::read_to_string(path).map_err(|e| format!("read config: {e}"))?;
        toml::from_str(&s).map_err(|e| format!("parse config: {e}"))
    }
}
