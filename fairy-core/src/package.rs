use std::{collections::HashMap, path::PathBuf};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub static PACKAGE_JSON: &'static str = "package.json";

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum ModuleType {
    #[serde(alias = "module")]
    Esm,
    #[serde(alias = "commonjs")]
    Commonjs,
}

impl Default for ModuleType {
    fn default() -> Self {
        ModuleType::Commonjs
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Browser {
    Str(String),
    Obj(HashMap<String, String>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Exports {
    Str(String),
    Obj(HashMap<String, Exports>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PackageJson {
    pub name: String,
    pub main: Option<String>,
    pub browser: Option<Browser>,
    pub module: Option<String>,
    pub exports: Option<Exports>,
    #[serde(rename = "type", default)]
    pub kind: ModuleType,
    #[serde(default)]
    pub dependencies: HashMap<String, String>,
    #[serde(rename = "peerDependencies", default)]
    pub peer_dependencies: HashMap<String, String>,
}

impl PackageJson {
    pub fn load(root: &Path) -> anyhow::Result<PackageJson> {
        let data =
            std::fs::read(root.join(PACKAGE_JSON)).context(format!("at root: {:?}", root))?;
        Ok(serde_json::from_slice(&data)?)
    }
}
