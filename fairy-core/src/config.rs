use core::fmt;
use relative_path::RelativePathBuf;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

pub type Environ = HashMap<String, String>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub root: PathBuf,
    pub entry: RelativePathBuf,
    #[serde(default)]
    pub env: Environ,
    #[serde(default)]
    pub plugins: Vec<Box<dyn FileLoader>>,
}

#[typetag::serde]
pub trait FileLoader: fmt::Debug {}
