use std::path::PathBuf;

use fairy_core::{ModuleType, PackageJson};
use relative_path::RelativePathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Package {
    pub pkgjson: PackageJson,
    pub entry: RelativePathBuf,
    pub root: PathBuf,
}

impl std::ops::Deref for Package {
    type Target = PackageJson;
    fn deref(&self) -> &Self::Target {
        &self.pkgjson
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Dependency {
    pub name: String,
    pub root: PathBuf,
    pub entry: RelativePathBuf,
    pub kind: ModuleType,
}

impl Dependency {
    pub fn package(&self) -> anyhow::Result<Package> {
        Ok(Package {
            pkgjson: PackageJson::load(&self.root)?,
            entry: self.entry.clone(),
            root: self.root.clone(),
        })
    }

    // pub fn dependencies(&self) -> anyhow::Result<Vec<Dependency>> {
    //     let pkg = self.package()?;
    //     find_peer_dependencies(&pkg)
    // }
}
