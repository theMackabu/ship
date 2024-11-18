use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Config {
    pub(crate) settings: Settings,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Settings {
    pub(crate) listen: String,
    pub(crate) storage: PathBuf,
    pub(crate) vault: Option<Vault>,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Vault {
    pub(crate) url: String,
    pub(crate) token: String,
}
