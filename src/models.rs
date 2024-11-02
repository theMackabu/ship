use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Config {
    pub(crate) settings: Settings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Settings {
    pub(crate) listen: String,
    pub(crate) storage: PathBuf,
    pub(crate) vault_url: String,
    pub(crate) vault_token: String,
}
